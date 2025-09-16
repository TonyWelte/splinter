use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEventKind};
use ratatui::widgets::{Block, BorderType, Widget};

use crate::{
    common::event::{Event, NewHzEvent},
    widgets::select_view_widget::SelectViewWidget,
};

pub struct AddHzState {
    topic: String,
    views: Vec<(usize, String)>,
    selected: usize,

    needs_redraw: bool,
}

impl AddHzState {
    pub fn new(topic: String, candidate_views: Vec<(usize, String)>) -> Self {
        Self {
            topic,
            views: candidate_views,
            selected: 0,
            needs_redraw: true,
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
                    self.needs_redraw = true;
                    return Event::None;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    if self.selected < self.views.len() {
                        self.selected = self.selected + 1;
                    }
                    self.needs_redraw = true;
                    return Event::None;
                }
                KeyCode::Enter => {
                    if self.selected == self.views.len() {
                        return Event::NewHzPlot(NewHzEvent {
                            topic: self.topic.clone(),
                            view: None,
                        });
                    } else {
                        return Event::NewHz(NewHzEvent {
                            topic: self.topic.clone(),
                            view: Some(self.views[self.selected].0),
                        });
                    }
                    self.needs_redraw = true;
                }
                KeyCode::Esc => {
                    return Event::ClosePopup;
                }
                _ => {}
            }
        }
        return event;
    }

    pub fn needs_redraw(&mut self) -> bool {
        if self.needs_redraw {
            self.needs_redraw = false;
            true
        } else {
            false
        }
    }
}

impl AddHzState {
    pub fn render(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let select_view_widget = SelectViewWidget::new(&self.views)
            .with_selection(self.selected)
            .with_new_option(true)
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .title("Select View")
                    .borders(ratatui::widgets::Borders::ALL),
            );
        select_view_widget.render(area, buf);
    }
}
