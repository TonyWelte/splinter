use std::{cell::RefCell, rc::Rc};

use indexmap::IndexMap;

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEventKind};
use ratatui::widgets::{Block, BorderType, Widget};

use ratatui::{buffer::Buffer, layout::Rect};

use crate::{
    common::event::Event,
    popups::TuiPopup,
    views::{node_details::NodeDetailState, FromNode, NodeInfo, TuiView},
    widgets::select_view_widget::SelectViewWidget,
};

// TODO: Make this configurable via plugins
type NewNodeFactoryClosure = dyn Fn(NodeInfo) -> Rc<RefCell<dyn TuiView>> + Send + Sync;

static FROM_NEW_NODE_FACTORIES: once_cell::sync::Lazy<
    IndexMap<&'static str, Box<NewNodeFactoryClosure>>,
> = once_cell::sync::Lazy::new(|| {
    let mut m = IndexMap::new();
    m.insert(
        "node_details",
        Box::new(|node_info: NodeInfo| {
            Rc::new(RefCell::new(NodeDetailState::from_node(node_info)))
                as Rc<RefCell<dyn TuiView>>
        }) as Box<NewNodeFactoryClosure>,
    );
    m
});

pub struct NewNodePopupState {
    node: NodeInfo,
    views: Vec<Rc<RefCell<dyn TuiView>>>,
    selected: usize,

    needs_redraw: bool,
}

impl NewNodePopupState {
    pub fn new(node: NodeInfo, candidate_views: Vec<Rc<RefCell<dyn TuiView>>>) -> Self {
        Self {
            node,
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
                    if self.selected < self.views.len() + FROM_NEW_NODE_FACTORIES.len() - 1 {
                        self.selected += 1;
                    }
                    self.needs_redraw = true;
                    return Event::None;
                }
                KeyCode::Enter => {
                    if self.selected < FROM_NEW_NODE_FACTORIES.len() {
                        let factory_index = self.selected;
                        let factory_key = FROM_NEW_NODE_FACTORIES
                            .keys()
                            .nth(factory_index)
                            .expect("Factory index out of bounds");
                        let factory = FROM_NEW_NODE_FACTORIES
                            .get(factory_key)
                            .expect("Factory key not found");
                        let new_view = factory(self.node.clone());
                        return Event::NewView(new_view);
                    } else {
                        let mut view =
                            self.views[self.selected - FROM_NEW_NODE_FACTORIES.len()].borrow_mut();
                        if let Some(accepts_node) = view.as_node_acceptor() {
                            accepts_node.accepts_node(self.node.clone());
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

impl TuiPopup for NewNodePopupState {
    fn handle_event(&mut self, event: Event) -> Event {
        NewNodePopupState::handle_event(self, event)
    }

    fn needs_redraw(&mut self) -> bool {
        NewNodePopupState::needs_redraw(self)
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        NewNodePopupState::render(self, area, buf);
    }
}

impl NewNodePopupState {
    pub fn render(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let mut views: Vec<(usize, String)> = FROM_NEW_NODE_FACTORIES
            .keys()
            .enumerate()
            .map(|(i, k)| (i, format!("New {}", k)))
            .collect();
        views.extend(self.views.iter().enumerate().map(|(i, v)| {
            (
                i + FROM_NEW_NODE_FACTORIES.len(),
                format!("Add to existing {}", v.borrow().name()),
            )
        }));
        let select_view_widget = SelectViewWidget::new(&views)
            .with_selection(self.selected)
            .with_new_option(true)
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .title("Select Node View")
                    .borders(ratatui::widgets::Borders::ALL),
            );
        select_view_widget.render(area, buf);
    }
}
