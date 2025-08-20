use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use ratatui::{
    prelude::{Buffer, Rect, Style},
    text::Line,
    widgets::{Block, Paragraph, Widget},
};
use rclrs::*;

use crate::{
    common::{
        generic_message::{
            ArrayField, BoundedSequenceField, GenericField, GenericMessage, MessageMetadata,
            SequenceField, SimpleField,
        },
        generic_message_selection::next_field,
        style::{HEADER_STYLE, SELECTED_STYLE},
    },
    connections::{Connection, ConnectionType},
    // generic_message::{GenericField, GenericMessage},
    views::{live_plot::LivePlotState, TuiView, Views},
    widgets::message_widget::MessageWidget,
};

use crossterm::event::{Event, KeyCode};

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

    pub fn select_next_field(&mut self) {
        if let Some(message) = self.message.lock().unwrap().as_ref() {
            self.selected_fields = next_field(&message, &self.selected_fields).unwrap_or_default();
        }
    }

    pub fn select_previous_field(&mut self) {
        if let Some(message) = self.message.lock().unwrap().as_ref() {
            // self.selected_fields = prev_field(&message, &self.selected_fields).unwrap_or_default();
        }
    }
}

impl TuiView for RawMessageState {
    fn handle_event(&mut self, event: Event) -> Option<Views> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Down => self.select_next_field(),
                KeyCode::Up => self.select_previous_field(),
                KeyCode::Char('l') | KeyCode::Right => {
                    if !self.selected_fields.is_empty() {
                        let live_plot_state = LivePlotState::new(
                            self.topic.clone(),
                            self.selected_fields.clone(),
                            self.connection.clone(),
                        );
                        return Some(Views::LivePlot(live_plot_state));
                    }
                }
                _ => {}
            },
            _ => {}
        }
        None
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
            .border_style(HEADER_STYLE);

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
