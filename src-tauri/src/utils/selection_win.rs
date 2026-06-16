use arboard::Clipboard;
use std::error::Error;
use windows::Win32::System::Com::{CoCreateInstance, CoInitialize, CLSCTX_ALL};
use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationTextPattern, UIA_TextPatternId,
};
pub fn get_text() -> Option<(String, i32, i32)> {
    match get_text_by_automation() {
        Ok(text) => {
            if !text.is_empty() {
                return Some((text, 23, 15));
            }
        }
        Err(_) => {
            // eprintln!("get_text_by_automation error:{}", err);
        }
    }

    match get_text_by_clipboard() {
        Ok(text) => {
            if !text.is_empty() {
                return Some((text, 23, 15));
            } else {
                // println!("get_text_by_clipboard is empty");
            }
        }
        Err(_) => {
            // eprintln!("get_text_by_clipboard error:{}", err);
        }
    }
    None
}

// Available for Edge, Chrome and UWP
fn get_text_by_automation() -> Result<String, Box<dyn Error>> {
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

fn copy() -> bool {
    use enigo::{
        Direction::{Click, Press, Release},
        Enigo, Key, Keyboard, Settings,
    };
    let num_before = unsafe { GetClipboardSequenceNumber() };

    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    enigo.key(Key::Control, Release).unwrap();
    enigo.key(Key::Alt, Release).unwrap();
    enigo.key(Key::Shift, Release).unwrap();
    enigo.key(Key::Space, Release).unwrap();
    enigo.key(Key::Meta, Release).unwrap();
    enigo.key(Key::Tab, Release).unwrap();
    enigo.key(Key::Escape, Release).unwrap();
    enigo.key(Key::CapsLock, Release).unwrap();
    enigo.key(Key::C, Release).unwrap();
    enigo.key(Key::Control, Press).unwrap();
    enigo.key(Key::C, Click).unwrap();
    enigo.key(Key::Control, Release).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(45));
    let num_after = unsafe { GetClipboardSequenceNumber() };
    num_after != num_before
}
