use bincode::{Decode, Encode};

#[derive(Encode, Decode, Clone)]
pub enum MouseInputType {
    Left,
    Right,
    Middle,
}
#[derive(Encode, Decode, Clone)]
pub enum InputType {
    MouseDown(MouseInputType),
    MouseUp(MouseInputType),

    ScrollUp,
    ScrollDown,

    MouseMove((f32, f32)), //x, y

    Key(bool, String, ModifierKeys),
}

#[derive(Encode, Decode, Clone)]
pub struct ModifierKeys {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}
