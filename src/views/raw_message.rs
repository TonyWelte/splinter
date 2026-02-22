use std::{
    cell::RefCell,
    rc::Rc,
    sync::{atomic::AtomicBool, Arc, Mutex},
};

use ratatui::{
    prelude::{Buffer, Rect},
    text::Line,
    widgets::{Block, BorderType, Clear, Paragraph, StatefulWidget, Widget},
};

use crate::{
    common::{
        event::Event,
        generic_message::{FieldType, GenericMessage, MessageMetadata},
        generic_message_selector::GenericMessageSelector,
        style::HEADER_STYLE,
    },
    connections::{Connection, ConnectionType},
    // generic_message::{GenericField, GenericMessage},
    views::{FieldInfo, FieldInfoType, FromTopic, TopicInfo, TuiView},
    widgets::message_widget::{MessageWidget, MessageWidgetState},
};

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEventKind};

pub struct RawMessageWidget;

pub struct RawMessageState {
    pub topic: String,
    pub message: Arc<Mutex<Option<GenericMessage>>>,
    _connection: Rc<RefCell<ConnectionType>>,
    selected_fields: Vec<usize>,
    message_widget_state: MessageWidgetState,
    needs_redraw: Arc<AtomicBool>,
}

impl RawMessageState {
    pub fn new(topic: String, connection: Rc<RefCell<ConnectionType>>) -> Self {
        let message = Arc::new(Mutex::new(None));
        let message_copy = message.clone();
        let needs_redraw = Arc::new(AtomicBool::new(true));
        let needs_redraw_copy = needs_redraw.clone();

        // Wait until the topic type is available or timeout after 1 second.
        // FIXME: This busy-waits on the main thread, blocking the TUI event loop for up to
        // 1 second during view creation. A non-blocking approach (e.g. moving the subscription
        // setup to a background thread or using a retry mechanism in handle_event) would
        // avoid this freeze.
        let mut message_type_wait_time = 0;
        while connection.borrow().get_topic_type(&topic).is_none() {
            std::thread::sleep(std::time::Duration::from_millis(100));
            message_type_wait_time += 100;
            if message_type_wait_time >= 1000 {
                break;
            }
        }

        connection
            .borrow_mut()
            .subscribe(
                &topic,
                move |msg: GenericMessage, _msg_info: MessageMetadata| {
                    let mut mut_message = message_copy.lock().unwrap();
                    needs_redraw_copy.store(true, std::sync::atomic::Ordering::Relaxed);
                    *mut_message = Some(msg);
                },
            )
            .expect("Failed to subscribe to topic");

        Self {
            topic: topic.clone(),
            message,
            _connection: connection,
            selected_fields: Vec::new(),
            message_widget_state: MessageWidgetState::new(true),
            needs_redraw,
        }
    }

