use std::fmt;

use ratatui::{
    prelude::{BlockExt, Buffer, Rect},
    style::Style,
    widgets::{Block, Widget},
};

use crate::common::{
    generic_message::{
        ArrayField, BoundedSequenceField, GenericField, GenericMessage, SequenceField, SimpleField,
    },
    style::SELECTED_STYLE,
};

pub struct MessageWidget<'a> {
    message: &'a GenericMessage,
    selection: &'a [usize],

    block: Option<Block<'a>>,
}

impl<'a> MessageWidget<'a> {
    pub fn new(message: &'a GenericMessage) -> Self {
        Self {
            message,
            selection: &[],
            block: None,
        }
    }

    pub fn with_selection(mut self, selection: &'a [usize]) -> Self {
        self.selection = selection;
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn height(&self) -> u16 {
        // TODO: Don't like having the calculation of the height of the displayed message separated from the rendering logic.
        let mut height = 0;
        for (_, value) in self.message.iter() {
            height += 1; // For the field name
            match value {
                GenericField::Simple(SimpleField::Message(inner_message)) => {
                    height += MessageWidget::new(inner_message).height();
                }
                GenericField::Array(ArrayField::Message(inner_messages)) => {
                    for inner_message in inner_messages {
                        height += MessageWidget::new(inner_message).height();
                    }
                }
                GenericField::Sequence(SequenceField::Message(inner_messages)) => {
                    for inner_message in inner_messages {
                        height += MessageWidget::new(inner_message).height();
                    }
                }
                GenericField::BoundedSequence(BoundedSequenceField::Message(inner_messages)) => {
                    for inner_message in inner_messages {
                        height += MessageWidget::new(inner_message).height();
                    }
                }
                _ => {}
            }
        }
        height
    }
}

impl<'a> Widget for MessageWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.block.as_ref().render(area, buf);
        let mut area_remaining = self.block.inner_if_some(area);

        for (i, (name, value)) in self.message.iter().enumerate() {
            let value_widget = if !self.selection.is_empty() && self.selection[0] == i {
                ValueWidget::new(name, value).with_selection(&self.selection[1..])
            } else {
                ValueWidget::new(name, value)
            };

            let widget_height = value_widget.height().min(area_remaining.height);
            value_widget.render(area_remaining, buf);

            // Move the area down for the next field
            area_remaining.y += widget_height;
            area_remaining.height -= widget_height;
        }
    }
}

struct ValueWidget<'a> {
    name: &'a str,
    value: &'a GenericField,
    selection: Option<&'a [usize]>,
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

impl<'a> ValueWidget<'a> {
    pub fn new(name: &'a str, value: &'a GenericField) -> Self {
        Self {
            name,
            value,
            selection: None,
        }
    }

    pub fn with_selection(mut self, selection: &'a [usize]) -> Self {
        self.selection = Some(selection);
        self
    }

    pub fn height(&self) -> u16 {
        match &self.value {
            GenericField::Simple(SimpleField::Message(inner_message)) => {
                MessageWidget::new(inner_message).height() + 1 // +1 for the field name
            }
            GenericField::Simple(_)
            | GenericField::Array(_)
            | GenericField::Sequence(_)
            | GenericField::BoundedSequence(_) => 1,
        }
    }
}

impl<'a> Widget for ValueWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = if self.selection.is_some() {
            SELECTED_STYLE
        } else {
            Style::default()
        };

        if (area.height == 0 || area.width == 0) {
            return; // Nothing to render if the area is empty
        }
        buf.set_stringn(
            area.x,
            area.y,
            format!("{}: ", self.name),
            area.width as usize - area.x as usize,
            Style::default(),
        );
        let area_right = Rect::new(
            area.x + self.name.len() as u16 + 2,
            area.y,
            area.width - self.name.len() as u16 - 2,
            area.height,
        );
        let area_under = Rect::new(
            area.x + 2,
            area.y + 1,
            area.width.saturating_sub(2),
            area.height.saturating_sub(1),
        );

        match &self.value {
            GenericField::Simple(SimpleField::Message(inner_message)) => {
                let inner_widget = self
                    .selection
                    .map(|selection| MessageWidget::new(inner_message).with_selection(selection))
                    .unwrap_or_else(|| MessageWidget::new(inner_message));
                inner_widget.render(area_under, buf);
            }
            GenericField::Simple(simple_value) => {
                if area_right.height == 0 {
                    return; // Nothing to render if the area is empty
                }
                buf.set_stringn(
                    area_right.x,
                    area_right.y,
                    format!("{}", simple_value),
                    area_right.width as usize,
                    style,
                );
            }
            GenericField::Array(array_value) => {
                if area_right.height == 0 {
                    return; // Nothing to render if the area is empty
                }
                buf.set_stringn(
                    area_right.x,
                    area_right.y,
                    format!("{:?}", array_value),
                    area_right.width as usize,
                    Style::default(),
                );
            }
            GenericField::Sequence(sequence_value) => {
                if area_right.height == 0 {
                    return; // Nothing to render if the area is empty
                }
                buf.set_stringn(
                    area_right.x,
                    area_right.y,
                    format!("{:?}", sequence_value),
                    area_right.width as usize,
                    Style::default(),
                );
            }
            GenericField::BoundedSequence(bounded_sequence_value) => {
                if area_right.height == 0 {
                    return; // Nothing to render if the area is empty
                }
                buf.set_stringn(
                    area_right.x,
                    area_right.y,
                    format!("{:?}", bounded_sequence_value),
                    area_right.width as usize,
                    Style::default(),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use color_eyre::owo_colors::OwoColorize;
    use rclrs::DynamicMessage;
    use rclrs::MessageTypeName;

    use super::*;

    #[test]
    fn test_message_widget_render() {
        let message_type = MessageTypeName {
            package_name: "nav_msgs".to_owned(),
            type_name: "Odometry".to_owned(),
        };
        let mut msg = DynamicMessage::new(message_type).unwrap();
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
                "  covariance: Double([0.0, 0.0, 0.0, 0.0, 0.0, 0.0",
                "twist:                                            ",
                "  twist:                                          ",
            ])
        )
    }

    #[test]
    fn test_message_widget_render_bordered() {
        let message_type = MessageTypeName {
            package_name: "nav_msgs".to_owned(),
            type_name: "Odometry".to_owned(),
        };
        let mut msg = DynamicMessage::new(message_type).unwrap();
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
                "│  covariance: Double([0.0, 0.0, 0.0, 0.0, 0.0, 0│",
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
        let mut msg = DynamicMessage::new(message_type).unwrap();
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
            "  covariance: Double([0.0, 0.0, 0.0, 0.0, 0.0, 0.0",
            "twist:                                            ",
            "  twist:                                          ",
        ]);
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
            "  covariance: Double([0.0, 0.0, 0.0, 0.0, 0.0, 0.0",
            "twist:                                            ",
            "  twist:                                          ",
        ]);
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
}
