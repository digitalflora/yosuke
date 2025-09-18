use enigo::{Button, Coordinate, Direction, Enigo, Keyboard, Mouse};
use shared::{
    commands::{BaseCommand, Command},
    input::{InputType, MouseInputType},
};

pub fn main(command: &BaseCommand, enigo: &mut Enigo) -> bool {
    match &command.command {
        Command::Input(input_type) => match input_type {
            InputType::MouseDown(input_type) => {
                match input_type {
                    MouseInputType::Left => {
                        let _ = enigo.button(Button::Left, Direction::Press);
                    }
                    MouseInputType::Middle => {
                        let _ = enigo.button(Button::Middle, Direction::Press);
                    }
                    MouseInputType::Right => {
                        let _ = enigo.button(Button::Right, Direction::Press);
                    }
                }
                return true;
            }
            InputType::MouseUp(input_type) => {
                match input_type {
                    MouseInputType::Left => {
                        let _ = enigo.button(Button::Left, Direction::Release);
                    }
                    MouseInputType::Middle => {
                        let _ = enigo.button(Button::Middle, Direction::Release);
                    }
                    MouseInputType::Right => {
                        let _ = enigo.button(Button::Right, Direction::Release);
                    }
                }
                return true;
            }
            InputType::MouseMove((x, y)) => {
                let _ = enigo.move_mouse(x.round() as i32, y.round() as i32, Coordinate::Abs);
                return true;
            }
            InputType::Key(release, key, modifiers) => {
                // println!("[*][input] lets press {}", key);

                let enigo_key = if key.len() != 1 {
                    // Handle egui Key enum variants (as lowercase strings)
                    match key.to_lowercase().as_str() {
                        // Navigation keys
                        "arrowdown" => enigo::Key::DownArrow,
                        "arrowleft" => enigo::Key::LeftArrow,
                        "arrowright" => enigo::Key::RightArrow,
                        "arrowup" => enigo::Key::UpArrow,
                        "escape" => enigo::Key::Escape,
                        "tab" => enigo::Key::Tab,
                        "backspace" => enigo::Key::Backspace,
                        "enter" => enigo::Key::Return,
                        "space" => enigo::Key::Space,
                        "insert" => enigo::Key::Insert,
                        "delete" => enigo::Key::Delete,
                        "home" => enigo::Key::Home,
                        "end" => enigo::Key::End,
                        "pageup" => enigo::Key::PageUp,
                        "pagedown" => enigo::Key::PageDown,

                        // Copy/Cut/Paste (these might need special handling as they're shortcuts)
                        "copy" => enigo::Key::Unicode('c'), // Usually Ctrl+C
                        "cut" => enigo::Key::Unicode('x'),  // Usually Ctrl+X
                        "paste" => enigo::Key::Unicode('v'), // Usually Ctrl+V

                        // Special characters
                        "colon" => enigo::Key::Unicode(':'),
                        "comma" => enigo::Key::Unicode(','),
                        "backslash" => enigo::Key::Unicode('\\'),
                        "slash" => enigo::Key::Unicode('/'),
                        "pipe" => enigo::Key::Unicode('|'),
                        "questionmark" => enigo::Key::Unicode('?'),
                        "exclamationmark" => enigo::Key::Unicode('!'),
                        "openbracket" => enigo::Key::Unicode('['),
                        "closebracket" => enigo::Key::Unicode(']'),
                        "opencurlybracket" => enigo::Key::Unicode('{'),
                        "closecurlybracket" => enigo::Key::Unicode('}'),
                        "backtick" => enigo::Key::Unicode('`'),
                        "minus" => enigo::Key::Unicode('-'),
                        "period" => enigo::Key::Unicode('.'),
                        "plus" => enigo::Key::Unicode('+'),
                        "equals" => enigo::Key::Unicode('='),
                        "semicolon" => enigo::Key::Unicode(';'),
                        "quote" => enigo::Key::Unicode('\''),

                        // Numbers (egui doesn't distinguish between main row and numpad)
                        "num0" => enigo::Key::Unicode('0'),
                        "num1" => enigo::Key::Unicode('1'),
                        "num2" => enigo::Key::Unicode('2'),
                        "num3" => enigo::Key::Unicode('3'),
                        "num4" => enigo::Key::Unicode('4'),
                        "num5" => enigo::Key::Unicode('5'),
                        "num6" => enigo::Key::Unicode('6'),
                        "num7" => enigo::Key::Unicode('7'),
                        "num8" => enigo::Key::Unicode('8'),
                        "num9" => enigo::Key::Unicode('9'),

                        // Letters (single characters, but handling them here for completeness)
                        "a" => enigo::Key::Unicode('a'),
                        "b" => enigo::Key::Unicode('b'),
                        "c" => enigo::Key::Unicode('c'),
                        "d" => enigo::Key::Unicode('d'),
                        "e" => enigo::Key::Unicode('e'),
                        "f" => enigo::Key::Unicode('f'),
                        "g" => enigo::Key::Unicode('g'),
                        "h" => enigo::Key::Unicode('h'),
                        "i" => enigo::Key::Unicode('i'),
                        "j" => enigo::Key::Unicode('j'),
                        "k" => enigo::Key::Unicode('k'),
                        "l" => enigo::Key::Unicode('l'),
                        "m" => enigo::Key::Unicode('m'),
                        "n" => enigo::Key::Unicode('n'),
                        "o" => enigo::Key::Unicode('o'),
                        "p" => enigo::Key::Unicode('p'),
                        "q" => enigo::Key::Unicode('q'),
                        "r" => enigo::Key::Unicode('r'),
                        "s" => enigo::Key::Unicode('s'),
                        "t" => enigo::Key::Unicode('t'),
                        "u" => enigo::Key::Unicode('u'),
                        "v" => enigo::Key::Unicode('v'),
                        "w" => enigo::Key::Unicode('w'),
                        "x" => enigo::Key::Unicode('x'),
                        "y" => enigo::Key::Unicode('y'),
                        "z" => enigo::Key::Unicode('z'),

                        // Function keys
                        "f1" => enigo::Key::F1,
                        "f2" => enigo::Key::F2,
                        "f3" => enigo::Key::F3,
                        "f4" => enigo::Key::F4,
                        "f5" => enigo::Key::F5,
                        "f6" => enigo::Key::F6,
                        "f7" => enigo::Key::F7,
                        "f8" => enigo::Key::F8,
                        "f9" => enigo::Key::F9,
                        "f10" => enigo::Key::F10,
                        "f11" => enigo::Key::F11,
                        "f12" => enigo::Key::F12,
                        "f13" => enigo::Key::F13,
                        "f14" => enigo::Key::F14,
                        "f15" => enigo::Key::F15,
                        "f16" => enigo::Key::F16,
                        "f17" => enigo::Key::F17,
                        "f18" => enigo::Key::F18,
                        "f19" => enigo::Key::F19,
                        "f20" => enigo::Key::F20,
                        "f21" => enigo::Key::F21,
                        "f22" => enigo::Key::F22,
                        "f23" => enigo::Key::F23,
                        "f24" => enigo::Key::F24,

                        // If no match found, try to get first character
                        _ => {
                            if let Some(ch) = key.chars().next() {
                                enigo::Key::Unicode(ch.to_lowercase().next().unwrap_or(ch))
                            } else {
                                println!("[!][input] Unknown key: {}", key);
                                return false;
                            }
                        }
                    }
                } else {
                    // Single character key
                    enigo::Key::Unicode(key.as_str().to_lowercase().chars().next().unwrap())
                };

                // println!("[*][input] we are going to actually press {:?}", enigo_key);
                let mut direction = Direction::Press;
                if *release {
                    direction = Direction::Release;
                    // println!("[*][input] we'll be releasing it");
                };

                // Handle modifier key combinations
                // Press down modifiers first
                if modifiers.ctrl {
                    let _ = enigo.key(enigo::Key::Control, Direction::Press);
                }
                if modifiers.alt {
                    let _ = enigo.key(enigo::Key::Alt, Direction::Press);
                }
                if modifiers.shift {
                    let _ = enigo.key(enigo::Key::Shift, Direction::Press);
                }

                // Press the main key
                match enigo.key(enigo_key, direction) {
                    Ok(_) => {
                        if modifiers.ctrl || modifiers.alt || modifiers.shift {
                            // println!("[*][input] key combo should be pressed");
                        } else {
                            //println!("[*][input] key should be pressed");
                        }
                    }
                    Err(e) => {
                        println!("[x][input] {}", e);
                    }
                };

                // Release modifiers in reverse order
                if modifiers.shift {
                    let _ = enigo.key(enigo::Key::Shift, Direction::Release);
                }
                if modifiers.alt {
                    let _ = enigo.key(enigo::Key::Alt, Direction::Release);
                }
                if modifiers.ctrl {
                    let _ = enigo.key(enigo::Key::Control, Direction::Release);
                }
                return true;
            }
            _ => false,
        },
        _ => false, // fall through
    }
}
