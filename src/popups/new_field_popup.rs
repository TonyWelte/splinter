use indexmap::IndexMap;
use std::{cell::RefCell, rc::Rc};

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEventKind};
use ratatui::widgets::{Block, BorderType, Widget};

use ratatui::{buffer::Buffer, layout::Rect};

use crate::{
    common::event::Event,
    popups::{text_popup::TextPopup, TuiPopup},
    views::{
        live_plot::LivePlotState, state_graph::StateGraphViewState, FieldInfo, FieldInfoType,
        FromField, TuiView,
    },
    widgets::select_view_widget::SelectViewWidget,
};

// TODO: Make this configurable via plugins
type NewFieldFactoryPredicate = dyn Fn(&FieldInfoType) -> bool + Send + Sync;
type NewFieldFactoryClosure = dyn Fn(FieldInfo) -> Rc<RefCell<dyn TuiView>> + Send + Sync;

/// Each entry is a (type_predicate, factory) pair.
/// The predicate determines which field types this factory supports.
static FROM_NEW_FIELD_FACTORIES: once_cell::sync::Lazy<
    IndexMap<&'static str, (Box<NewFieldFactoryPredicate>, Box<NewFieldFactoryClosure>)>,
> = once_cell::sync::Lazy::new(|| {
    let mut m = IndexMap::new();
    m.insert(
        "plot",
        (
            Box::new(|ft: &FieldInfoType| ft.is_numeric()) as Box<NewFieldFactoryPredicate>,
            Box::new(|field_info: FieldInfo| {
                Rc::new(RefCell::new(LivePlotState::from_field(field_info)))
                    as Rc<RefCell<dyn TuiView>>
            }) as Box<NewFieldFactoryClosure>,
        ),
    );
    m.insert(
        "state graph",
        (
            Box::new(|ft: &FieldInfoType| {
                matches!(ft, FieldInfoType::Integer | FieldInfoType::String)
            }) as Box<NewFieldFactoryPredicate>,
            Box::new(|field_info: FieldInfo| {
                Rc::new(RefCell::new(StateGraphViewState::from_field(field_info)))
                    as Rc<RefCell<dyn TuiView>>
            }) as Box<NewFieldFactoryClosure>,
        ),
    );
    m
});

pub struct NewFieldPopupState {
    field: FieldInfo,
    /// Factory keys from FROM_NEW_FIELD_FACTORIES that are compatible with the field's type.
    applicable_factory_keys: Vec<&'static str>,
    views: Vec<Rc<RefCell<dyn TuiView>>>,
    selected: usize,

    needs_redraw: bool,
}

impl NewFieldPopupState {
    pub fn new(
        field: FieldInfo,
        candidate_views: Vec<Rc<RefCell<dyn TuiView>>>,
    ) -> Box<dyn TuiPopup> {
        let applicable_factory_keys = FROM_NEW_FIELD_FACTORIES
            .iter()
            .filter(|(_, (predicate, _))| predicate(&field.field_type))
            .map(|(key, _)| *key)
            .collect::<Vec<_>>();
        if applicable_factory_keys.is_empty() && candidate_views.is_empty() {
            return Box::new(TextPopup::error(format!(
                "No views available for field '{}' of type {}.",
                field.field_name, field.field_type
            )));
        }
        Box::new(Self {
            field,
            applicable_factory_keys,
            views: candidate_views,
            selected: 0,
            needs_redraw: true,
        })
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
                    let total = self.applicable_factory_keys.len() + self.views.len();
                    if self.selected + 1 < total {
                        self.selected += 1;
                    }
                    self.needs_redraw = true;
                    return Event::None;
                }
                KeyCode::Enter => {
                    if self.selected < self.applicable_factory_keys.len() {
                        let key = self.applicable_factory_keys[self.selected];
                        let (_, factory) = FROM_NEW_FIELD_FACTORIES
                            .get(key)
                            .expect("Applicable factory key not found in global map");
                        let new_view = factory(self.field.clone());
                        return Event::NewView(new_view);
                    } else {
                        let view_index = self.selected - self.applicable_factory_keys.len();
                        let mut view = self.views[view_index].borrow_mut();
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

impl TuiPopup for NewFieldPopupState {
    fn handle_event(&mut self, event: Event) -> Event {
        NewFieldPopupState::handle_event(self, event)
    }

    fn needs_redraw(&mut self) -> bool {
        NewFieldPopupState::needs_redraw(self)
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        NewFieldPopupState::render(self, area, buf);
    }
}

impl NewFieldPopupState {
    pub fn render(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let mut views: Vec<(usize, String)> = self
            .applicable_factory_keys
            .iter()
            .enumerate()
            .map(|(i, k)| (i, format!("New {}", k)))
            .collect();
        views.extend(self.views.iter().enumerate().map(|(i, v)| {
            (
                i + self.applicable_factory_keys.len(),
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
