use arboard::Clipboard;
use std::error::Error;
use std::sync::atomic::{AtomicU32, Ordering};
use windows::Win32::Foundation::POINT;
use windows::Win32::System::Com::{CoCreateInstance, CoInitialize, CLSCTX_ALL};
use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationTextPattern, TextUnit_Word, UIA_TextPatternId,
};

/// 取词来源标记（随取词结果一起发给前端，便于诊断"这次结果是怎么来的"）
pub const SOURCE_UIA_SELECTION: &str = "uia-selection";
pub const SOURCE_UIA_POINT: &str = "uia-point";
pub const SOURCE_CLIPBOARD_COPY: &str = "clipboard-copy";
pub const SOURCE_CLIPBOARD_CURRENT: &str = "clipboard-current";

/// "鼠标下的词"允许的最大长度，超过则认为取到的是一整行/一整段，丢弃
const MAX_POINT_WORD_LEN: usize = 48;

/// 回退读取剪贴板时允许的最大长度（避免把很久以前复制的大段文本当成取词结果）
const MAX_CLIPBOARD_FALLBACK_LEN: usize = 512;

/// 上一次取词时看到的剪贴板序列号。
/// 「选择即复制」的 TUI（pi / opencode）会在用户拖选结束时立刻写入剪贴板，
/// 所以序列号相对上一次取词发生了变化 ⇒ 当前剪贴板内容就是本次拖选写进去的。
/// 这是比「跟鼠标下的词比内容」可靠得多的信号：鼠标很可能停在选区之外。
static LAST_CLIPBOARD_SEQ: AtomicU32 = AtomicU32::new(0);

/// 剪贴板是否「新鲜」（相对上一次取词发生了变化），并记录本次序列号。
fn clipboard_is_fresh() -> bool {
    let now = unsafe { GetClipboardSequenceNumber() };
    let last = LAST_CLIPBOARD_SEQ.swap(now, Ordering::Relaxed);
    // 首次调用（还没有基准值）时按「新鲜」处理，
    // 保证应用启动后第一次取词也能用上剪贴板。
    last == 0 || now != last
}

/// debug 构建下打印取词过程（release 无控制台，展开为空，不产生任何开销）
macro_rules! pick_log {
    ($($arg:tt)*) => {{
        #[cfg(debug_assertions)]
        eprintln!("[Pointra][取词] {}", format!($($arg)*));
    }};
}

