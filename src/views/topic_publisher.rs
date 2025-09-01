use std::{cell::RefCell, rc::Rc};

use ratatui::{
    prelude::{Buffer, Rect},
    text::Line,
    widgets::{Block, BorderType, Widget},
};
use rclrs::*;

use crate::{
    common::{
        event::Event,
        generic_message::{AnyTypeMutableRef, GenericMessage, Length},
        generic_message_selector::{get_field_category, FieldCategory, GenericMessageSelector},
        style::HEADER_STYLE,
    },
    connections::{Connection, ConnectionType},
    // generic_message::{GenericField, GenericMessage},
    views::TuiView,
    widgets::message_widget::MessageWidget,
};

use crossterm::event::{Event as CrosstermEvent, KeyCode};

pub struct TopicPublisherWidget;

pub struct TopicPublisherState {
    topic: String,
    connection: Rc<RefCell<ConnectionType>>,
    publisher: Box<dyn Fn(&GenericMessage)>,
    message: GenericMessage,
    selected_fields: Vec<usize>,
    is_editing: bool,
    field_content: String,
}

impl TopicPublisherState {
    pub fn new(topic: String, connection: Rc<RefCell<ConnectionType>>) -> Self {
        let message_type = MessageTypeName {
            package_name: "test_msgs".to_string(),
            type_name: "BoundedSequences".to_string(),
        };
        let message = DynamicMessage::new(message_type.clone()).expect("Failed to create message");
        let generic_message = GenericMessage::from(message.view());
        let publisher = connection
            .borrow_mut()
            .create_publisher(&topic, &message_type.clone())
            .expect("Failed to subscribe to topic");
        Self {
            topic,
            connection,
            publisher,
            message: generic_message,
            selected_fields: Vec::new(),
            is_editing: false,
            field_content: String::new(),
        }
    }

    pub fn select_next_field(&mut self) {
        self.selected_fields =
            GenericMessageSelector::new(&self.message).down(&self.selected_fields);
    }

    pub fn select_previous_field(&mut self) {
        self.selected_fields = GenericMessageSelector::new(&self.message).up(&self.selected_fields);
    }

    pub fn commit_edit(&mut self) -> Result<(), String> {
        // Update the message with the new field content
        let value = self
            .message
            .get_mut_deep_index(&self.selected_fields)
            .or_else(|e| Err(e))?;
        match value {
            AnyTypeMutableRef::Float(v) => {
                *v = self
                    .field_content
                    .parse::<f32>()
                    .map_err(|e| format!("Failed to parse float: {}", e))?;
                Ok(())
            }
            AnyTypeMutableRef::Double(v) => {
                *v = self
                    .field_content
                    .parse::<f64>()
                    .map_err(|e| format!("Failed to parse double: {}", e))?;
                Ok(())
            }
            AnyTypeMutableRef::Boolean(v) => {
                *v = self
                    .field_content
                    .parse::<bool>()
                    .map_err(|e| format!("Failed to parse boolean: {}", e))?;
                Ok(())
            }
            AnyTypeMutableRef::Uint8(v) => {
                *v = self
                    .field_content
                    .parse::<u8>()
                    .map_err(|e| format!("Failed to parse uint8: {}", e))?;
                Ok(())
            }
            AnyTypeMutableRef::Int8(v) => {
                *v = self
                    .field_content
                    .parse::<i8>()
                    .map_err(|e| format!("Failed to parse int8: {}", e))?;
                Ok(())
            }
            AnyTypeMutableRef::Uint16(v) => {
                *v = self
                    .field_content
                    .parse::<u16>()
                    .map_err(|e| format!("Failed to parse uint16: {}", e))?;
                Ok(())
            }
            AnyTypeMutableRef::Int16(v) => {
                *v = self
                    .field_content
                    .parse::<i16>()
                    .map_err(|e| format!("Failed to parse int16: {}", e))?;
                Ok(())
            }
            AnyTypeMutableRef::Uint32(v) => {
                *v = self
                    .field_content
                    .parse::<u32>()
                    .map_err(|e| format!("Failed to parse uint32: {}", e))?;
                Ok(())
            }
            AnyTypeMutableRef::Int32(v) => {
                *v = self
                    .field_content
                    .parse::<i32>()
                    .map_err(|e| format!("Failed to parse int32: {}", e))?;
                Ok(())
            }
            AnyTypeMutableRef::Uint64(v) => {
                *v = self
                    .field_content
                    .parse::<u64>()
                    .map_err(|e| format!("Failed to parse uint64: {}", e))?;
                Ok(())
            }
            AnyTypeMutableRef::Int64(v) => {
                *v = self
                    .field_content
                    .parse::<i64>()
                    .map_err(|e| format!("Failed to parse int64: {}", e))?;
                Ok(())
            }
            AnyTypeMutableRef::String(v) => {
                *v = self.field_content.clone();
                Ok(())
            }
            AnyTypeMutableRef::Array(_)
            | AnyTypeMutableRef::Sequence(_)
            | AnyTypeMutableRef::BoundedSequence(_) => {
                Err("Cannot edit non-primitive field".to_string())
            }
        }
    }
}

