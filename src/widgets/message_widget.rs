use std::fmt;

use ratatui::{
    prelude::{BlockExt, Buffer, Rect},
    widgets::{Block, StatefulWidget, Widget},
};

use crate::common::generic_message::{
    ArrayField, BoundedSequenceField, GenericMessage, SequenceField, SimpleField,
};

mod array_widget;
mod bounded_sequence_widget;
mod sequence_widget;
mod value_widget;

use array_widget::ArrayWidget;
use bounded_sequence_widget::BoundedSequenceWidget;
use sequence_widget::SequenceWidget;
use value_widget::ValueWidget;

pub struct MessageWidget<'a> {
    message: &'a GenericMessage,
    selection: Option<&'a [usize]>,
    edit: Option<&'a str>,

    block: Option<Block<'a>>,
}

pub struct MessageWidgetState {
    scroll_offset: u16,
    auto_scroll: bool,
}

impl MessageWidgetState {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
            auto_scroll: true,
        }
    }

    pub fn auto_scroll(mut self) -> Self {
        self.auto_scroll = true;
        self
    }
}

impl<'a> MessageWidget<'a> {
    pub fn new(message: &'a GenericMessage) -> Self {
        Self {
            message,
            selection: None,
            edit: None,
            block: None,
        }
    }

    pub fn with_selection(mut self, selection: &'a [usize]) -> Self {
        self.selection = Some(selection);
        self
    }

    pub fn with_edit(mut self, edit: &'a str) -> Self {
        self.edit = Some(edit);
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn height(&self, width: u16) -> u16 {
        // TODO: Don't like having the calculation of the height of the displayed message separated from the rendering logic.
        let mut height = 0;
        for (name, value) in self.message.iter() {
            height += ValueWidget::new(name, value).height(width);
        }
        height
    }

    pub fn selection_height(&self, width: u16) -> u16 {
        if let Some(selection) = self.selection {
            if selection.is_empty() {
                return 0;
            }

            let mut height = 0;
            for (i, (name, value)) in self.message.iter().enumerate() {
                if i != selection[0] {
                    height += ValueWidget::new(name, value).height(width);
                } else {
                    height += ValueWidget::new(name, value)
                        .with_selection(&selection[1..])
                        .selection_height(width);
                    break;
                }
            }
            return height;
        }
        0
    }
}

impl<'a> Widget for MessageWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = MessageWidgetState::new();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

impl<'a> StatefulWidget for MessageWidget<'a> {
    type State = MessageWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.block.as_ref().render(area, buf);
        let area_remaining = self.block.inner_if_some(area);

        // At the end otherwise would affect stateless rendering
        if state.auto_scroll {
            let selection_height = self.selection_height(area_remaining.width);
            state.scroll_offset = selection_height
                .saturating_sub(area_remaining.height / 2)
                .min(
                    self.height(area_remaining.width)
                        .saturating_sub(area_remaining.height),
                );
        }

        let mut extended_area = Rect {
            x: 0,
            y: 0,
            width: area_remaining.width,
            height: area_remaining.height + state.scroll_offset,
        };

        let mut extended_buffer = Buffer::empty(extended_area);

        for (i, (name, value)) in self.message.iter().enumerate() {
            let mut value_widget = ValueWidget::new(name, value);
            if let Some(selection) = self.selection {
                if !selection.is_empty() && selection[0] == i {
                    value_widget = value_widget.with_selection(&selection[1..]);
                    if let Some(edit) = self.edit {
                        value_widget = value_widget.with_edit(edit);
                    }
                }
            }

            let widget_height = value_widget
                .height(extended_area.width)
                .min(extended_area.height);
            value_widget.render(extended_area, &mut extended_buffer);

            // Move the area down for the next field
            extended_area.y += widget_height;
            extended_area.height -= widget_height;
        }

        // Copy the visible part of the extended buffer to the actual buffer
        for y in 0..area_remaining.height {
            for x in 0..area_remaining.width {
                if let Some(cell) = buf.cell_mut((area_remaining.x + x, area_remaining.y + y)) {
                    *cell = extended_buffer
                        .cell((x, y + state.scroll_offset))
                        .unwrap()
                        .clone();
                }
            }
        }
    }
}

