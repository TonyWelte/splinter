use std::{cell::RefCell, rc::Rc};
use indexmap::IndexMap;

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEventKind};
use ratatui::widgets::{Block, BorderType, Widget};

use crate::{
    common::event::Event,
    views::{
        hz_plot::HzPlotState, raw_message::RawMessageState, topic_publisher::TopicPublisherState,
        FromTopic, TopicInfo, TuiView,
    },
    widgets::select_view_widget::SelectViewWidget,
};

// TODO: Make this configurable via plugins
type NewTopicFactoryClosure = dyn Fn(TopicInfo) -> Rc<RefCell<dyn TuiView>> + Send + Sync;

static FROM_NEW_TOPIC_FACTORIES: once_cell::sync::Lazy<
    IndexMap<&'static str, Box<NewTopicFactoryClosure>>,
> = once_cell::sync::Lazy::new(|| {
    let mut m = IndexMap::new();
    m.insert(
        "raw_message",
        Box::new(|topic_info: TopicInfo| {
            Rc::new(RefCell::new(RawMessageState::from_topic(topic_info)))
                as Rc<RefCell<dyn TuiView>>
        }) as Box<NewTopicFactoryClosure>,
    );
    m.insert(
        "topic_publisher",
        Box::new(|topic_info: TopicInfo| {
            Rc::new(RefCell::new(TopicPublisherState::from_topic(topic_info)))
                as Rc<RefCell<dyn TuiView>>
        }) as Box<NewTopicFactoryClosure>,
    );
    m.insert(
        "hz_plot",
        Box::new(|topic_info: TopicInfo| {
            Rc::new(RefCell::new(HzPlotState::from_topic(topic_info))) as Rc<RefCell<dyn TuiView>>
        }) as Box<NewTopicFactoryClosure>,
    );
    m
});

pub struct NewTopicPopupState {
    topic: TopicInfo,
    views: Vec<Rc<RefCell<dyn TuiView>>>,
    selected: usize,

    needs_redraw: bool,
}

impl NewTopicPopupState {
    pub fn new(topic: TopicInfo, candidate_views: Vec<Rc<RefCell<dyn TuiView>>>) -> Self {
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
                    if self.selected < self.views.len() + FROM_NEW_TOPIC_FACTORIES.len() - 1 {
                        self.selected += 1;
                    }
                    self.needs_redraw = true;
                    return Event::None;
                }
                KeyCode::Enter => {
                    if self.selected < FROM_NEW_TOPIC_FACTORIES.len() {
                        let factory_index = self.selected;
                        let factory_key = FROM_NEW_TOPIC_FACTORIES
                            .keys()
                            .nth(factory_index)
                            .expect("Factory index out of bounds");
                        let factory = FROM_NEW_TOPIC_FACTORIES
                            .get(factory_key)
                            .expect("Factory key not found");
                        let new_view = factory(self.topic.clone());
                        return Event::NewView(new_view);
                    } else {
                        let mut view =
                            self.views[self.selected - FROM_NEW_TOPIC_FACTORIES.len()].borrow_mut();
                        if let Some(accepts_topic) = view.as_topic_acceptor() {
                            accepts_topic.accepts_topic(self.topic.clone());
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

impl NewTopicPopupState {
    pub fn render(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let mut views: Vec<(usize, String)> = FROM_NEW_TOPIC_FACTORIES
            .keys()
            .enumerate()
            .map(|(i, k)| (i, format!("New {}", k)))
            .collect();
        views.extend(self.views.iter().enumerate().map(|(i, v)| {
            (
                i + FROM_NEW_TOPIC_FACTORIES.len(),
                format!("Add to existing {}", v.borrow().name()),
            )
        }));
        let select_view_widget = SelectViewWidget::new(&views)
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
