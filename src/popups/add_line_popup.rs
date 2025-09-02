use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEventKind};
use ratatui::widgets::{BorderType, Widget};

use crate::{
    common::event::{Event, NewLineEvent},
    widgets::select_view_widget::SelectViewWidget,
};

pub struct AddLineState {
    topic: String,
    field: Vec<usize>,
    views: Vec<(usize, String)>,
    selected: usize,
}

impl AddLineState {
    pub fn new(topic: String, field: Vec<usize>, candidate_views: Vec<(usize, String)>) -> Self {
        Self {
            topic,
            field,
            views: candidate_views,
            selected: 0,
        }
    }

    pub fn handle_event(&mut self, event: Event) -> Event {
        if let Event::Key(CrosstermEvent::Key(key_event)) = event {
            if key_event.kind != KeyEventKind::Press {
                return event;
            }
            match key_event.code {
                KeyCode::Char('k') | KeyCode::Up => {
                    self.selected = self.selected.saturating_sub(1);
                    return Event::None;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    if self.selected < self.views.len() {
                        self.selected = self.selected + 1;
                    }
                    return Event::None;
                }
                KeyCode::Enter => {
                    if self.selected == self.views.len() {
                        return Event::NewLinePlot(NewLineEvent {
                            topic: self.topic.clone(),
                            field: self.field.clone(),
                            view: None,
                        });
                    } else {
                        return Event::NewLine(NewLineEvent {
                            topic: self.topic.clone(),
                            field: self.field.clone(),
                            view: Some(self.views[self.selected].0),
                        });
                    }
                }
                KeyCode::Esc => {
                    return Event::None;
                }
                _ => {}
            }
        }
        return event;
    }
}

impl AddLineState {
    pub fn render(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let select_view_widget = SelectViewWidget::new(&self.views)
            .with_selection(self.selected)
            .with_new_option(true)
            .block(
                ratatui::widgets::Block::default()
                    .title("Select View")
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_type(BorderType::Rounded),
            );
        select_view_widget.render(area, buf);
    }
}
