use std::{cell::RefCell, rc::Rc, sync::Arc};

use ratatui::{
    prelude::{Buffer, Rect},
    text::Line,
    widgets::{Block, StatefulWidget, Widget},
};
use rclrs::*;

use crate::{
    common::style::HEADER_STYLE,
    connections::{Connection, ConnectionType},
    // generic_message::{GenericField, GenericMessage},
    views::{
        message_editor::{MessageEditorState, MessageEditorWidget},
        TuiView, Views,
    },
};

use crossterm::event::{Event, KeyCode, KeyEventKind};

pub struct TopicPublisherWidget;

pub struct TopicPublisherState {
    pub topic: String,
    connection: Rc<RefCell<ConnectionType>>,
    publisher: Arc<DynamicPublisherState>,
    message_editor: MessageEditorState,
    message_type: MessageTypeName,
}

impl TopicPublisherState {
    pub fn new(topic: String, connection: Rc<RefCell<ConnectionType>>) -> Self {
        let message_type = MessageTypeName {
            package_name: "nav_msgs".to_string(),
            type_name: "Odometry".to_string(),
        };
        let message = DynamicMessage::new(message_type.clone()).expect("Failed to create message");
        let message_editor = MessageEditorState::new(message);
        let publisher = connection
            .borrow_mut()
            .create_publisher(&topic, &message_type.clone())
            .expect("Failed to subscribe to topic");
        Self {
            topic,
            connection,
            publisher,
            message_editor,
            message_type,
        }
    }
}

impl TuiView for TopicPublisherState {
    fn handle_event(&mut self, event: Event) -> Option<Views> {
        match event {
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    return None;
                }
                match key.code {
                    KeyCode::Char('p') => {
                        let new_message = DynamicMessage::new(self.message_type.clone())
                            .expect("Failed to create new message");
                        if let Err(e) = self.publisher.publish(new_message) {
                            eprintln!("Failed to publish message: {}", e);
                        }
                    }
                    _ => return None,
                }
            }
            _ => return None,
        }
        self.message_editor.handle_event(event)
    }

    fn name(&self) -> String {
        format!("Topic Publisher - {}", self.topic)
    }
}

impl TopicPublisherWidget {
    pub fn render(area: Rect, buf: &mut Buffer, state: &mut TopicPublisherState) {
        let block = Block::bordered()
            .title(Line::from(format!("Topic Publisher - {}", state.topic)).centered())
            .border_style(HEADER_STYLE);

        let message_editor_widget = MessageEditorWidget::new();

        message_editor_widget.render(area, buf, &mut state.message_editor);
    }
}