impl TuiView for TopicPublisherState {
    fn handle_event(&mut self, event: Event) -> Event {
        if let Event::Key(CrosstermEvent::Key(key_event)) = event {
            if key_event.kind != crossterm::event::KeyEventKind::Press {
                return event;
            }

            match key_event.code {
                KeyCode::Char('p') => {
                    if self.is_editing {
                        self.field_content.push('p');
                        Event::None
                    } else {
                        self.publisher.as_ref()(&self.message);
                        Event::None
                    }
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    if self.is_editing {
                        self.field_content.push('j');
                        Event::None
                    } else {
                        self.select_next_field();
                        Event::None
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if self.is_editing {
                        self.field_content.push('k');
                        Event::None
                    } else {
                        self.select_previous_field();
                        Event::None
                    }
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    if self.is_editing {
                        self.field_content.push('h');
                        Event::None
                    } else {
                        if let Some(field) =
                            self.message.get_mut_deep_index(&self.selected_fields).ok()
                        {
                            match field {
                                AnyTypeMutableRef::Sequence(sequence_field) => {
                                    sequence_field.resize(sequence_field.len().saturating_sub(1));
                                }
                                AnyTypeMutableRef::BoundedSequence(sequence_field) => {
                                    sequence_field.resize(sequence_field.len().saturating_sub(1));
                                }
                                _ => {}
                            }
                        }
                        Event::None
                    }
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    if self.is_editing {
                        self.field_content.push('l');
                        Event::None
                    } else {
                        if let Some(field) =
                            self.message.get_mut_deep_index(&self.selected_fields).ok()
                        {
                            match field {
                                AnyTypeMutableRef::Sequence(sequence_field) => {
                                    sequence_field.resize(sequence_field.len() + 1);
                                }
                                AnyTypeMutableRef::BoundedSequence(sequence_field) => {
                                    sequence_field.resize(sequence_field.len() + 1);
                                }
                                _ => {}
                            }
                        }
                        Event::None
                    }
                }
                KeyCode::Backspace => {
                    if self.is_editing {
                        self.field_content.pop();
                        Event::None
                    } else {
                        event
                    }
                }
                KeyCode::Enter => {
                    if self.is_editing {
                        // Update the message with the new field content
                        self.is_editing = false;
                        self.commit_edit().unwrap_or_else(|e| {
                            eprintln!("Failed to commit edit: {}", e);
                        });
                        self.field_content.clear();
                        Event::None
                    } else {
                        if get_field_category(&self.message, &self.selected_fields)
                            == Some(FieldCategory::Base)
                        {
                            self.is_editing = true;
                            self.field_content.clear();
                            Event::None
                        } else {
                            event
                        }
                    }
                }
                KeyCode::Char(c) => {
                    if self.is_editing {
                        self.field_content.push(c);
                        Event::None
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
        format!("Topic Publisher - {}", self.topic)
    }
}

impl TopicPublisherWidget {
    pub fn render(area: Rect, buf: &mut Buffer, state: &mut TopicPublisherState) {
        let block = Block::bordered()
            .title(
                Line::from(format!(
                    "Topic Publisher - {} {}",
                    state.topic, state.is_editing
                ))
                .centered(),
            )
            .border_style(HEADER_STYLE)
            .border_type(BorderType::Rounded);

        let mut message_widget = MessageWidget::new(&state.message).block(block);
        if !state.selected_fields.is_empty() {
            message_widget = message_widget.with_selection(&state.selected_fields);
            if state.is_editing {
                message_widget = message_widget.with_edit(&state.field_content);
            }
        }

        message_widget.render(area, buf);
    }
}
