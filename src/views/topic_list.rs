use std::{cell::RefCell, rc::Rc};

use ratatui::{
    layout::{Constraint, Layout},
    prelude::{Buffer, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, List, ListItem, StatefulWidget, Widget},
};

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEventKind};

use crate::{
    common::{
        event::{Event, NewHzEvent, NewPublisherEvent, NewTopicEvent},
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

enum Action {
    Echo,
    Pub,
    FrequencyPlot,
}

impl Action {
    pub fn next(&self) -> Self {
        match self {
            Action::Echo => Action::Pub,
            Action::Pub => Action::FrequencyPlot,
            Action::FrequencyPlot => Action::FrequencyPlot,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Action::Echo => Action::Echo,
            Action::Pub => Action::Echo,
            Action::FrequencyPlot => Action::Pub,
        }
    }
}

pub struct TopicListState {
    connection: Rc<RefCell<ConnectionType>>,
    state: TopicListWidgetState,
    action: Action,
}

impl TopicListState {
    pub fn new(connection: Rc<RefCell<ConnectionType>>) -> Self {
        let topics = connection.borrow().list_topics();
        Self {
            connection,
            state: TopicListWidgetState::new(topics, 0),
            action: Action::Echo,
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
                    self.action = self.action.next();
                    Event::None
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    self.action = self.action.previous();
                    Event::None
                }
                KeyCode::Enter => {
                    if let Some((topic, type_name)) =
                        self.state.topics.get(self.state.selected_index)
                    {
                        match self.action {
                            Action::Echo => Event::NewMessageView(NewTopicEvent {
                                topic: topic.clone(),
                                message_type: type_name.clone(),
                            }),
                            Action::Pub => Event::NewPublisher(NewPublisherEvent {
                                topic: topic.clone(),
                                message_type: type_name.clone(),
                            }),
                            Action::FrequencyPlot => Event::NewHz(NewHzEvent {
                                topic: topic.clone(),
                                view: None,
                            }),
                        }
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

    fn get_help_text(&self) -> String {
        "Topic List View Help:\n\
        - 'j' or ↓: Move down in the topic list.\n\
        - 'k' or ↑: Move up in the topic list.\n\
        - 'l' or →: Switch to the next action (Echo, Pub, Hz).\n\
        - 'h' or ←: Switch to the previous action (Echo, Pub, Hz).\n\
        - 'Enter': Execute the selected action on the highlighted topic."
            .to_string()
    }
}

impl TopicList {
    pub fn render(area: Rect, buf: &mut Buffer, state: &mut TopicListState) {
        state.update();

        let action_text = Line::from_iter([
            Span::raw(" Topic List - "),
            Span::styled(
                " Echo ",
                if matches!(state.action, Action::Echo) {
                    SELECTED_STYLE
                } else {
                    Style::default()
                },
            ),
            Span::styled(
                " Pub ",
                if matches!(state.action, Action::Pub) {
                    SELECTED_STYLE
                } else {
                    Style::default()
                },
            ),
            Span::styled(
                " Hz ",
                if matches!(state.action, Action::FrequencyPlot) {
                    SELECTED_STYLE
                } else {
                    Style::default()
                },
            ),
        ])
        .centered();

        let block = Block::bordered()
            .title(action_text)
            .border_style(HEADER_STYLE)
            .border_type(BorderType::Rounded);

        let topic_list_widget = TopicListWidget::new().block(block);

        topic_list_widget.render(area, buf, &mut state.state);
    }
}
