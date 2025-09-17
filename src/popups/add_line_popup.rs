use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEventKind};
use ratatui::widgets::{BorderType, Widget};

use crate::{
    common::event::{Event, NewLineEvent},
    widgets::select_view_widget::SelectViewWidget,
};

pub struct AddLineState {
    topic: String,
    field: Vec<usize>,
    field_name: String,
    views: Vec<(usize, String)>,
    selected: usize,

    needs_redraw: bool,
}

impl AddLineState {
    pub fn new(
        topic: String,
        field: Vec<usize>,
        field_name: String,
        candidate_views: Vec<(usize, String)>,
    ) -> Self {
        Self {
            topic,
            field,
            field_name,
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
                        self.selected += 1;
                    }
                    self.needs_redraw = true;
                    return Event::None;
                }
                KeyCode::Enter => {
                    if self.selected == self.views.len() {
                        return Event::NewLinePlot(NewLineEvent {
                            topic: self.topic.clone(),
                            field: self.field.clone(),
                            field_name: self.field_name.clone(),
                            view: None,
                        });
                    } else {
                        return Event::NewLine(NewLineEvent {
                            topic: self.topic.clone(),
                            field: self.field.clone(),
                            field_name: self.field_name.clone(),
                            view: Some(self.views[self.selected].0),
                        });
                    }
                }
                KeyCode::Esc => {
                    return Event::ClosePopup;
                }
                _ => {}
            }
        }
        event
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
