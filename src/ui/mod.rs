mod flex_box;
pub use flex_box::{Alignment, FlexBox, FlexDirection, FlexItem, FlexOrigin};

mod text;
pub use {text::FontSize, text::Text};

mod img;
pub use img::Image;

mod flex_button_line;
pub use flex_button_line::button_styles;
pub use {
    flex_button_line::Button, flex_button_line::ButtonStyle, flex_button_line::FlexButtonLine,
    flex_button_line::FlexButtonLineManager, flex_button_line::FlexCharacterGuiLine,
    flex_button_line::FlexCharacterGuiLineManager, flex_button_line::FlexProgressBarLine,
};

mod progress_bar;
pub use progress_bar::ProgressBar;

pub struct Padding {
    pub up: u8,
    pub down: u8,
    pub left: u8,
    pub right: u8,
}
impl Default for Padding {
    fn default() -> Self {
        Self {
            up: 0,
            down: 0,
            left: 0,
            right: 0,
        }
    }
}
