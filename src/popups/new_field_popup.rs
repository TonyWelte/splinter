use std::{cell::RefCell, rc::Rc};
use indexmap::IndexMap;

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEventKind};
use ratatui::widgets::{Block, BorderType, Widget};

use crate::{
    common::event::Event,
    views::{
        FieldInfo, FromField, TuiView, live_plot::LivePlotState
    },
    widgets::select_view_widget::SelectViewWidget,
};

// TODO: Make this configurable via plugins
type NewFieldFactoryClosure = dyn Fn(FieldInfo) -> Rc<RefCell<dyn TuiView>> + Send + Sync;

static FROM_NEW_FIELD_FACTORIES: once_cell::sync::Lazy<
    IndexMap<&'static str, Box<NewFieldFactoryClosure>>,
> = once_cell::sync::Lazy::new(|| {
    let mut m = IndexMap::new();
    m.insert(
        "plot",
        Box::new(|field_info: FieldInfo| {
            Rc::new(RefCell::new(LivePlotState::from_field(field_info)))
                as Rc<RefCell<dyn TuiView>>
        }) as Box<NewFieldFactoryClosure>,
    );
    m
});

pub struct NewFieldPopupState {
    field: FieldInfo,
    views: Vec<Rc<RefCell<dyn TuiView>>>,
    selected: usize,

    needs_redraw: bool,
}

impl NewFieldPopupState {
    pub fn new(field: FieldInfo, candidate_views: Vec<Rc<RefCell<dyn TuiView>>>) -> Self {
        Self {
            field,
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
                    if self.selected < self.views.len() + FROM_NEW_FIELD_FACTORIES.len() - 1 {
                        self.selected += 1;
                    }
                    self.needs_redraw = true;
                    return Event::None;
                }
                KeyCode::Enter => {
                    if self.selected < FROM_NEW_FIELD_FACTORIES.len() {
                        let factory_index = self.selected;
                        let factory_key = FROM_NEW_FIELD_FACTORIES
                            .keys()
                            .nth(factory_index)
                            .expect("Factory index out of bounds");
                        let factory = FROM_NEW_FIELD_FACTORIES
                            .get(factory_key)
                            .expect("Factory key not found");
                        let new_view = factory(self.field.clone());
                        return Event::NewView(new_view);
                    } else {
                        let mut view =
                            self.views[self.selected - FROM_NEW_FIELD_FACTORIES.len()].borrow_mut();
                        if let Some(accepts_field) = view.as_field_acceptor() {
                            accepts_field.accepts_field(self.field.clone());
                        }
                        return Event::ClosePopup;
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

impl NewFieldPopupState {
    pub fn render(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let mut views: Vec<(usize, String)> = FROM_NEW_FIELD_FACTORIES
            .keys()
            .enumerate()
            .map(|(i, k)| (i, format!("New {}", k)))
            .collect();
        views.extend(self.views.iter().enumerate().map(|(i, v)| {
            (
                i + FROM_NEW_FIELD_FACTORIES.len(),
                format!("Add to existing {}", v.borrow().name()),
            )
        }));
        let select_view_widget = SelectViewWidget::new(&views)
            .with_selection(self.selected)
            .with_new_option(true)
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .title("Select Field View")
                    .borders(ratatui::widgets::Borders::ALL),
            );
        select_view_widget.render(area, buf);
    }
}
