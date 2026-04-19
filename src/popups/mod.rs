pub mod new_field_popup;
pub mod new_node_popup;
pub mod new_topic_popup;
pub mod text_popup;

use ratatui::{buffer::Buffer, layout::Rect};

use crate::common::event::Event;

pub trait TuiPopup {
    fn handle_event(&mut self, event: Event) -> Event;
    fn needs_redraw(&mut self) -> bool;
    fn render(&self, area: Rect, buf: &mut Buffer);
}
