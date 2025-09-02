use crate::common::event::Event;

pub mod message_widget;
pub mod select_view_widget;
pub mod topic_list_widget;

pub trait TuiWidget {
    fn handle_event(&mut self, event: Event) -> Event;
}