    fn set_needs_redraw(&self) {
        self.needs_redraw
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn select_down(&mut self) {
        if let Some(message) = self.message.lock().unwrap().as_ref() {
            self.selected_fields = GenericMessageSelector::new(message).down(&self.selected_fields);
            self.set_needs_redraw();
        }
    }

    pub fn select_up(&mut self) {
        if let Some(message) = self.message.lock().unwrap().as_ref() {
            self.selected_fields = GenericMessageSelector::new(message).up(&self.selected_fields);
            self.set_needs_redraw();
        }
    }

    pub fn select_far_down(&mut self) {
        if let Some(message) = self.message.lock().unwrap().as_ref() {
            if self.selected_fields.is_empty() {
                self.selected_fields.push(0);
            }
            *self.selected_fields.last_mut().unwrap() += 1;
            if message.get_field_type(&self.selected_fields).is_err() {
                *self.selected_fields.last_mut().unwrap() -= 1;
            }
            self.set_needs_redraw();
        }
    }

    pub fn select_far_up(&mut self) {
        if let Some(_) = self.message.lock().unwrap().as_ref() {
            if let Some(last_selected) = self.selected_fields.last() {
                if *last_selected == 0 {
                    // If we're already at the top of the current level, jump to the top of the parent level
                    self.selected_fields.pop();
                } else {
                    // Otherwise, jump to the top of the current level
                    self.selected_fields.pop();
                    self.selected_fields.push(0);
                }
            }
            self.set_needs_redraw();
        }
    }

    pub fn select_left(&mut self) {
        if let Some(message) = self.message.lock().unwrap().as_ref() {
            self.selected_fields = GenericMessageSelector::new(message).left(&self.selected_fields);
            self.set_needs_redraw();
        }
    }

    pub fn select_right(&mut self) {
        if let Some(message) = self.message.lock().unwrap().as_ref() {
            self.selected_fields =
                GenericMessageSelector::new(message).right(&self.selected_fields);
            self.set_needs_redraw();
        }
    }

    pub fn select_last(&mut self) {
        if let Some(message) = self.message.lock().unwrap().as_ref() {
            self.selected_fields = GenericMessageSelector::new(message).last_field_path();
            self.set_needs_redraw();
        }
    }
}

impl TuiView for RawMessageState {
    fn handle_event(&mut self, event: Event) -> Event {
        if let Event::Key(CrosstermEvent::Key(key_event)) = event {
            if key_event.kind != KeyEventKind::Press {
                return event;
            }
            match key_event.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    if key_event
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::SHIFT)
                    {
                        self.select_far_down();
                    } else {
                        self.select_down();
                    }
                    Event::None
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if key_event
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::SHIFT)
                    {
                        self.select_far_up();
                    } else {
                        self.select_up();
                    }
                    Event::None
                }
                KeyCode::Char('G') => {
                    self.select_last();
                    Event::None
                }
                KeyCode::Enter => {
                    let message = self.message.lock().unwrap();
                    let message = match &*message {
                        Some(msg) => msg,
                        None => return Event::Error("No message available".to_string()),
                    };
                    match message.get_field_type(&self.selected_fields) {
                        Ok(FieldType::Float) | Ok(FieldType::Double) => {
                            Event::NewField(FieldInfo {
                                connection: self._connection.clone(),
                                topic: self.topic.clone(),
                                type_name: message.type_name().clone(),
                                field: self.selected_fields.clone(),
                                field_name: message
                                    .get_field_name(&self.selected_fields)
                                    .unwrap_or_else(|_| "this is a bug".to_string()),
                                field_type: FieldInfoType::Float,
                            })
                        }
                        Ok(FieldType::Boolean)
                        | Ok(FieldType::Int8)
                        | Ok(FieldType::Int16)
                        | Ok(FieldType::Int32)
                        | Ok(FieldType::Int64)
                        | Ok(FieldType::Uint8)
                        | Ok(FieldType::Uint16)
                        | Ok(FieldType::Uint32)
                        | Ok(FieldType::Uint64) => Event::NewField(FieldInfo {
                            connection: self._connection.clone(),
                            topic: self.topic.clone(),
                            type_name: message.type_name().clone(),
                            field: self.selected_fields.clone(),
                            field_name: message
                                .get_field_name(&self.selected_fields)
                                .unwrap_or_else(|_| "this is a bug".to_string()),
                            field_type: FieldInfoType::Integer,
                        }),
                        Ok(FieldType::String) => Event::NewField(FieldInfo {
                            connection: self._connection.clone(),
                            topic: self.topic.clone(),
                            type_name: message.type_name().clone(),
                            field: self.selected_fields.clone(),
                            field_name: message
                                .get_field_name(&self.selected_fields)
                                .unwrap_or_else(|_| "this is a bug".to_string()),
                            field_type: FieldInfoType::String,
                        }),
                        Ok(_) => Event::Error(
                            "Selected field is not a primitive type that can be plotted"
                                .to_string(),
                        ),
                        Err(e) => Event::Error(format!("Failed to get field type: {}", e)),
                    }
                }
                _ => event,
            }
        } else {
            event
        }
    }

    fn name(&self) -> String {
        format!("Raw Message - {}", self.topic)
    }

    fn get_help_text(&self) -> String {
        "Raw Message View Help:\n\
        - 'j' or ↓: Move down in the message fields.\n\
        - 'k' or ↑: Move up in the message fields.\n\
        - 'G': Jump to the last field in the message.\n\
        - 'Enter': Create a new plot for the selected primitive field."
            .to_string()
    }

    fn needs_redraw(&mut self) -> bool {
        if self.needs_redraw.load(std::sync::atomic::Ordering::Relaxed) {
            self.needs_redraw
                .store(false, std::sync::atomic::Ordering::Relaxed);
            return true;
        }
        false
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        RawMessageWidget::render(area, buf, self);
    }
}

impl FromTopic for RawMessageState {
    fn from_topic(topic_info: TopicInfo) -> Self {
        RawMessageState::new(topic_info.topic, topic_info.connection)
    }
}

impl RawMessageWidget {
    pub fn render(area: Rect, buf: &mut Buffer, state: &mut RawMessageState) {
        let mut block = Block::bordered()
            .border_style(HEADER_STYLE)
            .border_type(BorderType::Rounded);

        if let Some(message) = &*state.message.lock().unwrap() {
            // Clear the area before rendering
            Clear.render(area, buf);

            block = if state.selected_fields.is_empty() {
                block.title(
                    Line::raw(format!(" Raw Message {} (no field selected) ", state.topic))
                        .centered(),
                )
            } else {
                block.title(
                    Line::raw(format!(
                        " Raw Message {} {} ",
                        state.topic,
                        message
                            .get_field_name(&state.selected_fields)
                            .unwrap_or_else(|_| "".to_string())
                    ))
                    .centered(),
                )
            };

            let message_widget = MessageWidget::new(message)
                .with_selection(&state.selected_fields)
                .block(block);
            StatefulWidget::render(message_widget, area, buf, &mut state.message_widget_state);
        } else {
            block = block.title(Line::raw(format!(" Raw Message {} ", state.topic)).centered());

            let paragraph = Paragraph::new("No message available").block(block);
            Widget::render(paragraph, area, buf);
        }
    }
}
