use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use ratatui::{
    prelude::{Buffer, Rect},
    text::Line,
    widgets::{Block, BorderType, Paragraph, Widget},
};
use rclrs::*;

use crate::{
    common::{
        event::{Event, NewLineEvent},
        generic_message::{GenericMessage, MessageMetadata},
        generic_message_selector::GenericMessageSelector,
        style::HEADER_STYLE,
    },
    connections::{Connection, ConnectionType},
    // generic_message::{GenericField, GenericMessage},
    views::TuiView,
    widgets::message_widget::MessageWidget,
};

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEventKind};

pub struct RawMessageWidget;

pub struct RawMessageState {
    pub topic: String,
    pub message: Arc<Mutex<Option<GenericMessage>>>,
    raw_messages: Vec<Vec<u8>>, // TODO(@TonyWelte): Find a way to avoid copying the messages
    index: usize,
    connection: Rc<RefCell<ConnectionType>>,
    selected_fields: Vec<usize>,
}

impl RawMessageState {
    pub fn new(topic: String, connection: Rc<RefCell<ConnectionType>>) -> Self {
        let message_type = connection
            .borrow()
            .get_topic_type(&topic)
            .expect("Failed to get topic type");
        let message = Arc::new(Mutex::new(None));
        let message_copy = message.clone();
        connection
            .borrow_mut()
            .subscribe(
                &topic,
                move |msg: GenericMessage, _msg_info: MessageMetadata| {
                    let mut mut_message = message_copy.lock().unwrap();
                    *mut_message = Some(msg);
                },
            )
            .expect("Failed to subscribe to topic");
        let object = Self {
            topic: topic.clone(),
            message,
            raw_messages: Vec::new(),
            index: 0,
            connection,
            selected_fields: Vec::new(),
        };
        object
    }

    pub fn select_down(&mut self) {
        if let Some(message) = self.message.lock().unwrap().as_ref() {
            self.selected_fields =
                GenericMessageSelector::new(&message).down(&self.selected_fields);
        }
    }

    pub fn select_up(&mut self) {
        if let Some(message) = self.message.lock().unwrap().as_ref() {
            self.selected_fields = GenericMessageSelector::new(&message).up(&self.selected_fields);
        }
    }

    pub fn select_left(&mut self) {
        if let Some(message) = self.message.lock().unwrap().as_ref() {
            self.selected_fields =
                GenericMessageSelector::new(&message).left(&self.selected_fields);
        }
    }

    pub fn select_right(&mut self) {
        if let Some(message) = self.message.lock().unwrap().as_ref() {
            self.selected_fields =
                GenericMessageSelector::new(&message).right(&self.selected_fields);
        }
    }

    pub fn select_last(&mut self) {
        if let Some(message) = self.message.lock().unwrap().as_ref() {
            self.selected_fields = GenericMessageSelector::new(&message).last_field_path();
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
                    self.select_down();
                    Event::None
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.select_up();
                    Event::None
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    self.select_right();
                    Event::None
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    self.select_left();
                    Event::None
                }
                KeyCode::Char('g') => {
                    todo!("Wait for double g");
                }
                KeyCode::Char('G') => {
                    self.select_last();
                    Event::None
                }
                KeyCode::Enter => Event::NewLine(NewLineEvent {
                    topic: self.topic.clone(),
                    field: self.selected_fields.clone(),
                    view: None,
                }),
                _ => event,
            }
        } else {
            event
        }
    }

    fn name(&self) -> String {
        format!("Raw Message - {}", self.topic)
    }
}

impl RawMessageWidget {
    pub fn render(area: Rect, buf: &mut Buffer, state: &mut RawMessageState) {
        let block = Block::bordered()
            .title(
                Line::raw(format!(
                    "Raw Message ({}/{}) {:?}",
                    state.index + 1,
                    state.raw_messages.len(),
                    state.selected_fields
                ))
                .centered(),
            )
            .border_style(HEADER_STYLE)
            .border_type(BorderType::Rounded);

        if let Some(message) = &*state.message.lock().unwrap() {
            // Clear the area before rendering
            for x in area.left()..area.right() {
                for y in area.top()..area.bottom() {
                    buf.cell_mut((x, y)).unwrap().reset();
                }
            }

            let message_widget = MessageWidget::new(message)
                .with_selection(&state.selected_fields)
                .block(block);
            Widget::render(message_widget, area, buf);
        } else {
            let paragraph = Paragraph::new("No message available").block(block);
            Widget::render(paragraph, area, buf);
        }
    }
}
