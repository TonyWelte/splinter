pub mod add_hz_popup;
pub mod add_line_popup;
pub mod error_popup;

use add_hz_popup::AddHzState;
use add_line_popup::AddLineState;
use error_popup::ErrorPopup;

pub enum PopupView {
    None,
    AddLine(AddLineState),
    AddHz(AddHzState),
    Error(ErrorPopup),
}
