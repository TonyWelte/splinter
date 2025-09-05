use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEventKind};
use ratatui::widgets::{BorderType, Paragraph, Widget, Wrap};

use crate::{
    common::event::{Event, NewLineEvent},
    widgets::select_view_widget::SelectViewWidget,
};

pub struct ErrorPopup {
    message: String,
}

impl ErrorPopup {
    pub fn new(message: String) -> Self {
        Self { message }
    }

    pub fn handle_event(&mut self, event: Event) -> Event {
        if let Event::Key(CrosstermEvent::Key(key_event)) = event {
            if key_event.kind != KeyEventKind::Press {
                return event;
            }
            match key_event.code {
                KeyCode::Enter | KeyCode::Esc => return Event::ClosePopup,
                _ => {}
            }
        }
        return event;
    }
}

impl ErrorPopup {
    pub fn render(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let error_popup_widget = Paragraph::new(self.message.clone())
            .wrap(Wrap { trim: true })
            .block(
                ratatui::widgets::Block::default()
                    .title("Error")
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_type(BorderType::Rounded),
            );
        error_popup_widget.render(area, buf);
    }
}
