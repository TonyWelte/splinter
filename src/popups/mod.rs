pub mod add_hz_popup;
pub mod add_line_popup;

use add_hz_popup::AddHzState;
use add_line_popup::AddLineState;

pub enum PopupView {
    None,
    AddLine(AddLineState),
    AddHz(AddHzState),
}