pub fn get_text() -> Option<(String, i32, i32, &'static str)> {
    // 1) 焦点元素的文本选区（浏览器 / Edge / UWP 等有真实选区的应用）
    match get_selection_by_automation() {
        Ok(text) if !text.is_empty() => {
            pick_log!("① uia-selection 命中: {text:?}");
            return Some((text, 23, 15, SOURCE_UIA_SELECTION));
        }
        Ok(_) => pick_log!("① uia-selection 为空（焦点元素没有选区）"),
        Err(err) => pick_log!("① uia-selection 失败: {err}"),
    }

    // 先记录「本次按下快捷键时」剪贴板有没有刚被更新过（必须在 ② 之前读取！）：
    // ① copy() 本身也会改变剪贴板序列号，若在其后才读，会把「copy 的副作用」
    // 误判成「用户刚拖选过」，导致 ④ 用上陈旧内容。
    // TUI 的「选择即复制」会在拖选结束时写剪贴板，这才是判断 ④ 新鲜度的硬信号。
    let clipboard_fresh = clipboard_is_fresh();

    // 2) 模拟 Ctrl+C 后读到的新剪贴板内容（有真实选区时，它就是选区内容）
    match get_text_by_clipboard() {
        Ok(text) if !text.is_empty() => {
            pick_log!("② clipboard-copy 命中: {text:?}");
            return Some((text, 23, 15, SOURCE_CLIPBOARD_COPY));
        }
        Ok(_) => pick_log!("② clipboard-copy 复制到空内容（终端无选区时是常态）"),
        Err(err) => pick_log!("② clipboard-copy 失败: {err}"),
    }

    // 顺便探一下「鼠标下的词」：它既能做兵底，也能在剪贴板不新鲜时帮忙判断是不是同一段内容。
    // TUI / 终端启用鼠标跟踪后不会建立终端选区，但控件依旧支持 TextPattern。
    let point_word = match cursor_position() {
        Some((x, y)) => match get_word_by_automation_at(x, y) {
            Ok(text) if !text.is_empty() => {
                pick_log!("③ uia-point 候选 (鼠标 {x},{y}): {text:?}");
                Some((text, x, y))
            }
            Ok(_) => {
                pick_log!("③ uia-point 候选为空 (鼠标 {x},{y})");
                None
            }
            Err(err) => {
                pick_log!("③ uia-point 候选失败 (鼠标 {x},{y}): {err}");
                None
            }
        },
        None => {
            pick_log!("③ uia-point 候选跳过：拿不到鼠标位置");
            None
        }
    };

    // 4) 当前剪贴板内容：现代 TUI（pi / opencode）在「选择即复制」下，
    //    用户一拖选就已经把**完整选中内容**写进剪贴板了，它比「鼠标下的那个分词」更接近用户意图。
    //    但它也可能是很久以前复制的文本，所以用上面探到的词交叉验证：
    //    两者只要是「同一段内容的不同截取」（大面积重叠）就采用剪贴板。
    //
    //    注意：这里**不能**用「严格互相包含」做判据。
    //    实测中 TUI 在宽字符 / 反引号边界上会差 1 个字符：
    //      剪贴板   = "停 1~2 秒再关掉（方便日志落盘"
    //      鼠标词 = "秒再关掉（方便日志落盘））→"
    //    两者互不包含，但它们显然是同一段选区，剪贴板那个才是用户真正选中的。
    if let Some(clip) = current_clipboard_text() {
        let overlap = match &point_word {
            Some((word, _, _)) => same_selection_ratio(&clip, word),
            None => 0.0,
        };

        // 采用剪贴板的三种情形：
        //   a) 剪贴板刚被更新过（拖选 → 选择即复制）—— 最可靠的信号
        //   b) 与鼠标下的词大面积重合（同一段内容的不同截取，容忍 ±1 字符的边界差）
        //   c) 鼠标下取不到词，没有更好的候选
        let use_clipboard = clipboard_fresh || overlap >= 0.5 || point_word.is_none();

        if use_clipboard {
            pick_log!(
                "④ clipboard-current 命中（新鲜={clipboard_fresh}，与鼠标词重合 {:.0}%）: {clip:?}",
                overlap * 100.0
            );
            return Some((clip, 23, 15, SOURCE_CLIPBOARD_CURRENT));
        }
        pick_log!(
            "④ clipboard-current 跳过（不新鲜且与鼠标词仅重合 {:.0}%）: {clip:?}，改用鼠标下的词",
            overlap * 100.0
        );
    } else {
        pick_log!("④ clipboard-current 为空/过长");
    }

    // 5) 都没有时，用鼠标下那个词做最后兵底（至少是屏幕上的真实文本，不是 OCR 猜的）
    if let Some((word, x, y)) = point_word {
        pick_log!("⑤ uia-point 命中 (鼠标 {x},{y}): {word:?}");
        return Some((word, 23, 15, SOURCE_UIA_POINT));
    }

    pick_log!("五条路径都拿不到，交给 OCR 兜底");
    None
}

/// 两个文本是不是「同一段内容的不同截取」？
/// 用「最长公共子串」占较短一方的比例衡量：
/// - TUI 的选区与 UIA 分词单元常有 ± 1 字符的边界差，比例会很高（> 0.5）；
/// - 很久以前复制的无关文本比例几乎为 0。
fn same_selection_ratio(a: &str, b: &str) -> f64 {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    // 经典 DP 求最长公共子串长度（只保留上一行）
    let mut prev = vec![0usize; b.len() + 1];
    let mut best = 0usize;
    for ac in a.iter() {
        let mut cur = vec![0usize; b.len() + 1];
        for (j, bc) in b.iter().enumerate() {
            if ac == bc {
                cur[j + 1] = prev[j] + 1;
                if cur[j + 1] > best {
                    best = cur[j + 1];
                }
            }
        }
        prev = cur;
    }

    best as f64 / a.len().min(b.len()) as f64
}

fn cursor_position() -> Option<(i32, i32)> {
    match mouse_position::mouse_position::Mouse::get_mouse_position() {
        mouse_position::mouse_position::Mouse::Position { x, y } => Some((x, y)),
        mouse_position::mouse_position::Mouse::Error => None,
    }
}

// Available for Edge, Chrome and UWP
fn get_selection_by_automation() -> Result<String, Box<dyn Error>> {
    // Init COM
    let _ = unsafe { CoInitialize(None) };
    // Create IUIAutomation instance
    let auto: IUIAutomation = unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_ALL) }?;
    // Get Focused Element
    let el = unsafe { auto.GetFocusedElement() }?;
    // Get TextPattern
    let res: IUIAutomationTextPattern = unsafe { el.GetCurrentPatternAs(UIA_TextPatternId) }?;
    // Get TextRange Array
    let text_array = unsafe { res.GetSelection() }?;
    let length = unsafe { text_array.Length() }?;
    // Iterate TextRange Array
    let mut target = String::new();
    for i in 0..length {
        let text = unsafe { text_array.GetElement(i) }?;
        let str = unsafe { text.GetText(-1) }?;
        let str = str.to_string();
        target.push_str(&str);
    }
    Ok(target.trim().to_string())
}

