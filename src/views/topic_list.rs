use std::{cell::RefCell, rc::Rc};

use ratatui::{
    prelude::{Buffer, Rect},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Widget},
};

use crossterm::event::{Event, KeyCode};

use crate::{
    common::style::{HEADER_STYLE, SELECTED_STYLE},
    connections::{Connection, ConnectionType},
    views::{raw_message::RawMessageState, topic_publisher::TopicPublisherState, TuiView, Views},
};

// TODO(@TonyWelte): Remove dependency on rclrs in widgets module
use rclrs::MessageTypeName;

pub struct TopicList;

pub struct TopicListState {
    connection: Rc<RefCell<ConnectionType>>,
    pub topics: Vec<(String, MessageTypeName)>,
    pub selected_index: usize,
}

impl TopicListState {
    pub fn new(connection: Rc<RefCell<ConnectionType>>) -> Self {
        let topics = connection.borrow().list_topics();
        Self {
            connection,
            topics,
            selected_index: 0,
        }
    }

    pub fn next_topic(&mut self) {
        if !self.topics.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.topics.len();
        }
    }

    pub fn previous_topic(&mut self) {
        if !self.topics.is_empty() {
            self.selected_index = (self.selected_index + self.topics.len() - 1) % self.topics.len();
        }
    }

    pub fn update(&mut self) {
        let mut new_topics = self.connection.borrow().list_topics();
        new_topics.sort_by(|a, b| a.0.cmp(&b.0));
        if self.topics.is_empty() {
            self.topics = new_topics;
            self.selected_index = 0;
        } else if new_topics != self.topics {
            let selected_topic = self.topics.get(self.selected_index).unwrap().0.clone();
            let new_index = new_topics
                .iter()
                .position(|topic| topic.0 == selected_topic)
                .unwrap_or(0);
            self.topics = new_topics;
            self.selected_index = new_index;
        }
    }
}

impl TuiView for TopicListState {
    fn handle_event(&mut self, event: Event) -> Option<Views> {
        match event {
            Event::Key(key) => {
                match key.code {
                    KeyCode::Char('j') | KeyCode::Down => self.next_topic(),
                    KeyCode::Char('k') | KeyCode::Up => self.previous_topic(),
                    KeyCode::Char('l') | KeyCode::Right => {
                        // if let Some(topic) = self.topics.get(self.selected_index) {
                        //     let topic_publisher_state =
                        //         TopicPublisherState::new(topic.0.clone(), self.connection.clone());
                        //     return Some(Views::TopicPublisher(topic_publisher_state));
                        // }
                        if let Some(topic) = self.topics.get(self.selected_index) {
                            let raw_message_state =
                                RawMessageState::new(topic.0.clone(), self.connection.clone());
                            return Some(Views::RawMessage(raw_message_state));
                        }
                    }
                    _ => {}
                }
                None
            }
            _ => None,
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

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = state
            .topics
            .iter()
            .enumerate()
            .map(|(i, topic_item)| {
                if i == state.selected_index {
                    ListItem::new(topic_item.0.as_str()).style(SELECTED_STYLE)
                } else {
                    let available_width = area.width as usize;
                    let remaining_space = available_width.saturating_sub(
                        topic_item.0.len()
                            + topic_item.1.package_name.len()
                            + "/msg/".len()
                            + topic_item.1.type_name.len(),
                    );

                    let text = Span::raw(format!(
                        "{}{:>remaining_space$}{}/msg/{}",
                        topic_item.0, "", topic_item.1.package_name, topic_item.1.type_name
                    ));
                    ListItem::new(text)
                }
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items).block(block);

        Widget::render(list, area, buf);
    }
}
