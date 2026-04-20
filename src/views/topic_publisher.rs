use std::{cell::RefCell, rc::Rc};

use ratatui::{
    prelude::{Buffer, Rect},
    text::Line,
    widgets::{Block, BorderType, StatefulWidget, Widget},
};
use rclrs::*;

use crate::{
    common::{
        event::Event,
        generic_message::{
            AnyTypeMutableRef, BoundedSequenceField, GenericField, GenericMessage, InterfaceType,
            Length, SequenceField, SimpleField,
        },
        generic_message_selector::{get_field_category, FieldCategory},
        style::HEADER_STYLE,
    },
    connections::{Connection, ConnectionType},
    views::{
        message_pane::{commit_field_edit, MessagePaneState},
        FromTopic, TopicInfo, TuiView,
    },
    widgets::message_widget::MessageWidget,
};

use crossterm::event::{Event as CrosstermEvent, KeyCode};

pub struct TopicPublisherWidget;

pub struct TopicPublisherState {
    topic: String,
    _connection: Rc<RefCell<ConnectionType>>,
    publisher: Box<dyn Fn(&GenericMessage) -> Result<Vec<String>, String>>,
    message: GenericMessage,
    pane: MessagePaneState,
    is_editing: bool,
    field_content: String,
    needs_redraw: bool,
    counter: usize,
}

impl TopicPublisherState {
    pub fn new(
        topic: String,
        topic_type: InterfaceType,
        connection: Rc<RefCell<ConnectionType>>,
    ) -> Self {
        let message_type = MessageTypeName {
            package_name: topic_type.package_name.clone(),
            type_name: topic_type.type_name.clone(),
        };
        let message = DynamicMessage::new(message_type.clone()).expect("Failed to create message");
        let generic_message = GenericMessage::from(message.view());
        let publisher = connection
            .borrow_mut()
            .create_publisher(&topic, &topic_type)
            .expect("Failed to subscribe to topic");
        Self {
            topic,
            _connection: connection,
            publisher,
            message: generic_message,
            pane: MessagePaneState::new(),
            is_editing: false,
            field_content: String::new(),
            needs_redraw: true,
            counter: 0,
        }
    }

    /// Returns true if the message has a `header` field of type `std_msgs/msg/Header`.
    pub fn has_header_stamp(&self) -> bool {
        if let Some(GenericField::Simple(SimpleField::Message(header))) = self.message.get("header")
        {
            let t = header.type_name();
            return t.package_name == "std_msgs" && t.category == "msg" && t.type_name == "Header";
        }
        false
    }

    /// Sets header.stamp.sec and header.stamp.nanosec to the current system time.
    fn apply_auto_stamp(&mut self) {
        use std::time::SystemTime;
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        // header.stamp.sec = [0, 0, 0]
        if let Ok(AnyTypeMutableRef::Int32(sec)) = self.message.get_mut_deep_index(&[0, 0, 0]) {
            *sec = now.as_secs() as i32;
        }
        // header.stamp.nanosec = [0, 0, 1]
        if let Ok(AnyTypeMutableRef::Uint32(nanosec)) = self.message.get_mut_deep_index(&[0, 0, 1])
        {
            *nanosec = now.subsec_nanos();
        }
    }

    pub fn commit_edit(&mut self) -> Result<(), String> {
        self.needs_redraw = true;
        commit_field_edit(
            &mut self.message,
            &self.pane.selected_fields,
            &self.field_content,
        )
    }
}

