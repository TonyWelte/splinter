use ratatui::{
    prelude::{Buffer, Rect, Style},
    widgets::StatefulWidget,
};
use rclrs::*;

use crate::{
    common::{generic_message_selection::next_field, style::SELECTED_STYLE},
    connections::Connection,
    // generic_message::{GenericField, GenericMessage},
    views::{TuiView, Views},
};

use crossterm::event::{Event, KeyCode};

pub struct MessageEditorWidget;

impl MessageEditorWidget {
    pub fn new() -> Self {
        MessageEditorWidget {}
    }
}

pub struct MessageEditorState {
    pub message: DynamicMessage,
    selected_fields: Vec<usize>,
    is_editing: bool,
    field_content: String,
}

impl MessageEditorState {
    pub fn new(message: DynamicMessage) -> Self {
        Self {
            message,
            selected_fields: Vec::new(),
            is_editing: false,
            field_content: String::new(),
        }
    }

    pub fn select_next_field(&mut self) {
        // self.selected_fields =
        //     next_field(&self.message.view(), &self.selected_fields).unwrap_or_default();
    }

    pub fn select_previous_field(&mut self) {
        // self.selected_fields =
        //     prev_field(&self.message.view(), &self.selected_fields).unwrap_or_default();
    }
}

impl TuiView for MessageEditorState {
    fn handle_event(&mut self, event: Event) -> Option<Views> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Down => {
                    if !self.is_editing {
                        self.select_next_field()
                    }
                }
                KeyCode::Up => {
                    if !self.is_editing {
                        self.select_previous_field()
                    }
                }
                KeyCode::Enter => {
                    self.is_editing = !self.is_editing;
                }
                _ => {}
            },
            _ => {}
        }
        None
    }

    fn name(&self) -> String {
        "Message Editor".to_string()
    }
}

impl StatefulWidget for MessageEditorWidget {
    type State = MessageEditorState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Clear the area before rendering
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                buf.cell_mut((x, y)).unwrap().reset();
            }
        }

        render_message(
            &state.message.view(),
            &state.selected_fields,
            area,
            buf,
            1,
            0,
        );
    }
}

// struct ValueWidget<'a> {
//     value: Value<'a>,
//     is_selected: bool,
// }

// impl ValueWidget<'_> {
//     pub fn new(value: Value, is_selected: bool) -> Self {
//         ValueWidget { value, is_selected }
//     }
// }

// impl Widget for ValueWidget<'_> {
//     fn render(self, area: Rect, buf: &mut Buffer) {
//         let style = if self.is_selected {
//             SELECTED_STYLE
//         } else {
//             Style::default()
//         };
//         Text::from(format!("{:?}", self.value))
//             .style(style)
//             .render(area, buf);
//     }
// }

// struct MessageWidget<'a> {
//     message_view: DynamicMessageView<'a>,
// }

// impl MessageWidget<'_> {
//     pub fn new(message_view: DynamicMessageView) -> Self {
//         MessageWidget { message_view }
//     }

//     pub fn size(&self) -> usize {
//         self.message_view
//             .fields
//             .iter()
//             .map(|field_info| {
//                 self.message_view
//                     .get(&field_info.name)
//                     .map_or(0, |value| match value {
//                         Value::Simple(inner_value) => match inner_value {
//                             SimpleValue::Message(inner_message_view) => 1,
//                             _ => 1,
//                         },
//                         Value::Array(inner_value) => match inner_value {
//                             ArrayValue::MessageArray(inner_message_view) => 1,
//                             _ => 1,
//                         },
//                         Value::Sequence(inner_value) => match inner_value {
//                             SequenceValue::MessageSequence(inner_message_view) => 1,
//                             _ => 1,
//                         },
//                         Value::BoundedSequence(inner_value) => match inner_value {
//                             BoundedSequenceValue::MessageBoundedSequence(inner_message_view) => 1,
//                             _ => 1,
//                         },
//                     })
//             })
//             .sum()
//     }
// }

// impl Widget for MessageWidget<'_> {
//     fn render(self, area: Rect, buf: &mut Buffer) {
//         self.message_view.fields.iter().map(|field_info| {
//             let name = &field_info.name;
//             let value = self.message_view.get(&name).unwrap();
//             let field_name_widget = Text::from(format!("{}: ", name)).style(Style::default());
//             let value_widget = ValueWidget::new(value, false);

//             let [name_area, field_area] =
//                 Layout::horizontal([Constraint::Min(1), Constraint::Fill(1)]).areas(area);

//             field_name_widget.render(name_area, buf);
//             value_widget.render(field_area, buf);
//         });
//     }
// }

fn render_message(
    msg: &DynamicMessageView,
    selected_item: &[usize],
    area: Rect,
    buf: &mut Buffer,
    line_index: u16,
    column_index: u16,
) -> u16 {
    let mut new_line_index = line_index;

    for (index, field_info) in msg.fields.iter().enumerate() {
        let name = &field_info.name;
        let value = msg.get(&name).unwrap();
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

impl ValueDisplay for Value<'_> {
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
            Value::Simple(simple_value) => simple_value.display(
                selected_item,
                is_selected,
                area,
                buf,
                line_index,
                column_index,
                name,
            ),
            Value::Array(array_value) => array_value.display(
                selected_item,
                is_selected,
                area,
                buf,
                line_index,
                column_index,
                name,
            ),
            Value::Sequence(sequence_value) => sequence_value.display(
                selected_item,
                is_selected,
                area,
                buf,
                line_index,
                column_index,
                name,
            ),
            Value::BoundedSequence(bounded_sequence_value) => bounded_sequence_value.display(
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

impl ValueDisplay for SimpleValue<'_> {
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
            SimpleValue::Float(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleValue::Double(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleValue::LongDouble(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{:?}", value), // TODO: Handle LongDouble display properly
                    style,
                );
            }
            SimpleValue::Char(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("\'{}\'", value),
                    style,
                );
            }
            SimpleValue::WChar(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("\'{}\'", value),
                    style,
                );
            }
            SimpleValue::Boolean(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleValue::Octet(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleValue::Uint8(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleValue::Int8(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleValue::Uint16(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleValue::Int16(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleValue::Uint32(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleValue::Int32(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleValue::Uint64(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleValue::Int64(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("{}", value),
                    style,
                );
            }
            SimpleValue::String(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("\"{}\"", value),
                    style,
                );
            }
            SimpleValue::BoundedString(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("\"{}\"", value),
                    style,
                );
            }
            SimpleValue::WString(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("\"{}\"", value),
                    style,
                );
            }
            SimpleValue::BoundedWString(value) => {
                buf.set_string(
                    area.x + column_index + name.len() as u16 + 2,
                    area.y + line_index,
                    format!("\"{}\"", value),
                    style,
                );
            }
            SimpleValue::Message(value) => {
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

impl ValueDisplay for ArrayValue<'_> {
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
            ArrayValue::MessageArray(inner_msgs) => {
                // Render each message in the array
                let mut new_line_index = line_index;
                for inner_msg in inner_msgs.as_ref().iter() {
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

impl ValueDisplay for SequenceValue<'_> {
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
            SequenceValue::MessageSequence(inner_msgs) => {
                // Render each message in the sequence
                let mut new_line_index = line_index;
                for inner_msg in inner_msgs.as_ref().iter() {
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

impl ValueDisplay for BoundedSequenceValue<'_> {
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
            BoundedSequenceValue::MessageBoundedSequence(inner_msgs) => {
                // Render each message in the bounded sequence
                let mut new_line_index = line_index;
                for inner_msg in inner_msgs.as_ref().iter() {
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
