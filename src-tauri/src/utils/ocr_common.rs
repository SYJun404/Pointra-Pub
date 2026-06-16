// ── 数据结构 ──────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct WordBox {
    pub text: String,
    /// 归一化坐标（0.0 ~ 1.0，原点左上）
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// BBox 计算内部用的归一化矩形（与 WordBox 字段布局一致）
#[derive(Debug, Clone, Copy)]
pub(crate) struct NormalizedRect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

// ── 字符分类 ──────────────────────────────────────────────

#[inline]
pub(crate) fn is_token_char(c: char) -> bool {
    c.is_ascii_alphabetic()
}

#[inline]
pub(crate) fn is_uppercase(c: char) -> bool {
    c.is_ascii_uppercase()
}

#[inline]
pub(crate) fn is_lowercase(c: char) -> bool {
    c.is_ascii_lowercase()
}

/// CamelCase 拆分：HelloWorld → [Hello, World]
pub(crate) fn refined_latin_ranges(s: &str) -> Vec<std::ops::Range<usize>> {
    let chars: Vec<(usize, char)> = s.char_indices().collect();
    let mut ranges = Vec::new();
    let mut start: Option<usize> = None;
    let mut prev: Option<char> = None;

    for i in 0..chars.len() {
        let (byte_idx, cur) = chars[i];
        let next = chars.get(i + 1).map(|&(_, c)| c);

        if is_token_char(cur) {
            if start.is_none() {
                start = Some(byte_idx);
            } else if let Some(p) = prev {
                let split = (is_lowercase(p) && is_uppercase(cur))
                    || (is_uppercase(p)
                        && is_uppercase(cur)
                        && next.map(is_lowercase).unwrap_or(false));
                if split {
                    ranges.push(start.unwrap()..byte_idx);
                    start = Some(byte_idx);
                }
            }
            prev = Some(cur);
        } else {
            if let Some(s) = start {
                ranges.push(s..byte_idx);
            }
            start = None;
            prev = None;
        }
    }
    if let Some(s) = start {
        ranges.push(s..s_byte_end(s, &chars));
    }
    ranges
}

fn s_byte_end(start: usize, chars: &[(usize, char)]) -> usize {
    chars
        .iter()
        .rev()
        .find(|(bi, _)| *bi >= start)
        .map(|(bi, c)| bi + c.len_utf8())
        .unwrap_or(start)
}

// ── 脚本类型 ──────────────────────────────────────────────

#[derive(PartialEq)]
pub(crate) enum TokenKind {
    Latin,
    Han,
    OtherLetter,
}

pub(crate) fn token_kind(c: char) -> Option<TokenKind> {
    if c.is_ascii_alphabetic() {
        return Some(TokenKind::Latin);
    }
    let cp = c as u32;
    if (0x4E00..=0x9FFF).contains(&cp)
        || (0x3400..=0x4DBF).contains(&cp)
        || (0x20000..=0x2A6DF).contains(&cp)
        || (0x3040..=0x30FF).contains(&cp)
    {
        return Some(TokenKind::Han);
    }
    if c.is_alphabetic() {
        return Some(TokenKind::OtherLetter);
    }
    None
}

/// 脚本感知分词：拉丁段做 CamelCase 拆分，汉字段每字单独成 token。
/// 这样 "调用APIRequest时" 会产生 ["API", "Request", "时"] 三个 token，
/// 保证中英混排行里的英文单词都能被正确提取并定位。
pub(crate) fn script_aware_ranges(text: &str) -> Vec<std::ops::Range<usize>> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut ranges = Vec::new();
    let mut seg_start: Option<usize> = None;
    let mut cur_kind: Option<TokenKind> = None;

    let flush = |seg_start: usize, end: usize, kind: &TokenKind, ranges: &mut Vec<_>| {
        if seg_start >= end {
            return;
        }
        match kind {
            TokenKind::Latin => ranges.extend(
                refined_latin_ranges(&text[seg_start..end])
                    .into_iter()
                    .map(|r| (seg_start + r.start)..(seg_start + r.end)),
            ),
            _ => ranges.push(seg_start..end),
        }
    };

    for &(byte_idx, c) in &chars {
        match token_kind(c) {
            Some(k) => match &cur_kind {
                None => {
                    seg_start = Some(byte_idx);
                    cur_kind = Some(k);
                }
                Some(ck) if *ck != k => {
                    let ss = seg_start.unwrap();
                    flush(ss, byte_idx, ck, &mut ranges);
                    seg_start = Some(byte_idx);
                    cur_kind = Some(k);
                }
                _ => {}
            },
            None => {
                if let (Some(ss), Some(ck)) = (seg_start, &cur_kind) {
                    flush(ss, byte_idx, ck, &mut ranges);
                }
                seg_start = None;
                cur_kind = None;
            }
        }
    }
    if let (Some(ss), Some(ck)) = (seg_start, &cur_kind) {
        flush(ss, text.len(), ck, &mut ranges);
    }
    ranges
}

/// token 包含至少一个英文字母（汉字行里的中文 token 会被过滤掉，只保留英文）
#[inline]
pub(crate) fn contains_letter(s: &str) -> bool {
    s.bytes().any(|b| b.is_ascii_alphabetic())
}

// ── BBox 计算 ─────────────────────────────────────────────

