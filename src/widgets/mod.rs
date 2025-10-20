use crate::common::event::Event;

pub mod edit_value_widget;
pub mod list_widget;
pub mod message_widget;
pub mod parameter_list_widget;
pub mod select_view_widget;
pub mod topic_list_widget;

pub trait TuiWidget {
    fn handle_event(&mut self, event: Event) -> Event;
}
