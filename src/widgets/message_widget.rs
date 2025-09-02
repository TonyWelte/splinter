use std::fmt;

use ratatui::{
    prelude::{BlockExt, Buffer, Rect},
    widgets::{Block, Widget},
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
}

impl<'a> Widget for MessageWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.block.as_ref().render(area, buf);
        let mut area_remaining = self.block.inner_if_some(area);

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
                .height(area_remaining.width)
                .min(area_remaining.height);
            value_widget.render(area_remaining, buf);

            // Move the area down for the next field
            area_remaining.y += widget_height;
            area_remaining.height -= widget_height;
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
    use ratatui::style::Modifier;
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
        widget.render(area, &mut buffer);

        // Check if the buffer has been modified as expected
        assert_eq!(
            buffer,
            Buffer::with_lines([
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
                "  covariance:                                     ",
                "    0         0         0         0               ",
                "    0         0         0         0               ",
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

        // let mut message_state =
        //     MessageWidgetState::new(Arc::new(Mutex::new(Some(generic_message))));
        let widget = MessageWidget::new(&generic_message);

        let area = Rect::new(0, 0, 50, 50);
        let mut buffer = Buffer::empty(area);
        widget.render(area, &mut buffer);

        // Check if the buffer has been modified as expected
        assert_eq!(
            buffer,
            Buffer::with_lines([
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
                "  covariance:                                     ",
                "    0         0         0         0               ",
                "    0         0         0         0               ",
                "    0         0         0         0               ",
                "    0         0         0         0               ",
                "    0         0         0         0               ",
                "    0         0         0         0               ",
                "    0         0         0         0               ",
                "    0         0         0         0               ",
                "    0         0         0         0               ",
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
                "  covariance:                                     ",
                "    0         0         0         0               ",
                "    0         0         0         0               ",
                "    0         0         0         0               ",
                "    0         0         0         0               ",
                "    0         0         0         0               ",
                "    0         0         0         0               ",
                "    0         0         0         0               ",
                "    0         0         0         0               ",
                "    0         0         0         0               ",
                "                                                  ",
                "                                                  ",
                "                                                  ",
            ])
        );

        let widget = MessageWidget::new(&generic_message);
        let area = Rect::new(0, 0, 52, 50);
        let mut buffer = Buffer::empty(area);
        widget.render(area, &mut buffer);

        // Check if the buffer has been modified as expected
        assert_eq!(
            buffer,
            Buffer::with_lines([
                "header:                                             ",
                "  stamp:                                            ",
                "    sec: 0                                          ",
                "    nanosec: 0                                      ",
                "  frame_id: \"\"                                      ",
                "child_frame_id: \"\"                                  ",
                "pose:                                               ",
                "  pose:                                             ",
                "    position:                                       ",
                "      x: 0                                          ",
                "      y: 0                                          ",
                "      z: 0                                          ",
                "    orientation:                                    ",
                "      x: 0                                          ",
                "      y: 0                                          ",
                "      z: 0                                          ",
                "      w: 1                                          ",
                "  covariance:                                       ",
                "    0         0         0         0                 ",
                "    0         0         0         0                 ",
                "    0         0         0         0                 ",
                "    0         0         0         0                 ",
                "    0         0         0         0                 ",
                "    0         0         0         0                 ",
                "    0         0         0         0                 ",
                "    0         0         0         0                 ",
                "    0         0         0         0                 ",
                "twist:                                              ",
                "  twist:                                            ",
                "    linear:                                         ",
                "      x: 0                                          ",
                "      y: 0                                          ",
                "      z: 0                                          ",
                "    angular:                                        ",
                "      x: 0                                          ",
                "      y: 0                                          ",
                "      z: 0                                          ",
                "  covariance:                                       ",
                "    0         0         0         0                 ",
                "    0         0         0         0                 ",
                "    0         0         0         0                 ",
                "    0         0         0         0                 ",
                "    0         0         0         0                 ",
                "    0         0         0         0                 ",
                "    0         0         0         0                 ",
                "    0         0         0         0                 ",
                "    0         0         0         0                 ",
                "                                                    ",
                "                                                    ",
                "                                                    ",
            ])
        );
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
        widget.render(area, &mut buffer);

        // Check if the buffer has been modified as expected
        assert_eq!(
            buffer,
            Buffer::with_lines([
                "┌Message Widget──────────────────────────────────┐",
                "│header:                                         │",
                "│  stamp:                                        │",
                "│    sec: 0                                      │",
                "│    nanosec: 0                                  │",
                "│  frame_id: \"\"                                  │",
                "│child_frame_id: \"\"                              │",
                "│pose:                                           │",
                "│  pose:                                         │",
                "│    position:                                   │",
                "│      x: 0                                      │",
                "│      y: 0                                      │",
                "│      z: 0                                      │",
                "│    orientation:                                │",
                "│      x: 0                                      │",
                "│      y: 0                                      │",
                "│      z: 0                                      │",
                "│      w: 1                                      │",
                "│  covariance:                                   │",
                "└────────────────────────────────────────────────┘",
            ])
        )
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
        widget.render(area, &mut buffer);

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
            "  covariance:                                     ",
            "    0         0         0         0               ",
            "    0         0         0         0               ",
        ]);
        expected_buffer.set_style(
            Rect {
                x: 0,
                y: 0,
                width: 6,
                height: 1,
            },
            SELECTED_STYLE,
        );
        expected_buffer.set_style(
            Rect {
                x: 12,
                y: 4,
                width: 2,
                height: 1,
            },
            SELECTED_STYLE,
        );

        // Check if the buffer has been modified as expected
        assert_eq!(buffer, expected_buffer);

        let widget = MessageWidget::new(&generic_message).with_selection(&[2, 0, 1, 1]); // Select pose.pose.orientation.y field

        let area = Rect::new(0, 0, 50, 20);
        let mut buffer = Buffer::empty(area);
        widget.render(area, &mut buffer);

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
            "  covariance:                                     ",
            "    0         0         0         0               ",
            "    0         0         0         0               ",
        ]);
        expected_buffer.set_style(
            Rect {
                x: 0,
                y: 6,
                width: 4,
                height: 1,
            },
            SELECTED_STYLE,
        );
        expected_buffer.set_style(
            Rect {
                x: 2,
                y: 7,
                width: 4,
                height: 1,
            },
            SELECTED_STYLE,
        );
        expected_buffer.set_style(
            Rect {
                x: 4,
                y: 12,
                width: 11,
                height: 1,
            },
            SELECTED_STYLE,
        );
        expected_buffer.set_style(
            Rect {
                x: 9,
                y: 14,
                width: 1,
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
        widget.render(area, &mut buffer);

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
            "  covariance:                                     ",
            "    0         0         0         0               ",
            "    0         0         0         0               ",
            "    0         0         0         0               ",
            "    0         0         0         0               ",
            "    0         0         0         0               ",
            "    0         0         0         0               ",
            "    0         0         0         0               ",
            "    0         0         0         0               ",
            "    0         0         0         0               ",
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
            "  covariance:                                     ",
            "    0         0         0         0               ",
            "    0         0         0         0               ",
            "    0         0         0         0               ",
            "    0         0         0         0               ",
            "    0         0         0         0               ",
            "    0         0         0         0               ",
            "    0         0         0         0               ",
            "    0         0         0         0               ",
            "    0         0         0         0               ",
            "                                                  ",
            "                                                  ",
            "                                                  ",
        ]);

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
                width: 12,
                height: 1,
            },
            SELECTED_STYLE,
        );
        expected_buffer.set_style(
            Rect {
                x: 14,
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
        widget.render(area, &mut buffer);

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
            "  covariance:                                     ",
            "    0         0         0         0               ",
            "    0         0         0         0               ",
        ]);
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
                x: 9,
                y: 10,
                width: 7,
                height: 1,
            },
            SELECTED_STYLE.add_modifier(Modifier::SLOW_BLINK),
        );

        // Check if the buffer has been modified as expected_buffer
        assert_eq!(buffer, expected_buffer);
    }
}
