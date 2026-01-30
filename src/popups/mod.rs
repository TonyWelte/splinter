pub mod new_node_popup;
pub mod new_topic_popup;
pub mod new_field_popup;
pub mod text_popup;

use new_node_popup::NewNodePopupState;
use new_field_popup::NewFieldPopupState;
use new_topic_popup::NewTopicPopupState;
use text_popup::TextPopup;

pub enum PopupView {
    None,
    Error(TextPopup),
    NewNode(NewNodePopupState),
    NewTopic(NewTopicPopupState),
    NewField(NewFieldPopupState),
}
