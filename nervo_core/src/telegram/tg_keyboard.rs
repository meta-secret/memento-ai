use teloxide::types::{KeyboardButton, KeyboardMarkup};

pub struct NervoBotKeyboard {}

impl NervoBotKeyboard {
    pub fn build_keyboard() -> KeyboardMarkup {
        let buttons = [
            KeyboardButton::new("/chat"),
            KeyboardButton::new("/analyse"),
        ];

        KeyboardMarkup::new([buttons]).resize_keyboard(true)
    }
}