/// 用 UIA 取「鼠标位置所在的那个词」。
/// 优先用 ElementFromPoint（即使焦点不在该窗口也能取到），失败再退回焦点元素。
fn get_word_by_automation_at(x: i32, y: i32) -> Result<String, Box<dyn Error>> {
    let _ = unsafe { CoInitialize(None) };
    let auto: IUIAutomation = unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_ALL) }?;
    let point = POINT { x, y };

    let pattern = match unsafe { auto.ElementFromPoint(point) } {
        Ok(el) => {
            unsafe { el.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId) }.ok()
        }
        Err(_) => None,
    }
    .or_else(|| {
        let el = unsafe { auto.GetFocusedElement() }.ok()?;
        unsafe { el.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId) }.ok()
    });

    let pattern = pattern.ok_or("no text pattern at cursor")?;

    let range = unsafe { pattern.RangeFromPoint(point) }?;
    // RangeFromPoint 返回的是「退化范围」（零长度），需要扩到整个词才能拿到文本
    let _ = unsafe { range.ExpandToEnclosingUnit(TextUnit_Word) };
    let text = unsafe { range.GetText(-1) }?.to_string();
    let text = text.trim().to_string();

    // 有些提供程序会把整行当成一个 word，这里做长度兜底
    if text.is_empty() || text.chars().count() > MAX_POINT_WORD_LEN {
        return Err("cursor word is empty or too long".into());
    }

    Ok(text)
}

// Available for almost all applications
fn get_text_by_clipboard() -> Result<String, Box<dyn Error>> {
    // Read Old Clipboard
    let old_clipboard = (Clipboard::new()?.get_text(), Clipboard::new()?.get_image());

    if copy() {
        // Read New Clipboard
        let new_text = Clipboard::new()?.get_text();

        // Create Write Clipboard
        let mut write_clipboard = Clipboard::new()?;

        match old_clipboard {
            (Ok(text), _) => {
                // Old Clipboard is Text
                write_clipboard.set_text(text)?;
                if let Ok(new) = new_text {
                    Ok(new.trim().to_string())
                } else {
                    Err("New clipboard is not Text".into())
                }
            }
            (_, Ok(image)) => {
                // Old Clipboard is Image
                write_clipboard.set_image(image)?;
                if let Ok(new) = new_text {
                    Ok(new.trim().to_string())
                } else {
                    Err("New clipboard is not Text".into())
                }
            }
            _ => {
                // Old Clipboard is Empty
                write_clipboard.clear()?;
                if let Ok(new) = new_text {
                    Ok(new.trim().to_string())
                } else {
                    Err("New clipboard is not Text".into())
                }
            }
        }
    } else {
        Err("Copy Failed".into())
    }
}

/// 直接读取当前剪贴板文本（不做 Ctrl+C，也不改动剪贴板）。
/// 用于「选择即复制」的 TUI 场景：用户拖选时目标程序已经把选中内容写进剪贴板。
fn current_clipboard_text() -> Option<String> {
    let text = Clipboard::new().ok()?.get_text().ok()?;
    let text = text.trim().to_string();
    if text.is_empty() || text.chars().count() > MAX_CLIPBOARD_FALLBACK_LEN {
        return None;
    }
    Some(text)
}

fn copy() -> bool {
    use enigo::{
        Direction::{Click, Press, Release},
        Enigo, Key, Keyboard, Settings,
    };
    let num_before = unsafe { GetClipboardSequenceNumber() };

    // 这些按键调用失败（例如没有输入权限）不应该 panic 掉整个取词任务，
    // 直接返回 false，让上层降级到其它取词途径。
    let Ok(mut enigo) = Enigo::new(&Settings::default()) else {
        return false;
    };
    let _ = enigo.key(Key::Control, Release);
    let _ = enigo.key(Key::Alt, Release);
    let _ = enigo.key(Key::Shift, Release);
    let _ = enigo.key(Key::Space, Release);
    let _ = enigo.key(Key::Meta, Release);
    let _ = enigo.key(Key::Tab, Release);
    let _ = enigo.key(Key::Escape, Release);
    let _ = enigo.key(Key::CapsLock, Release);
    let _ = enigo.key(Key::C, Release);
    let _ = enigo.key(Key::Control, Press);
    let _ = enigo.key(Key::C, Click);
    let _ = enigo.key(Key::Control, Release);
    std::thread::sleep(std::time::Duration::from_millis(45));
    let num_after = unsafe { GetClipboardSequenceNumber() };
    num_after != num_before
}
