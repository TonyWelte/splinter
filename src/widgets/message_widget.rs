use std::fmt;

use ratatui::{
    prelude::{BlockExt, Buffer, Rect},
    style::{Modifier, Style},
    widgets::{Block, Widget},
};

use crate::common::{
    generic_message::{
        ArrayField, GenericField, GenericMessage, Length,
        SimpleField,
    },
    style::SELECTED_STYLE,
};

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

struct ValueWidget<'a> {
    name: &'a str,
    value: &'a GenericField,
    selection: Option<&'a [usize]>,
    edit: Option<&'a str>,
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
            edit: None,
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

    pub fn height(&self, width: u16) -> u16 {
        match &self.value {
            GenericField::Simple(SimpleField::Message(inner_message)) => {
                MessageWidget::new(inner_message).height(width.saturating_sub(2)) + 1
                // +1 for the field name
            }
            GenericField::Simple(_) => 1,
            GenericField::Array(ArrayField::Message(inner_messages)) => {
                let mut height = 1; // +1 for the field name
                for inner_message in inner_messages {
                    height += MessageWidget::new(inner_message).height(width.saturating_sub(2));
                }
                height
            }
            GenericField::Array(array_value) => {
                1 + ArrayWidget::new(array_value).height(width.saturating_sub(2))
                // +1 for the field name
                // -2 for the indentation
            }
            GenericField::Sequence(_) | GenericField::BoundedSequence(_) => 1,
        }
    }
}

struct ArrayWidget<'a> {
    value: &'a ArrayField,
    selection: Option<&'a [usize]>,
    edit: Option<&'a str>,
}

impl<'a> ArrayWidget<'a> {
    pub fn new(value: &'a ArrayField) -> Self {
        Self {
            value,
            edit: None,
            selection: None,
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

    pub fn height(&self, width: u16) -> u16 {
        match &self.value {
            ArrayField::Message(inner_messages) => {
                let mut height = 0;
                for inner_message in inner_messages {
                    height += MessageWidget::new(inner_message).height(width);
                }
                height
            }
            _ => {
                let quot = (self.value.len() as u16) / (width / 10/* fixed width of val */);
                let rem = (self.value.len() as u16) % (width / 10/* fixed width of val */);
                quot + if rem > 0 { 1 } else { 0 } // +1 for the field name
            }
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
            ArrayField::String(values) => values.iter().map(|v| format!("{}", v)).collect(),
            ArrayField::BoundedString(values) => values.iter().map(|v| format!("{}", v)).collect(),
            ArrayField::WString(values) => values.iter().map(|v| format!("{}", v)).collect(),
            ArrayField::BoundedWString(values) => values.iter().map(|v| format!("{}", v)).collect(),
            _ => vec![],
        }
    }
}

impl<'a> Widget for ArrayWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match &self.value {
            ArrayField::Message(inner_messages) => {
                let mut area_remaining = area;
                for inner_message in inner_messages {
                    let inner_widget = MessageWidget::new(inner_message);
                    let widget_height = inner_widget
                        .height(area_remaining.width)
                        .min(area_remaining.height);
                    inner_widget.render(area_remaining, buf);

                    // Move the area down for the next field
                    area_remaining.y += widget_height;
                    area_remaining.height -= widget_height;
                }
            }
            _ => {
                let values = self.value.as_str_iter();
                let mut x = area.x;
                let mut y = area.y;
                for (i, value) in values.iter().enumerate() {
                    if x + 10 > area.x + area.width {
                        x = area.x;
                        y += 1;
                        if y >= area.y + area.height {
                            break; // No more space to render
                        }
                    }
                    let style = if let Some(selection) = self.selection {
                        if !selection.is_empty() && selection[0] == i {
                            if self.edit.is_some() {
                                SELECTED_STYLE.add_modifier(Modifier::SLOW_BLINK)
                            } else {
                                SELECTED_STYLE
                            }
                        } else {
                            Style::default()
                        }
                    } else {
                        Style::default()
                    };
                    if let Some(edit) = self.edit {
                        if !self.selection.is_none()
                            && !self.selection.unwrap().is_empty()
                            && self.selection.unwrap()[0] == i
                        {
                            buf.set_stringn(x, y, edit, 10, style);
                            x += 10;
                            continue;
                        }
                    }
                    buf.set_stringn(x, y, value, 10, style);
                    x += 10;
                }
            }
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

        if area.height == 0 || area.width == 0 {
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
            area.width.saturating_sub(self.name.len() as u16 + 2),
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
                let mut inner_widget = MessageWidget::new(inner_message);
                if let Some(selection) = self.selection {
                    inner_widget = inner_widget.with_selection(selection);
                }
                if let Some(edit) = self.edit {
                    inner_widget = inner_widget.with_edit(edit);
                }
                inner_widget.render(area_under, buf);
            }
            GenericField::Simple(simple_value) => {
                if area_right.height == 0 {
                    return; // Nothing to render if the area is empty
                }
                if let Some(edit) = self.edit {
                    if !self.selection.is_none() && self.selection.unwrap().is_empty() {
                        buf.set_stringn(
                            area_right.x,
                            area_right.y,
                            edit,
                            area_right.width as usize,
                            style.add_modifier(Modifier::SLOW_BLINK),
                        );
                        return;
                    }
                }
                buf.set_stringn(
                    area_right.x,
                    area_right.y,
                    format!("{}", simple_value),
                    area_right.width as usize,
                    style,
                );
            }
            GenericField::Array(ArrayField::Message(inner_messages)) => {
                if area_right.height == 0 {
                    return; // Nothing to render if the area is empty
                }
                for inner_message in inner_messages {
                    let inner_widget = MessageWidget::new(inner_message);
                    inner_widget.render(area_under, buf);
                }
            }
            GenericField::Array(array_value) => {
                if area_under.height == 0 {
                    return; // Nothing to render if the area is empty
                }
                let mut inner_widget = ArrayWidget::new(array_value);
                if let Some(selection) = self.selection {
                    inner_widget = inner_widget.with_selection(selection);
                    if let Some(edit) = self.edit {
                        inner_widget = inner_widget.with_edit(edit);
                    }
                }
                inner_widget.render(area_under, buf);
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
    use rclrs::DynamicMessage;
    use rclrs::MessageTypeName;

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

        let mut expected = Buffer::with_lines([
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
        expected.set_style(
            Rect {
                x: 9,
                y: 10,
                width: 7,
                height: 1,
            },
            SELECTED_STYLE.add_modifier(Modifier::SLOW_BLINK),
        );

        // Check if the buffer has been modified as expected
        assert_eq!(buffer, expected);
    }
}
