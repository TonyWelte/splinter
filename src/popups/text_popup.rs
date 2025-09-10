use crossterm::event::{Event as CrosstermEvent, KeyEventKind};
use ratatui::widgets::{BorderType, Clear, Paragraph, Widget, Wrap};

use crate::common::{
        event::Event,
        style::HEADER_STYLE,
    };

pub struct TextPopup {
    title: String,
    message: String,
}

impl TextPopup {
    pub fn error(message: String) -> Self {
        Self {
            title: "Error".to_string(),
            message,
        }
    }

    pub fn info(message: String) -> Self {
        Self {
            title: "Info".to_string(),
            message,
        }
    }

    pub fn handle_event(&mut self, event: Event) -> Event {
        if let Event::Key(CrosstermEvent::Key(key_event)) = event {
            if key_event.kind != KeyEventKind::Press {
                return event;
            }
            return Event::ClosePopup;
        }
        return event;
    }
}

impl TextPopup {
    pub fn render(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        Clear.render(area, buf);
        let error_popup_widget = Paragraph::new(self.message.clone())
            .wrap(Wrap { trim: true })
            .block(
                ratatui::widgets::Block::default()
                    .title(self.title.clone())
                    .border_style(HEADER_STYLE)
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_type(BorderType::Rounded),
            );
        error_popup_widget.render(area, buf);
    }
}