impl fmt::Display for SimpleField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimpleField::Float(value) => write!(f, "{}", value),
            SimpleField::Double(value) => write!(f, "{}", value),
            SimpleField::LongDouble(value) => write!(f, "{:?}", value), // TODO: Handle LongDouble display properly
            SimpleField::Char(value) => write!(f, "'{}'", value),
            SimpleField::WChar(value) => write!(f, "'{}'", value),
            SimpleField::Boolean(value) => write!(f, "{}", value),
            SimpleField::Octet(value) => write!(f, "{}", value),
            SimpleField::Uint8(value) => write!(f, "{}", value),
            SimpleField::Int8(value) => write!(f, "{}", value),
            SimpleField::Uint16(value) => write!(f, "{}", value),
            SimpleField::Int16(value) => write!(f, "{}", value),
            SimpleField::Uint32(value) => write!(f, "{}", value),
            SimpleField::Int32(value) => write!(f, "{}", value),
            SimpleField::Uint64(value) => write!(f, "{}", value),
            SimpleField::Int64(value) => write!(f, "{}", value),
            SimpleField::String(value) => write!(f, "\"{}\"", value),
            SimpleField::BoundedString(value) => write!(f, "\"{}\"", value),
            SimpleField::WString(value) => write!(f, "\"{}\"", value),
            SimpleField::BoundedWString(value) => write!(f, "\"{}\"", value),
            SimpleField::Message(_) => write!(f, "<message>"),
        }
    }
}

trait AsStrVec {
    fn as_str_iter(&self) -> Vec<String>;
}

impl AsStrVec for ArrayField {
    fn as_str_iter(&self) -> Vec<String> {
        match self {
            ArrayField::Float(values) => values.iter().map(|v| format!("{}", v)).collect(),
            ArrayField::Double(values) => values.iter().map(|v| format!("{}", v)).collect(),
            ArrayField::LongDouble(_) => vec![],
            ArrayField::Char(values) => values.iter().map(|v| format!("{}", v)).collect(),
            ArrayField::WChar(values) => values.iter().map(|v| format!("{}", v)).collect(),
            ArrayField::Boolean(values) => values.iter().map(|v| format!("{}", v)).collect(),
            ArrayField::Octet(values) => values.iter().map(|v| format!("{}", v)).collect(),
            ArrayField::Uint8(values) => values.iter().map(|v| format!("{}", v)).collect(),
            ArrayField::Int8(values) => values.iter().map(|v| format!("{}", v)).collect(),
            ArrayField::Uint16(values) => values.iter().map(|v| format!("{}", v)).collect(),
            ArrayField::Int16(values) => values.iter().map(|v| format!("{}", v)).collect(),
            ArrayField::Uint32(values) => values.iter().map(|v| format!("{}", v)).collect(),
            ArrayField::Int32(values) => values.iter().map(|v| format!("{}", v)).collect(),
            ArrayField::Uint64(values) => values.iter().map(|v| format!("{}", v)).collect(),
            ArrayField::Int64(values) => values.iter().map(|v| format!("{}", v)).collect(),
            ArrayField::String(values) => values.iter().map(|v| format!("\"{}\"", v)).collect(),
            ArrayField::BoundedString(values) => {
                values.iter().map(|v| format!("\"{}\"", v)).collect()
            }
            ArrayField::WString(values) => values.iter().map(|v| format!("\"{}\"", v)).collect(),
            ArrayField::BoundedWString(values) => {
                values.iter().map(|v| format!("\"{}\"", v)).collect()
            }
            _ => vec![],
        }
    }
}

impl AsStrVec for SequenceField {
    fn as_str_iter(&self) -> Vec<String> {
        match self {
            SequenceField::Float(values) => values.iter().map(|v| format!("{}", v)).collect(),
            SequenceField::Double(values) => values.iter().map(|v| format!("{}", v)).collect(),
            SequenceField::LongDouble(_) => vec![],
            SequenceField::Char(values) => values.iter().map(|v| format!("{}", v)).collect(),
            SequenceField::WChar(values) => values.iter().map(|v| format!("{}", v)).collect(),
            SequenceField::Boolean(values) => values.iter().map(|v| format!("{}", v)).collect(),
            SequenceField::Octet(values) => values.iter().map(|v| format!("{}", v)).collect(),
            SequenceField::Uint8(values) => values.iter().map(|v| format!("{}", v)).collect(),
            SequenceField::Int8(values) => values.iter().map(|v| format!("{}", v)).collect(),
            SequenceField::Uint16(values) => values.iter().map(|v| format!("{}", v)).collect(),
            SequenceField::Int16(values) => values.iter().map(|v| format!("{}", v)).collect(),
            SequenceField::Uint32(values) => values.iter().map(|v| format!("{}", v)).collect(),
            SequenceField::Int32(values) => values.iter().map(|v| format!("{}", v)).collect(),
            SequenceField::Uint64(values) => values.iter().map(|v| format!("{}", v)).collect(),
            SequenceField::Int64(values) => values.iter().map(|v| format!("{}", v)).collect(),
            SequenceField::String(values) => values.iter().map(|v| format!("\"{}\"", v)).collect(),
            SequenceField::BoundedString(values) => {
                values.iter().map(|v| format!("\"{}\"", v)).collect()
            }
            SequenceField::WString(values) => values.iter().map(|v| format!("\"{}\"", v)).collect(),
            SequenceField::BoundedWString(values) => {
                values.iter().map(|v| format!("\"{}\"", v)).collect()
            }
            SequenceField::Message(_) => {
                panic!("Sequence of messages cannot be converted to string vector")
            }
        }
    }
}

