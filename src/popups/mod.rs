pub mod add_hz_popup;
pub mod add_line_popup;
pub mod text_popup;

use add_hz_popup::AddHzState;
use add_line_popup::AddLineState;
use text_popup::TextPopup;

pub enum PopupView {
    None,
    AddLine(AddLineState),
    AddHz(AddHzState),
    Error(TextPopup),
}
