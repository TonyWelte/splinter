use std::{cell::RefCell, rc::Rc};

use ratatui::{
    prelude::{Buffer, Rect},
    text::{Line, Span},
    widgets::{Block, List, ListItem, StatefulWidget, Widget},
};

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEventKind};

use crate::{
    common::{
        event::{Event, NewTopicEvent},
        style::{HEADER_STYLE, SELECTED_STYLE},
    },
    connections::{Connection, ConnectionType},
    views::TuiView,
    widgets::{
        topic_list_widget::{TopicListWidget, TopicListWidgetState},
        TuiWidget,
    },
};

// TODO(@TonyWelte): Remove dependency on rclrs in widgets module
use rclrs::MessageTypeName;

pub struct TopicList;

pub struct TopicListState {
    connection: Rc<RefCell<ConnectionType>>,
    state: TopicListWidgetState,
}

impl TopicListState {
    pub fn new(connection: Rc<RefCell<ConnectionType>>) -> Self {
        let topics = connection.borrow().list_topics();
        Self {
            connection,
            state: TopicListWidgetState {
                topics,
                selected_index: 0,
            },
        }
    }

    pub fn update(&mut self) {
        let mut new_topics = self.connection.borrow().list_topics();
        new_topics.sort_by(|a, b| a.0.cmp(&b.0));
        self.state.update(new_topics);
    }
}

impl TuiView for TopicListState {
    fn handle_event(&mut self, event: Event) -> Event {
        let event = self.state.handle_event(event);
        if let Event::Key(CrosstermEvent::Key(key_event)) = event {
            if key_event.kind != KeyEventKind::Press {
                return event;
            }
            match key_event.code {
                KeyCode::Char('l') | KeyCode::Right => {
                    if let Some((topic, type_name)) =
                        self.state.topics.get(self.state.selected_index)
                    {
                        Event::NewHzPlot(NewTopicEvent {
                            topic: topic.clone(),
                            message_type: type_name.clone(),
                        })
                    } else {
                        event
                    }
                }
                _ => event,
            }
        } else {
            event
        }
    }

    fn name(&self) -> String {
        "Topics".to_string()
    }
}

impl TopicList {
    pub fn render(area: Rect, buf: &mut Buffer, state: &mut TopicListState) {
        state.update();

        let block = Block::bordered()
            .title(Line::raw("Topic List").centered())
            .border_style(HEADER_STYLE);

        let topic_list_widget = TopicListWidget::new().block(block);

        topic_list_widget.render(area, buf, &mut state.state);
    }
}