impl AsStrVec for BoundedSequenceField {
    fn as_str_iter(&self) -> Vec<String> {
        match self {
            BoundedSequenceField::Float(values, _) => {
                values.iter().map(|v| format!("{}", v)).collect()
            }
            BoundedSequenceField::Double(values, _) => {
                values.iter().map(|v| format!("{}", v)).collect()
            }
            BoundedSequenceField::LongDouble(_, _) => vec![],
            BoundedSequenceField::Char(values, _) => {
                values.iter().map(|v| format!("{}", v)).collect()
            }
            BoundedSequenceField::WChar(values, _) => {
                values.iter().map(|v| format!("{}", v)).collect()
            }
            BoundedSequenceField::Boolean(values, _) => {
                values.iter().map(|v| format!("{}", v)).collect()
            }
            BoundedSequenceField::Octet(values, _) => {
                values.iter().map(|v| format!("{}", v)).collect()
            }
            BoundedSequenceField::Uint8(values, _) => {
                values.iter().map(|v| format!("{}", v)).collect()
            }
            BoundedSequenceField::Int8(values, _) => {
                values.iter().map(|v| format!("{}", v)).collect()
            }
            BoundedSequenceField::Uint16(values, _) => {
                values.iter().map(|v| format!("{}", v)).collect()
            }
            BoundedSequenceField::Int16(values, _) => {
                values.iter().map(|v| format!("{}", v)).collect()
            }
            BoundedSequenceField::Uint32(values, _) => {
                values.iter().map(|v| format!("{}", v)).collect()
            }
            BoundedSequenceField::Int32(values, _) => {
                values.iter().map(|v| format!("{}", v)).collect()
            }
            BoundedSequenceField::Uint64(values, _) => {
                values.iter().map(|v| format!("{}", v)).collect()
            }
            BoundedSequenceField::Int64(values, _) => {
                values.iter().map(|v| format!("{}", v)).collect()
            }
            BoundedSequenceField::String(values, _) => {
                values.iter().map(|v| format!("\"{}\"", v)).collect()
            }
            BoundedSequenceField::BoundedString(values, _) => {
                values.iter().map(|v| format!("\"{}\"", v)).collect()
            }
            BoundedSequenceField::WString(values, _) => {
                values.iter().map(|v| format!("\"{}\"", v)).collect()
            }
            BoundedSequenceField::BoundedWString(values, _) => {
                values.iter().map(|v| format!("\"{}\"", v)).collect()
            }
            BoundedSequenceField::Message(_, _) => {
                panic!("Sequence of messages cannot be converted to string vector")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::style::Color;
    use ratatui::style::Modifier;
    use ratatui::style::Style;
    use ratatui::style::Stylize;
    use ratatui::text::Line;
    use rclrs::DynamicMessage;
    use rclrs::MessageTypeName;

    use crate::common::style::SELECTED_STYLE;

    use super::*;

    #[test]
    fn test_message_widget_render() {
        let message_type = MessageTypeName {
            package_name: "nav_msgs".to_owned(),
            type_name: "Odometry".to_owned(),
        };
        let msg = DynamicMessage::new(message_type).unwrap();
        let generic_message = GenericMessage::from(msg.view());

        // let mut message_state =
        //     MessageWidgetState::new(Arc::new(Mutex::new(Some(generic_message))));
        let widget = MessageWidget::new(&generic_message);

        let area = Rect::new(0, 0, 50, 20);
        let mut buffer = Buffer::empty(area);
        Widget::render(widget, area, &mut buffer);

        // Check if the buffer has been modified as expected
        assert_eq!(
            buffer,
            Buffer::with_lines([
                "header:                                           ".into(),
                "  stamp:                                          ".into(),
                "    sec: 0                                        ".into(),
                "    nanosec: 0                                    ".into(),
                "  frame_id: \"\"                                    ".into(),
                "child_frame_id: \"\"                                ".into(),
                "pose:                                             ".into(),
                "  pose:                                           ".into(),
                "    position:                                     ".into(),
                "      x: 0                                        ".into(),
                "      y: 0                                        ".into(),
                "      z: 0                                        ".into(),
                "    orientation:                                  ".into(),
                "      x: 0                                        ".into(),
                "      y: 0                                        ".into(),
                "      z: 0                                        ".into(),
                "      w: 1                                        ".into(),
                Line::from_iter([
                    "  covariance:".into(),
                    " 36 elements".fg(Color::DarkGray),
                    "                         ".into()
                ]),
                "  0         0         0         0                 ".into(),
                "  0         0         0         0                 ".into(),
            ])
        )
    }

    #[test]
    fn test_message_widget_render_full() {
        let message_type = MessageTypeName {
            package_name: "nav_msgs".to_owned(),
            type_name: "Odometry".to_owned(),
        };
        let msg = DynamicMessage::new(message_type).unwrap();
        let generic_message = GenericMessage::from(msg.view());

        let widget = MessageWidget::new(&generic_message);
        let area = Rect::new(0, 0, 50, 50);
        let mut buffer = Buffer::empty(area);
        Widget::render(widget, area, &mut buffer);

        let expected = Buffer::with_lines([
            "header:                                           ".into(),
            "  stamp:                                          ".into(),
            "    sec: 0                                        ".into(),
            "    nanosec: 0                                    ".into(),
            "  frame_id: \"\"                                    ".into(),
            "child_frame_id: \"\"                                ".into(),
            "pose:                                             ".into(),
            "  pose:                                           ".into(),
            "    position:                                     ".into(),
            "      x: 0                                        ".into(),
            "      y: 0                                        ".into(),
            "      z: 0                                        ".into(),
            "    orientation:                                  ".into(),
            "      x: 0                                        ".into(),
            "      y: 0                                        ".into(),
            "      z: 0                                        ".into(),
            "      w: 1                                        ".into(),
            Line::from_iter([
                "  covariance:".fg(Color::Reset),
                " 36 elements".fg(Color::DarkGray),
                "                       ".into(),
            ]),
            "  0         0         0         0                 ".into(),
            "  0         0         0         0                 ".into(),
            "  0         0         0         0                 ".into(),
            "  0         0         0         0                 ".into(),
            "  0         0         0         0                 ".into(),
            "  0         0         0         0                 ".into(),
            "  0         0         0         0                 ".into(),
            "  0         0         0         0                 ".into(),
            "  0         0         0         0                 ".into(),
            "twist:                                            ".into(),
            "  twist:                                          ".into(),
            "    linear:                                       ".into(),
            "      x: 0                                        ".into(),
            "      y: 0                                        ".into(),
            "      z: 0                                        ".into(),
            "    angular:                                      ".into(),
            "      x: 0                                        ".into(),
            "      y: 0                                        ".into(),
            "      z: 0                                        ".into(),
            Line::from_iter([
                "  covariance:".fg(Color::Reset),
                " 36 elements".fg(Color::DarkGray),
                "                       ".into(),
            ]),
            "  0         0         0         0                 ".into(),
            "  0         0         0         0                 ".into(),
            "  0         0         0         0                 ".into(),
            "  0         0         0         0                 ".into(),
            "  0         0         0         0                 ".into(),
            "  0         0         0         0                 ".into(),
            "  0         0         0         0                 ".into(),
            "  0         0         0         0                 ".into(),
            "  0         0         0         0                 ".into(),
            "                                                  ".into(),
            "                                                  ".into(),
            "                                                  ".into(),
        ]);

        // Check if the buffer has been modified as expected
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_message_widget_render_bordered() {
        let message_type = MessageTypeName {
            package_name: "nav_msgs".to_owned(),
            type_name: "Odometry".to_owned(),
        };
        let msg = DynamicMessage::new(message_type).unwrap();
        let generic_message = GenericMessage::from(msg.view());

        let widget =
            MessageWidget::new(&generic_message).block(Block::bordered().title("Message Widget"));

        let area = Rect::new(0, 0, 50, 20);
        let mut buffer = Buffer::empty(area);
        Widget::render(widget, area, &mut buffer);

        let expected = Buffer::with_lines([
            Line::raw("┌Message Widget──────────────────────────────────┐"),
            Line::raw("│header:                                         │"),
            Line::raw("│  stamp:                                        │"),
            Line::raw("│    sec: 0                                      │"),
            Line::raw("│    nanosec: 0                                  │"),
            Line::raw("│  frame_id: \"\"                                  │"),
            Line::raw("│child_frame_id: \"\"                              │"),
            Line::raw("│pose:                                           │"),
            Line::raw("│  pose:                                         │"),
            Line::raw("│    position:                                   │"),
            Line::raw("│      x: 0                                      │"),
            Line::raw("│      y: 0                                      │"),
            Line::raw("│      z: 0                                      │"),
            Line::raw("│    orientation:                                │"),
            Line::raw("│      x: 0                                      │"),
            Line::raw("│      y: 0                                      │"),
            Line::raw("│      z: 0                                      │"),
            Line::raw("│      w: 1                                      │"),
            Line::from_iter([
                "│  covariance:".fg(Color::Reset),
                " 36 elements".fg(Color::DarkGray),
                "                       │".into(),
            ]),
            Line::raw("└────────────────────────────────────────────────┘"),
        ]);

        // Check if the buffer has been modified as expected
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_message_widget_render_with_selection() {
        let message_type = MessageTypeName {
            package_name: "nav_msgs".to_owned(),
            type_name: "Odometry".to_owned(),
        };
        let msg = DynamicMessage::new(message_type).unwrap();
        let generic_message = GenericMessage::from(msg.view());

        let widget = MessageWidget::new(&generic_message).with_selection(&[0, 1]); // Select frame_id field

        let area = Rect::new(0, 0, 50, 20);
        let mut buffer = Buffer::empty(area);
        Widget::render(widget, area, &mut buffer);

        let mut expected_buffer = Buffer::with_lines([
            "header:                                           ",
            "  stamp:                                          ",
            "    sec: 0                                        ",
            "    nanosec: 0                                    ",
            "  frame_id: \"\"                                    ",
            "child_frame_id: \"\"                                ",
            "pose:                                             ",
            "  pose:                                           ",
            "    position:                                     ",
            "      x: 0                                        ",
            "      y: 0                                        ",
            "      z: 0                                        ",
            "    orientation:                                  ",
            "      x: 0                                        ",
            "      y: 0                                        ",
            "      z: 0                                        ",
            "      w: 1                                        ",
            "  covariance: 36 elements                         ",
            "  0         0         0         0                 ",
            "  0         0         0         0                 ",
        ]);
        // Covariance element count
        expected_buffer.set_style(
            Rect {
                x: 13,
                y: 17,
                width: 12,
                height: 1,
            },
            Style::default().fg(Color::DarkGray),
        );
        expected_buffer.set_style(
            Rect {
                x: 0,
                y: 0,
                width: 8,
                height: 1,
            },
            SELECTED_STYLE,
        );
        expected_buffer.set_style(
            Rect {
                x: 2,
                y: 4,
                width: 12,
                height: 1,
            },
            SELECTED_STYLE,
        );

        // Check if the buffer has been modified as expected
        assert_eq!(buffer, expected_buffer);

        let widget = MessageWidget::new(&generic_message).with_selection(&[2, 0, 1, 1]); // Select pose.pose.orientation.y field

        let area = Rect::new(0, 0, 50, 20);
        let mut buffer = Buffer::empty(area);
        Widget::render(widget, area, &mut buffer);

        let mut expected_buffer = Buffer::with_lines([
            "header:                                           ",
            "  stamp:                                          ",
            "    sec: 0                                        ",
            "    nanosec: 0                                    ",
            "  frame_id: \"\"                                    ",
            "child_frame_id: \"\"                                ",
            "pose:                                             ",
            "  pose:                                           ",
            "    position:                                     ",
            "      x: 0                                        ",
            "      y: 0                                        ",
            "      z: 0                                        ",
            "    orientation:                                  ",
            "      x: 0                                        ",
            "      y: 0                                        ",
            "      z: 0                                        ",
            "      w: 1                                        ",
            "  covariance: 36 elements                         ",
            "  0         0         0         0                 ",
            "  0         0         0         0                 ",
        ]);
        // Covariance element count
        expected_buffer.set_style(
            Rect {
                x: 13,
                y: 17,
                width: 12,
                height: 1,
            },
            Style::default().fg(Color::DarkGray),
        );
        // Select pose field
        expected_buffer.set_style(
            Rect {
                x: 0,
                y: 6,
                width: 6,
                height: 1,
            },
            SELECTED_STYLE,
        );
        // Select pose.pose field
        expected_buffer.set_style(
            Rect {
                x: 2,
                y: 7,
                width: 6,
                height: 1,
            },
            SELECTED_STYLE,
        );
        // Select pose.pose.orientation field
        expected_buffer.set_style(
            Rect {
                x: 4,
                y: 12,
                width: 13,
                height: 1,
            },
            SELECTED_STYLE,
        );
        // Select pose.pose.orientation.y field
        expected_buffer.set_style(
            Rect {
                x: 6,
                y: 14,
                width: 4,
                height: 1,
            },
            SELECTED_STYLE,
        );

        // Check if the buffer has been modified as expected
        assert_eq!(buffer, expected_buffer);
    }

    #[test]
    fn test_message_widget_render_full_with_selection() {
        let message_type = MessageTypeName {
            package_name: "nav_msgs".to_owned(),
            type_name: "Odometry".to_owned(),
        };
        let msg = DynamicMessage::new(message_type).unwrap();
        let generic_message = GenericMessage::from(msg.view());

        // let mut message_state =
        //     MessageWidgetState::new(Arc::new(Mutex::new(Some(generic_message))));
        let widget = MessageWidget::new(&generic_message).with_selection(&[2, 1, 5]); // Select pose.covariance.5 field

        let area = Rect::new(0, 0, 50, 50);
        let mut buffer = Buffer::empty(area);
        Widget::render(widget, area, &mut buffer);

        let mut expected_buffer = Buffer::with_lines([
            "header:                                           ",
            "  stamp:                                          ",
            "    sec: 0                                        ",
            "    nanosec: 0                                    ",
            "  frame_id: \"\"                                    ",
            "child_frame_id: \"\"                                ",
            "pose:                                             ",
            "  pose:                                           ",
            "    position:                                     ",
            "      x: 0                                        ",
            "      y: 0                                        ",
            "      z: 0                                        ",
            "    orientation:                                  ",
            "      x: 0                                        ",
            "      y: 0                                        ",
            "      z: 0                                        ",
            "      w: 1                                        ",
            "  covariance: 36 elements                         ",
            "  0         0         0         0                 ",
            "  0         0         0         0                 ",
            "  0         0         0         0                 ",
            "  0         0         0         0                 ",
            "  0         0         0         0                 ",
            "  0         0         0         0                 ",
            "  0         0         0         0                 ",
            "  0         0         0         0                 ",
            "  0         0         0         0                 ",
            "twist:                                            ",
            "  twist:                                          ",
            "    linear:                                       ",
            "      x: 0                                        ",
            "      y: 0                                        ",
            "      z: 0                                        ",
            "    angular:                                      ",
            "      x: 0                                        ",
            "      y: 0                                        ",
            "      z: 0                                        ",
            "  covariance: 36 elements                         ",
            "  0         0         0         0                 ",
            "  0         0         0         0                 ",
            "  0         0         0         0                 ",
            "  0         0         0         0                 ",
            "  0         0         0         0                 ",
            "  0         0         0         0                 ",
            "  0         0         0         0                 ",
            "  0         0         0         0                 ",
            "  0         0         0         0                 ",
            "                                                  ",
            "                                                  ",
            "                                                  ",
        ]);

        // Covariance element count
        expected_buffer.set_style(
            Rect {
                x: 13,
                y: 17,
                width: 12,
                height: 1,
            },
            Style::default().fg(Color::DarkGray),
        );
        // Covariance element count
        expected_buffer.set_style(
            Rect {
                x: 13,
                y: 37,
                width: 12,
                height: 1,
            },
            Style::default().fg(Color::DarkGray),
        );
        expected_buffer.set_style(
            Rect {
                x: 0,
                y: 6,
                width: 6,
                height: 1,
            },
            SELECTED_STYLE,
        );
        expected_buffer.set_style(
            Rect {
                x: 2,
                y: 17,
                width: 11,
                height: 1,
            },
            SELECTED_STYLE,
        );
        expected_buffer.set_style(
            Rect {
                x: 12,
                y: 19,
                width: 1,
                height: 1,
            },
            SELECTED_STYLE,
        );

        // Check if the buffer has been modified as expected
        assert_eq!(buffer, expected_buffer)
    }

    #[test]
    fn test_message_widget_render_with_edit() {
        let message_type = MessageTypeName {
            package_name: "nav_msgs".to_owned(),
            type_name: "Odometry".to_owned(),
        };
        let msg = DynamicMessage::new(message_type).unwrap();
        let generic_message = GenericMessage::from(msg.view());

        // let mut message_state =
        //     MessageWidgetState::new(Arc::new(Mutex::new(Some(generic_message))));
        let widget = MessageWidget::new(&generic_message)
            .with_selection(&[2, 0, 0, 1]) // Select pose.pose.position.y field
            .with_edit("chicken");

        let area = Rect::new(0, 0, 50, 20);
        let mut buffer = Buffer::empty(area);
        Widget::render(widget, area, &mut buffer);

        let mut expected_buffer = Buffer::with_lines([
            "header:                                           ",
            "  stamp:                                          ",
            "    sec: 0                                        ",
            "    nanosec: 0                                    ",
            "  frame_id: \"\"                                    ",
            "child_frame_id: \"\"                                ",
            "pose:                                             ",
            "  pose:                                           ",
            "    position:                                     ",
            "      x: 0                                        ",
            "      y: chicken                                  ",
            "      z: 0                                        ",
            "    orientation:                                  ",
            "      x: 0                                        ",
            "      y: 0                                        ",
            "      z: 0                                        ",
            "      w: 1                                        ",
            "  covariance: 36 elements                         ",
            "  0         0         0         0                 ",
            "  0         0         0         0                 ",
        ]);
        // Covariance element count
        expected_buffer.set_style(
            Rect {
                x: 13,
                y: 17,
                width: 12,
                height: 1,
            },
            Style::default().fg(Color::DarkGray),
        );
        expected_buffer.set_style(
            Rect {
                x: 0,
                y: 6,
                width: 6,
                height: 1,
            },
            SELECTED_STYLE,
        );
        expected_buffer.set_style(
            Rect {
                x: 2,
                y: 7,
                width: 6,
                height: 1,
            },
            SELECTED_STYLE,
        );
        expected_buffer.set_style(
            Rect {
                x: 4,
                y: 8,
                width: 10,
                height: 1,
            },
            SELECTED_STYLE,
        );
        expected_buffer.set_style(
            Rect {
                x: 6,
                y: 10,
                width: 3,
                height: 1,
            },
            SELECTED_STYLE,
        );
        expected_buffer.set_style(
            Rect {
                x: 9,
                y: 10,
                width: 7,
                height: 1,
            },
            SELECTED_STYLE
                .add_modifier(Modifier::SLOW_BLINK)
                .fg(Color::Red),
        );

        // Check if the buffer has been modified as expected_buffer
        assert_eq!(buffer, expected_buffer);
    }

    #[test]
    fn test_message_widget_selection_height() {
        let message_type = MessageTypeName {
            package_name: "nav_msgs".to_owned(),
            type_name: "Odometry".to_owned(),
        };
        let msg = DynamicMessage::new(message_type).unwrap();
        let generic_message = GenericMessage::from(msg.view());

        let widget = MessageWidget::new(&generic_message);
        assert_eq!(widget.selection_height(50), 0); // No selection

        let widget = MessageWidget::new(&generic_message).with_selection(&[0]);
        assert_eq!(widget.selection_height(50), 1); // Select header field

        let widget = MessageWidget::new(&generic_message).with_selection(&[0, 0]);
        assert_eq!(widget.selection_height(50), 2); // Select header.stamp field

        let widget = MessageWidget::new(&generic_message).with_selection(&[2, 0, 1, 1]);
        assert_eq!(widget.selection_height(50), 15); // Select pose.pose.orientation.y field

        let widget = MessageWidget::new(&generic_message).with_selection(&[2, 1, 5]);
        assert_eq!(widget.selection_height(50), 27); // Select pose.covariance.5 field

        let widget = MessageWidget::new(&generic_message).with_selection(&[2, 1, 35]);
        assert_eq!(widget.selection_height(50), 27); // Select pose.covariance.35 field
    }
}
