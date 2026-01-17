use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop, CFRunLoopSource};
use core_graphics::event::{
    CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType, CallbackResult, EventField,
};
use std::collections::HashMap;
use std::sync::mpsc;
use yashiki_ipc::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hotkey {
    pub key_code: u16,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Modifiers {
    pub cmd: bool,
    pub alt: bool,
    pub ctrl: bool,
    pub shift: bool,
}

pub fn parse_hotkey(key_str: &str) -> Result<Hotkey, String> {
    let parts: Vec<&str> = key_str.split('-').collect();
    if parts.is_empty() {
        return Err("Empty key string".to_string());
    }

    let mut modifiers = Modifiers::default();
    let key_part = parts.last().unwrap();

    for part in &parts[..parts.len() - 1] {
        match part.to_lowercase().as_str() {
            "cmd" | "super" | "command" => modifiers.cmd = true,
            "alt" | "opt" | "option" => modifiers.alt = true,
            "ctrl" | "control" => modifiers.ctrl = true,
            "shift" => modifiers.shift = true,
            _ => return Err(format!("Unknown modifier: {}", part)),
        }
    }

    let key_code = parse_key_code(key_part)?;

    Ok(Hotkey {
        key_code,
        modifiers,
    })
}

pub fn format_hotkey(hotkey: &Hotkey) -> String {
    let mut parts = Vec::new();
    if hotkey.modifiers.cmd {
        parts.push("cmd");
    }
    if hotkey.modifiers.alt {
        parts.push("alt");
    }
    if hotkey.modifiers.ctrl {
        parts.push("ctrl");
    }
    if hotkey.modifiers.shift {
        parts.push("shift");
    }
    parts.push(key_code_to_str(hotkey.key_code));
    parts.join("-")
}

pub struct HotkeyManager {
    bindings: HashMap<Hotkey, Command>,
    command_tx: mpsc::Sender<Command>,
    tap: Option<HotkeyTap>,
}

impl HotkeyManager {
    pub fn new(command_tx: mpsc::Sender<Command>) -> Self {
        Self {
            bindings: HashMap::new(),
            command_tx,
            tap: None,
        }
    }

    pub fn bind(&mut self, key_str: &str, command: Command) -> Result<(), String> {
        let hotkey = parse_hotkey(key_str)?;
        tracing::info!("Binding {} to {:?}", key_str, command);
        self.bindings.insert(hotkey, command);

        if self.tap.is_some() {
            self.restart_tap()?;
        }
        Ok(())
    }

    pub fn unbind(&mut self, key_str: &str) -> Result<(), String> {
        let hotkey = parse_hotkey(key_str)?;
        self.bindings.remove(&hotkey);
        tracing::info!("Unbound {}", key_str);

        if self.tap.is_some() {
            self.restart_tap()?;
        }
        Ok(())
    }

    pub fn list_bindings(&self) -> Vec<(String, Command)> {
        self.bindings
            .iter()
            .map(|(hotkey, cmd)| (format_hotkey(hotkey), cmd.clone()))
            .collect()
    }

    pub fn start(&mut self) -> Result<(), String> {
        self.tap = Some(self.create_tap()?);
        tracing::info!("Hotkey tap started with {} bindings", self.bindings.len());
        Ok(())
    }

    fn restart_tap(&mut self) -> Result<(), String> {
        self.tap = Some(self.create_tap()?);
        tracing::info!("Hotkey tap restarted with {} bindings", self.bindings.len());
        Ok(())
    }

    fn create_tap(&self) -> Result<HotkeyTap, String> {
        let bindings = self.bindings.clone();
        let tx = self.command_tx.clone();

        let tap = CGEventTap::new(
            CGEventTapLocation::Session,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            vec![CGEventType::KeyDown],
            move |_proxy, _event_type, event| {
                let key_code =
                    event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u16;
                let flags = event.get_flags();

                let modifiers = Modifiers {
                    cmd: flags.contains(CGEventFlags::CGEventFlagCommand),
                    alt: flags.contains(CGEventFlags::CGEventFlagAlternate),
                    ctrl: flags.contains(CGEventFlags::CGEventFlagControl),
                    shift: flags.contains(CGEventFlags::CGEventFlagShift),
                };

                let hotkey = Hotkey {
                    key_code,
                    modifiers,
                };

                if let Some(command) = bindings.get(&hotkey).cloned() {
                    tracing::debug!("Hotkey matched: {:?} -> {:?}", hotkey, command);
                    if tx.send(command).is_err() {
                        tracing::error!("Failed to send command from hotkey");
                    }
                    return CallbackResult::Drop;
                }

                CallbackResult::Keep
            },
        )
        .map_err(|_| {
            "Failed to create event tap. Make sure Accessibility permission is granted."
        })?;

        tap.enable();

        let source = tap
            .mach_port()
            .create_runloop_source(0)
            .map_err(|_| "Failed to create run loop source")?;

        CFRunLoop::get_current().add_source(&source, unsafe { kCFRunLoopCommonModes });

        Ok(HotkeyTap {
            _tap: tap,
            _source: source,
        })
    }
}

struct HotkeyTap {
    _tap: CGEventTap<'static>,
    _source: CFRunLoopSource,
}

fn parse_key_code(key: &str) -> Result<u16, String> {
    match key.to_lowercase().as_str() {
        // Letters
        "a" => Ok(0x00),
        "b" => Ok(0x0B),
        "c" => Ok(0x08),
        "d" => Ok(0x02),
        "e" => Ok(0x0E),
        "f" => Ok(0x03),
        "g" => Ok(0x05),
        "h" => Ok(0x04),
        "i" => Ok(0x22),
        "j" => Ok(0x26),
        "k" => Ok(0x28),
        "l" => Ok(0x25),
        "m" => Ok(0x2E),
        "n" => Ok(0x2D),
        "o" => Ok(0x1F),
        "p" => Ok(0x23),
        "q" => Ok(0x0C),
        "r" => Ok(0x0F),
        "s" => Ok(0x01),
        "t" => Ok(0x11),
        "u" => Ok(0x20),
        "v" => Ok(0x09),
        "w" => Ok(0x0D),
        "x" => Ok(0x07),
        "y" => Ok(0x10),
        "z" => Ok(0x06),
        // Numbers
        "1" => Ok(0x12),
        "2" => Ok(0x13),
        "3" => Ok(0x14),
        "4" => Ok(0x15),
        "5" => Ok(0x17),
        "6" => Ok(0x16),
        "7" => Ok(0x1A),
        "8" => Ok(0x1C),
        "9" => Ok(0x19),
        "0" => Ok(0x1D),
        // Special keys
        "return" | "enter" => Ok(0x24),
        "tab" => Ok(0x30),
        "space" => Ok(0x31),
        "delete" | "backspace" => Ok(0x33),
        "escape" | "esc" => Ok(0x35),
        "left" => Ok(0x7B),
        "right" => Ok(0x7C),
        "down" => Ok(0x7D),
        "up" => Ok(0x7E),
        "f1" => Ok(0x7A),
        "f2" => Ok(0x78),
        "f3" => Ok(0x63),
        "f4" => Ok(0x76),
        "f5" => Ok(0x60),
        "f6" => Ok(0x61),
        "f7" => Ok(0x62),
        "f8" => Ok(0x64),
        "f9" => Ok(0x65),
        "f10" => Ok(0x6D),
        "f11" => Ok(0x67),
        "f12" => Ok(0x6F),
        // Punctuation
        "minus" => Ok(0x1B),
        "equal" => Ok(0x18),
        "leftbracket" => Ok(0x21),
        "rightbracket" => Ok(0x1E),
        "backslash" => Ok(0x2A),
        "semicolon" => Ok(0x29),
        "quote" => Ok(0x27),
        "comma" => Ok(0x2B),
        "period" => Ok(0x2F),
        "slash" => Ok(0x2C),
        "grave" => Ok(0x32),
        _ => Err(format!("Unknown key: {}", key)),
    }
}

fn key_code_to_str(code: u16) -> &'static str {
    match code {
        0x00 => "a",
        0x0B => "b",
        0x08 => "c",
        0x02 => "d",
        0x0E => "e",
        0x03 => "f",
        0x05 => "g",
        0x04 => "h",
        0x22 => "i",
        0x26 => "j",
        0x28 => "k",
        0x25 => "l",
        0x2E => "m",
        0x2D => "n",
        0x1F => "o",
        0x23 => "p",
        0x0C => "q",
        0x0F => "r",
        0x01 => "s",
        0x11 => "t",
        0x20 => "u",
        0x09 => "v",
        0x0D => "w",
        0x07 => "x",
        0x10 => "y",
        0x06 => "z",
        0x12 => "1",
        0x13 => "2",
        0x14 => "3",
        0x15 => "4",
        0x17 => "5",
        0x16 => "6",
        0x1A => "7",
        0x1C => "8",
        0x19 => "9",
        0x1D => "0",
        0x24 => "return",
        0x30 => "tab",
        0x31 => "space",
        0x33 => "delete",
        0x35 => "escape",
        0x7B => "left",
        0x7C => "right",
        0x7D => "down",
        0x7E => "up",
        0x7A => "f1",
        0x78 => "f2",
        0x63 => "f3",
        0x76 => "f4",
        0x60 => "f5",
        0x61 => "f6",
        0x62 => "f7",
        0x64 => "f8",
        0x65 => "f9",
        0x6D => "f10",
        0x67 => "f11",
        0x6F => "f12",
        0x1B => "minus",
        0x18 => "equal",
        0x21 => "leftbracket",
        0x1E => "rightbracket",
        0x2A => "backslash",
        0x29 => "semicolon",
        0x27 => "quote",
        0x2B => "comma",
        0x2F => "period",
        0x2C => "slash",
        0x32 => "grave",
        _ => "unknown",
    }
}