pub(crate) fn fallback_box(
    text_box: NormalizedRect,
    text: &str,
    range: &std::ops::Range<usize>,
) -> Option<NormalizedRect> {
    let total = if text.is_ascii() {
        text.len()
    } else {
        text.chars().count()
    };
    if total == 0 {
        return None;
    }
    let sc = if text.is_ascii() {
        range.start
    } else {
        text[..range.start].chars().count()
    };
    let ec = if text.is_ascii() {
        range.end
    } else {
        text[..range.end].chars().count()
    };
    if ec <= sc {
        return None;
    }
    let sf = sc as f64 / total as f64;
    let ef = ec as f64 / total as f64;
    let x = text_box.x + text_box.w * sf;
    let w = text_box.w * (ef - sf);
    if w <= 0.0 {
        return None;
    }
    Some(NormalizedRect {
        x,
        y: text_box.y,
        w,
        h: text_box.h,
    })
}

pub(crate) fn is_valid_box(b: NormalizedRect, parent: NormalizedRect) -> bool {
    if b.w <= 0.0 || b.h <= 0.0 {
        return false;
    }
    let tol = 0.02;
    b.x >= parent.x - tol
        && b.y >= parent.y - tol
        && (b.x + b.w) <= (parent.x + parent.w + tol)
        && (b.y + b.h) <= (parent.y + parent.h + tol)
}

pub(crate) fn is_compatible_subrange(
    precise: NormalizedRect,
    fallback: NormalizedRect,
    parent: NormalizedRect,
) -> bool {
    if precise.w <= 0.0 || fallback.w <= 0.0 {
        return false;
    }
    if precise.w >= parent.w * 0.8 && precise.w > fallback.w * 1.8 {
        return false;
    }
    let tol = (parent.w * 0.01).max(fallback.w * 0.35);
    if precise.x < fallback.x - tol {
        return false;
    }
    if precise.x + precise.w > fallback.x + fallback.w + tol {
        return false;
    }
    let overlap = (precise.x + precise.w).min(fallback.x + fallback.w) - precise.x.max(fallback.x);
    overlap.max(0.0) / precise.w.min(fallback.w) >= 0.35
}

pub(crate) fn resolved_box(
    precise: Option<NormalizedRect>,
    fallback: Option<NormalizedRect>,
    parent: NormalizedRect,
    is_sub: bool,
) -> Option<NormalizedRect> {
    if let Some(p) = precise {
        if is_valid_box(p, parent) {
            if is_sub {
                if let Some(f) = fallback {
                    if !is_compatible_subrange(p, f, parent) {
                        return fallback;
                    }
                }
            }
            return Some(p);
        }
    }
    fallback
}

// ── 命中检测 ──────────────────────────────────────

pub fn select_word(words: &[WordBox]) -> Option<String> {
    // 只有一个结果时直接返回，无需命中检测
    if words.len() == 1 {
        return Some(words[0].text.clone());
    }

    const CX: f64 = 0.5;
    const CY: f64 = 0.5;
    const TOL: f64 = 0.004;

    let mut best_exact: Option<(f64, &WordBox)> = None;
    let mut best_fuzzy: Option<(f64, &WordBox)> = None;

    for w in words {
        let cx = w.x + w.w * 0.5;
        let cy = w.y + w.h * 0.5;
        let dist2 = (cx - CX).powi(2) + (cy - CY).powi(2);

        let in_exact = CX >= w.x && CX <= w.x + w.w && CY >= w.y && CY <= w.y + w.h;
        let in_fuzzy = !in_exact
            && CX >= w.x - TOL
            && CX <= w.x + w.w + TOL
            && CY >= w.y - TOL
            && CY <= w.y + w.h + TOL;

        if in_exact {
            if best_exact.map_or(true, |(d, _)| dist2 < d) {
                best_exact = Some((dist2, w));
            }
        } else if in_fuzzy {
            if best_fuzzy.map_or(true, |(d, _)| dist2 < d) {
                best_fuzzy = Some((dist2, w));
            }
        }
    }
    best_exact.or(best_fuzzy).map(|(_, w)| w.text.clone())
}

pub const CURSOR_NX: f64 = 0.5;
pub const WORD_GAP_FACTOR: f64 = 1.8;

fn ends_sentence(w: &WordBox) -> bool {
    matches!(w.text.trim_end().chars().last(), Some('.' | '!' | '?'))
}

pub fn select_sentence(words: &[WordBox]) -> Option<String> {
    if words.is_empty() {
        return None;
    }

    let mut sorted: Vec<&WordBox> = words.iter().collect();
    sorted.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));

    let mut widths: Vec<f64> = sorted.iter().map(|w| w.w).collect();
    widths.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_w = widths[widths.len() / 2];
    let gap_threshold = median_w * WORD_GAP_FACTOR;

    let pivot = sorted
        .iter()
        .position(|w| CURSOR_NX >= w.x && CURSOR_NX <= w.x + w.w)
        .unwrap_or_else(|| {
            sorted
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    let da = (a.x + a.w * 0.5 - CURSOR_NX).abs();
                    let db = (b.x + b.w * 0.5 - CURSOR_NX).abs();
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(i, _)| i)
                .unwrap_or(0)
        });

    let start = {
        let mut i = pivot;
        while i > 0 {
            let gap = sorted[i].x - (sorted[i - 1].x + sorted[i - 1].w);
            if gap > gap_threshold {
                break;
            }
            if ends_sentence(sorted[i - 1]) {
                break;
            }
            i -= 1;
        }
        i
    };

    let end = {
        let mut i = pivot;
        loop {
            if ends_sentence(sorted[i]) {
                i += 1;
                break;
            }
            if i + 1 >= sorted.len() {
                i += 1;
                break;
            }
            let gap = sorted[i + 1].x - (sorted[i].x + sorted[i].w);
            if gap > gap_threshold {
                i += 1;
                break;
            }
            i += 1;
        }
        i
    };

    let s = sorted[start..end]
        .iter()
        .map(|w| w.text.as_str())
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