impl TuiView for TopicPublisherState {
    fn handle_event(&mut self, event: Event) -> Event {
        match event {
            Event::Key(CrosstermEvent::Key(key_event)) => {
                if key_event.kind != crossterm::event::KeyEventKind::Press {
                    return event;
                }

                match key_event.code {
                    KeyCode::Char('p') => {
                        if self.is_editing {
                            self.field_content.push('p');
                            self.needs_redraw = true;
                            Event::None
                        } else {
                            if self.has_header_stamp() {
                                self.apply_auto_stamp();
                            }
                            match self.publisher.as_ref()(&self.message) {
                                Ok(warnings) if !warnings.is_empty() => {
                                    self.counter = self.counter.saturating_add(1);
                                    self.needs_redraw = true;
                                    Event::Error(format!(
                                        "Published with warnings:\n{}",
                                        warnings.join("\n")
                                    ))
                                }
                                Ok(_) => {
                                    self.counter = self.counter.saturating_add(1);
                                    self.needs_redraw = true;
                                    Event::None
                                }
                                Err(e) => Event::Error(format!("Failed to publish: {}", e)),
                            }
                        }
                    }
                    KeyCode::Char('j')
                    | KeyCode::Down
                    | KeyCode::Char('k')
                    | KeyCode::Up
                    | KeyCode::Char('G')
                        if !self.is_editing =>
                    {
                        if self.pane.handle_nav_key(key_event, &self.message) {
                            self.needs_redraw = true;
                        }
                        Event::None
                    }
                    KeyCode::Char('-') => {
                        self.needs_redraw = true;
                        if self.is_editing {
                            self.field_content.push('-');
                            Event::None
                        } else {
                            if let Ok(field) =
                                self.message.get_mut_deep_index(&self.pane.selected_fields)
                            {
                                match field {
                                    AnyTypeMutableRef::Sequence(SequenceField::Message(_)) => {
                                        return Event::Error(
                                            "Cannot resize sequence of messages".to_string(),
                                        );
                                    }
                                    AnyTypeMutableRef::BoundedSequence(
                                        BoundedSequenceField::Message(_, _),
                                    ) => {
                                        return Event::Error(
                                            "Cannot resize sequence of messages".to_string(),
                                        );
                                    }
                                    AnyTypeMutableRef::Sequence(sequence_field) => {
                                        sequence_field
                                            .resize(sequence_field.len().saturating_sub(1));
                                    }
                                    AnyTypeMutableRef::BoundedSequence(sequence_field) => {
                                        sequence_field
                                            .resize(sequence_field.len().saturating_sub(1));
                                    }
                                    _ => {}
                                }
                            }
                            Event::None
                        }
                    }
                    KeyCode::Char('+') => {
                        self.needs_redraw = true;
                        if self.is_editing {
                            self.field_content.push('+');
                            Event::None
                        } else {
                            if let Ok(field) =
                                self.message.get_mut_deep_index(&self.pane.selected_fields)
                            {
                                match field {
                                    AnyTypeMutableRef::Sequence(SequenceField::Message(_)) => {
                                        return Event::Error(
                                            "Cannot resize sequence of messages".to_string(),
                                        );
                                    }
                                    AnyTypeMutableRef::BoundedSequence(
                                        BoundedSequenceField::Message(_, _),
                                    ) => {
                                        return Event::Error(
                                            "Cannot resize sequence of messages".to_string(),
                                        );
                                    }
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
                            self.needs_redraw = true;
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
                            let commit_result = self.commit_edit();
                            self.field_content.clear();
                            self.needs_redraw = true;
                            match commit_result {
                                Ok(()) => Event::None,
                                Err(e) => Event::Error(format!("Failed to commit edit: {}", e)),
                            }
                        } else if get_field_category(&self.message, &self.pane.selected_fields)
                            == Some(FieldCategory::Base)
                        {
                            self.is_editing = true;
                            self.field_content.clear();
                            self.needs_redraw = true;
                            Event::None
                        } else {
                            event
                        }
                    }
                    KeyCode::Char(c) => {
                        if self.is_editing {
                            self.field_content.push(c);
                            self.needs_redraw = true;
                            Event::None
                        } else {
                            event
                        }
                    }
                    _ => event,
                }
            }
            Event::Key(_)
            | Event::None
            | Event::NewConnection(_)
            | Event::NewNode(_)
            | Event::NewTopic(_)
            | Event::NewField(_)
            | Event::Error(_)
            | Event::ClosePopup
            | Event::NewView(_) => event,
        }
    }

    fn name(&self) -> String {
        format!("Topic Publisher - {}", self.topic)
    }

    fn get_help_text(&self) -> String {
        "Topic Publisher View Help:\n\
        - 'j' or ↓: Move down in the message fields.\n\
        - 'k' or ↑: Move up in the message fields.\n\
        - 'G': Jump to the last field in the message.\n\
        - '+': Increase size of sequence field.\n\
        - '-': Decrease size of sequence field.\n\
        - 'p': Publish the current message (only when not editing).\n\
        - 'Enter': Toggle edit mode for primitive fields and commit changes when exiting edit mode.\n\
        - 'Backspace': Remove last character from the field content when editing."
            .to_string()
    }

    fn needs_redraw(&mut self) -> bool {
        if self.needs_redraw {
            self.needs_redraw = false;
            return true;
        }
        false
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        TopicPublisherWidget::render(area, buf, self);
    }
}

impl FromTopic for TopicPublisherState {
    fn from_topic(topic_info: TopicInfo) -> Self {
        let message_type = topic_info
            .connection
            .borrow()
            .get_topic_type(&topic_info.topic)
            .expect("Failed to get topic type");
        TopicPublisherState::new(topic_info.topic, message_type, topic_info.connection)
    }
}

impl TopicPublisherWidget {
    pub fn render(area: Rect, buf: &mut Buffer, state: &mut TopicPublisherState) {
        let block = Block::bordered()
            .title(Line::from(format!(" Topic Publisher - {} ", state.topic)).centered())
            .border_style(HEADER_STYLE)
            .border_type(BorderType::Rounded)
            .title_bottom(
                Line::from(format!(" Publish counter: {} ", state.counter)).left_aligned(),
            )
            .title_bottom(match state.is_editing {
                true => Line::from("INSERT").right_aligned(),
                false => Line::from("NORMAL").right_aligned(),
            });

        let mut message_widget = MessageWidget::new(&state.message).block(block);
        if !state.pane.selected_fields.is_empty() {
            message_widget = message_widget.with_selection(&state.pane.selected_fields);
            if state.is_editing {
                message_widget = message_widget.with_edit(&state.field_content);
            }
        }

        StatefulWidget::render(message_widget, area, buf, &mut state.pane.widget_state);

        // If the message has header.stamp fields, draw an "auto stamp" indicator
        // on the right side of the "stamp:" row (2nd row of the inner area, y+2
        // because y+0 is the top border and y+1 is "header:", y+2 is "  stamp:").
        if state.has_header_stamp() && area.height > 2 {
            use ratatui::style::{Color, Style};
            let label = " [auto stamp] ";
            let inner_x = area.x + 1; // inside left border
            let inner_width = area.width.saturating_sub(2);
            let stamp_row = area.y + 2; // border(1) + header:(1) + stamp:(this row)
            let label_len = label.len() as u16;
            if inner_width >= label_len {
                let label_x = inner_x + inner_width - label_len;
                buf.set_string(
                    label_x,
                    stamp_row,
                    label,
                    Style::default().fg(Color::Yellow),
                );
            }
        }
    }
}
