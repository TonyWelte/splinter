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
        dynamic_message_selection::next_field,
        generic_message::{
            ArrayField, BoundedSequenceField, GenericField, GenericMessage, MessageMetadata,
            SequenceField, SimpleField,
        },
        style::{HEADER_STYLE, SELECTED_STYLE},
    },
    connections::{Connection, ConnectionType},
    // generic_message::{GenericField, GenericMessage},
    widgets::{live_plot::LivePlotState, TuiView, Views},
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

            Widget::render(block, area, buf);
            render_message(&message, &state.selected_fields, area, buf, 1, 0);
        } else {
            let paragraph = Paragraph::new("No message available").block(block);
            Widget::render(paragraph, area, buf);
        }
    }
}

fn render_message(
    msg: &GenericMessage,
    selected_item: &[usize],
    area: Rect,
    buf: &mut Buffer,
    line_index: u16,
    column_index: u16,
) -> u16 {
    let mut new_line_index = line_index;

    for (index, (name, value)) in msg.iter().enumerate() {
        buf.set_string(
            area.x + column_index,
            area.y + new_line_index,
            format!("{}: ", name),
            Style::default(),
        );
        let is_selected = index == *selected_item.first().unwrap_or(&usize::MAX);

        let next_selected_item = if is_selected {
            selected_item[1..].to_vec()
        } else {
            Vec::new()
        };
        new_line_index = value.display(
            &next_selected_item,
            is_selected,
            area,
            buf,
            new_line_index,
            column_index,
            &name,
        );
    }
    new_line_index
}

trait ValueDisplay {
    fn display(
        &self,
        selected_item: &[usize],
        is_selected: bool,
        area: Rect,
        buf: &mut Buffer,
        line_index: u16,
        column_index: u16,
        name: &str,
    ) -> u16;
}

impl ValueDisplay for GenericField {
    fn display(
        &self,
        selected_item: &[usize],
        is_selected: bool,
        area: Rect,
        buf: &mut Buffer,
        line_index: u16,
        column_index: u16,
        name: &str,
    ) -> u16 {
        match self {
            GenericField::Simple(simple_value) => simple_value.display(
                selected_item,
                is_selected,
                area,
                buf,
                line_index,
                column_index,
                name,
            ),
            GenericField::Array(array_value) => array_value.display(
                selected_item,
                is_selected,
                area,
                buf,
                line_index,
                column_index,
                name,
            ),
            GenericField::Sequence(sequence_value) => sequence_value.display(
                selected_item,
                is_selected,
                area,
                buf,
                line_index,
                column_index,
                name,
            ),
            GenericField::BoundedSequence(bounded_sequence_value) => bounded_sequence_value
                .display(
                    selected_item,
                    is_selected,
                    area,
                    buf,
                    line_index,
                    column_index,
                    name,
                ),
        }
    }
}

impl ValueDisplay for SimpleField {
    fn display(
        &self,
        selected_item: &[usize],
        is_selected: bool,
        area: Rect,
        buf: &mut Buffer,
        line_index: u16,
        column_index: u16,
        name: &str,
    ) -> u16 {
        buf.set_string(
            area.x + column_index,
            area.y + line_index,
            format!("{}: ", name),
            Style::default(),
        );
        let style = if is_selected {
            SELECTED_STYLE
        } else {
            Style::default()
        };
        match self {
            SimpleField::Float(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleField::Double(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleField::LongDouble(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{:?}", value), // TODO: Handle LongDouble display properly
                    style,
                );
            }
            SimpleField::Char(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("\'{}\'", value),
                    style,
                );
            }
            SimpleField::WChar(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("\'{}\'", value),
                    style,
                );
            }
            SimpleField::Boolean(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleField::Octet(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleField::Uint8(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleField::Int8(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleField::Uint16(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleField::Int16(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleField::Uint32(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleField::Int32(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleField::Uint64(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleField::Int64(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleField::String(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("\"{}\"", value),
                    style,
                );
            }
            SimpleField::BoundedString(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("\"{}\"", value),
                    style,
                );
            }
            SimpleField::WString(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("\"{}\"", value),
                    style,
                );
            }
            SimpleField::BoundedWString(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("\"{}\"", value),
                    style,
                );
            }
            SimpleField::Message(value) => {
                // If the field is a message, we can render its fields
                let new_line_index = render_message(
                    value,
                    selected_item,
                    area,
                    buf,
                    line_index + 1,
                    column_index + 2,
                );
                return new_line_index;
            }
        }
        line_index + 1
    }
}

impl ValueDisplay for ArrayField {
    fn display(
        &self,
        selected_item: &[usize],
        is_selected: bool,
        area: Rect,
        buf: &mut Buffer,
        line_index: u16,
        column_index: u16,
        name: &str,
    ) -> u16 {
        let style = if is_selected {
            SELECTED_STYLE
        } else {
            Style::default()
        };
        match self {
            ArrayField::Message(inner_msgs) => {
                // Render each message in the array
                let mut new_line_index = line_index;
                for inner_msg in inner_msgs.iter() {
                    new_line_index = render_message(
                        inner_msg,
                        selected_item,
                        area,
                        buf,
                        new_line_index + 1,
                        column_index + 2,
                    );
                }
                new_line_index
            }
            _ => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{:?}", self),
                    style,
                );
                line_index + 1
            }
        }
    }
}

impl ValueDisplay for SequenceField {
    fn display(
        &self,
        selected_item: &[usize],
        is_selected: bool,
        area: Rect,
        buf: &mut Buffer,
        line_index: u16,
        column_index: u16,
        name: &str,
    ) -> u16 {
        let style = if is_selected {
            SELECTED_STYLE
        } else {
            Style::default()
        };
        match self {
            SequenceField::Message(inner_msgs) => {
                // Render each message in the sequence
                let mut new_line_index = line_index;
                for inner_msg in inner_msgs.iter() {
                    new_line_index = render_message(
                        inner_msg,
                        &Vec::new(), // No selection in sequence context
                        area,
                        buf,
                        new_line_index + 1,
                        column_index + 2,
                    );
                }
                new_line_index
            }
            _ => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{:?}", self),
                    style,
                );
                line_index + 1
            }
        }
    }
}

impl ValueDisplay for BoundedSequenceField {
    fn display(
        &self,
        selected_item: &[usize],
        is_selected: bool,
        area: Rect,
        buf: &mut Buffer,
        line_index: u16,
        column_index: u16,
        name: &str,
    ) -> u16 {
        let style = if is_selected {
            SELECTED_STYLE
        } else {
            Style::default()
        };
        match self {
            BoundedSequenceField::Message(inner_msgs) => {
                // Render each message in the bounded sequence
                let mut new_line_index = line_index;
                for inner_msg in inner_msgs.iter() {
                    new_line_index = render_message(
                        inner_msg,
                        &Vec::new(), // No selection in bounded sequence context
                        area,
                        buf,
                        new_line_index + 1,
                        column_index + 2,
                    );
                }
                new_line_index
            }
            _ => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{:?}", self),
                    style,
                );
                line_index + 1
            }
        }
    }
}
