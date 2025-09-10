use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::{
    common::{
        generic_message::{BoundedSequenceField, Length},
        style::SELECTED_STYLE,
    },
    widgets::message_widget::{AsStrVec, MessageWidget},
};

pub struct BoundedSequenceWidget<'a> {
    name: &'a str,
    value: &'a BoundedSequenceField,
    selection: Option<&'a [usize]>,
    edit: Option<&'a str>,
}

impl<'a> BoundedSequenceWidget<'a> {
    pub fn new(name: &'a str, value: &'a BoundedSequenceField) -> Self {
        Self {
            name,
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
            BoundedSequenceField::Message(inner_messages, _) => {
                let mut height = 1;
                for inner_message in inner_messages {
                    height += MessageWidget::new(inner_message).height(width);
                }
                height
            }
            BoundedSequenceField::String(inner_strings, _)
            | BoundedSequenceField::WString(inner_strings, _)
            | BoundedSequenceField::BoundedString(inner_strings, _)
            | BoundedSequenceField::BoundedWString(inner_strings, _) => {
                1 + inner_strings.len() as u16 // +1 for the field name
            }
            _ => {
                let quot = (self.value.len() as u16) / (width / 10/* fixed width of val */);
                let rem = (self.value.len() as u16) % (width / 10/* fixed width of val */);
                1 + quot + if rem > 0 { 1 } else { 0 } // +1 for the field name
            }
        }
    }
}

impl<'a> Widget for BoundedSequenceWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return; // Nothing to render if the area is empty
        }

        let style = if self.selection.is_some() {
            SELECTED_STYLE
        } else {
            Style::default()
        };

        Line::from_iter([
            Span::raw(format!("{}:", self.name)).style(style),
            Span::raw(format!(
                " {} elements (max: {})",
                self.value.len(),
                self.value.max_len()
            ))
            .style(Style::default().fg(Color::DarkGray)),
        ])
        .render(area, buf);

        let mut area_remaining = area;
        area_remaining.y += 1;
        area_remaining.height = area_remaining.height.saturating_sub(1);

        if area_remaining.height == 0 {
            return; // No more space to render
        }

        match &self.value {
            BoundedSequenceField::Message(inner_messages, _) => {
                for (i, inner_message) in inner_messages.iter().enumerate() {
                    let mut inner_widget = MessageWidget::new(inner_message);
                    if let Some(selection) = self.selection {
                        if !selection.is_empty() && selection[0] == i {
                            inner_widget = inner_widget.with_selection(&selection[1..]);
                            if let Some(edit) = self.edit {
                                inner_widget = inner_widget.with_edit(edit);
                            }
                        }
                    }
                    let widget_height = inner_widget
                        .height(area_remaining.width.saturating_sub(2))
                        .min(area_remaining.height);
                    let message_area = Rect {
                        x: area_remaining.x + 2,
                        y: area_remaining.y,
                        width: area_remaining.width.saturating_sub(2),
                        height: widget_height,
                    };
                    let style = if let Some(selection) = self.selection {
                        if !selection.is_empty() && selection[0] == i {
                            SELECTED_STYLE
                        } else {
                            Style::default()
                        }
                    } else {
                        Style::default()
                    };

                    Span::raw("- ").style(style).render(area_remaining, buf);
                    inner_widget.render(message_area, buf);

                    // Move the area down for the next field
                    area_remaining.y += widget_height;
                    area_remaining.height -= widget_height;
                }
            }
            BoundedSequenceField::String(inner_strings, _)
            | BoundedSequenceField::WString(inner_strings, _)
            | BoundedSequenceField::BoundedString(inner_strings, _)
            | BoundedSequenceField::BoundedWString(inner_strings, _) => {
                // Print each string on its own line, with quotes
                let mut y = area_remaining.y;
                for (i, value) in inner_strings.iter().enumerate() {
                    if y >= area_remaining.y + area_remaining.height {
                        break; // No more space to render
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
                            let is_edit_valid = true; // Any string is valid
                            buf.set_stringn(
                                area_remaining.x,
                                y,
                                edit,
                                area_remaining.width as usize,
                                style.fg(if is_edit_valid {
                                    Color::Green
                                } else {
                                    Color::Red
                                }),
                            );
                            y += 1;
                            continue;
                        }
                    }
                    buf.set_stringn(
                        area_remaining.x,
                        y,
                        &format!("- \"{}\"", value),
                        area_remaining.width as usize,
                        style,
                    );
                    y += 1;
                }
            }
            _ => {
                let values = self.value.as_str_iter();
                let mut x = area_remaining.x;
                for (i, value) in values.iter().enumerate() {
                    if x + 10 > area_remaining.x + area_remaining.width {
                        x = area_remaining.x;
                        area_remaining.y += 1;
                        area_remaining.height = area_remaining.height.saturating_sub(1);
                        if area_remaining.height == 0 {
                            break; // No more space to render
                        }
                    }
                    let style = if let Some(selection) = self.selection {
                        if !selection.is_empty() && selection[0] == i {
                            if self.edit.is_some() {
                                // TODO: Edit validation for Sequences
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
                            let is_edit_valid = match self.value {
                                BoundedSequenceField::Float(_, _) => {
                                    edit.parse::<f32>().is_ok() && !edit.contains(' ')
                                }
                                BoundedSequenceField::Double(_, _) => {
                                    edit.parse::<f64>().is_ok() && !edit.contains(' ')
                                }
                                BoundedSequenceField::LongDouble(_, _) => false,
                                BoundedSequenceField::Char(_, _) => edit.len() == 1,
                                BoundedSequenceField::WChar(_, _) => edit.len() == 1,
                                BoundedSequenceField::Boolean(_, _) => {
                                    let lower = edit.to_lowercase();
                                    lower == "true" || lower == "false"
                                }
                                BoundedSequenceField::Octet(_, _) => {
                                    edit.parse::<u8>().is_ok() && !edit.contains(' ')
                                }
                                BoundedSequenceField::Uint8(_, _) => {
                                    edit.parse::<u8>().is_ok() && !edit.contains(' ')
                                }
                                BoundedSequenceField::Int8(_, _) => {
                                    edit.parse::<i8>().is_ok() && !edit.contains(' ')
                                }
                                BoundedSequenceField::Uint16(_, _) => {
                                    edit.parse::<u16>().is_ok() && !edit.contains(' ')
                                }
                                BoundedSequenceField::Int16(_, _) => {
                                    edit.parse::<i16>().is_ok() && !edit.contains(' ')
                                }
                                BoundedSequenceField::Uint32(_, _) => {
                                    edit.parse::<u32>().is_ok() && !edit.contains(' ')
                                }
                                BoundedSequenceField::Int32(_, _) => {
                                    edit.parse::<i32>().is_ok() && !edit.contains(' ')
                                }
                                BoundedSequenceField::Uint64(_, _) => {
                                    edit.parse::<u64>().is_ok() && !edit.contains(' ')
                                }
                                BoundedSequenceField::Int64(_, _) => {
                                    edit.parse::<i64>().is_ok() && !edit.contains(' ')
                                }
                                BoundedSequenceField::String(_, _) => true, // Any string is valid
                                BoundedSequenceField::BoundedString(_, _) => true, // Any string is valid
                                BoundedSequenceField::WString(_, _) => true, // Any string is valid
                                BoundedSequenceField::BoundedWString(_, _) => true, // Any string is valid
                                BoundedSequenceField::Message(_, _) => false, // Should not happen
                            };
                            buf.set_stringn(
                                x,
                                area_remaining.y,
                                edit,
                                10,
                                style.fg(if is_edit_valid {
                                    Color::Green
                                } else {
                                    Color::Red
                                }),
                            );
                            x += 10;
                            continue;
                        }
                    }
                    buf.set_stringn(x, area_remaining.y, value, 10, style);
                    x += 10;
                }
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
    use indexmap::IndexMap;

    use super::*;
    use crate::common::generic_message::{
        BoundedSequenceField, GenericField, GenericMessage, InterfaceType, SimpleField,
    };

    #[test]
    fn test_sequence_widget_height() {
        let sequence_field = BoundedSequenceField::Uint32(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10], 20);
        let sequence_widget = BoundedSequenceWidget::new("test_sequence", &sequence_field);
        assert_eq!(sequence_widget.height(200), 2); // 10 elements, all fit in one line
        assert_eq!(sequence_widget.height(50), 3); // 10 elements, 5 per line + 1 for name
        assert_eq!(sequence_widget.height(30), 5); // 10 elements, 3 per line + 1 for name
    }

    #[test]
    fn test_sequence_widget_render() {
        let sequence_field = BoundedSequenceField::Uint32(vec![1, 2, 3, 4, 5], 20);
        let sequence_widget = BoundedSequenceWidget::new("test_sequence", &sequence_field);
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));
        sequence_widget.render(buf.area, &mut buf);
        let expected = Buffer::with_lines([
            "test_sequence: 5 elements (max: 20)               ",
            "1         2         3         4         5         ",
            "                                                  ",
            "                                                  ",
        ]);

        buf.set_style(buf.area, Style::reset()); // Not testing style here
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_sequence_widget_render_overflow() {
        let sequence_field = BoundedSequenceField::Uint32((1..=20).collect(), 20);
        let sequence_widget = BoundedSequenceWidget::new("test_sequence", &sequence_field);
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));
        sequence_widget.render(buf.area, &mut buf);
        let expected = Buffer::with_lines([
            "test_sequence: 20 elements (max: 20)              ",
            "1         2         3         4         5         ",
            "6         7         8         9         10        ",
            "11        12        13        14        15        ",
        ]);

        buf.set_style(buf.area, Style::reset()); // Not testing style here
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_sequence_of_messages_render() {
        let sequence_field = BoundedSequenceField::Message(
            vec![
                GenericMessage::new(
                    InterfaceType {
                        package_name: "test".to_string(),
                        catergory: "msg".to_string(),
                        type_name: "Message".to_string(),
                    },
                    IndexMap::from_iter([
                        (
                            "field1".to_string(),
                            GenericField::Simple(SimpleField::Int32(42)),
                        ),
                        (
                            "field2".to_string(),
                            GenericField::Simple(SimpleField::String("hello".to_string())),
                        ),
                    ]),
                ),
                GenericMessage::new(
                    InterfaceType {
                        package_name: "test".to_string(),
                        catergory: "msg".to_string(),
                        type_name: "Message".to_string(),
                    },
                    IndexMap::from_iter([
                        (
                            "field1".to_string(),
                            GenericField::Simple(SimpleField::Int32(100)),
                        ),
                        (
                            "field2".to_string(),
                            GenericField::Simple(SimpleField::String("world".to_string())),
                        ),
                    ]),
                ),
            ],
            4,
        );
        let sequence_widget = BoundedSequenceWidget::new("test_sequence", &sequence_field);
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 10));
        sequence_widget.render(buf.area, &mut buf);
        let expected = Buffer::with_lines([
            "test_sequence: 2 elements (max: 4)                ",
            "- field1: 42                                      ",
            "  field2: \"hello\"                                 ",
            "- field1: 100                                     ",
            "  field2: \"world\"                                 ",
            "                                                  ",
            "                                                  ",
            "                                                  ",
            "                                                  ",
            "                                                  ",
        ]);

        buf.set_style(buf.area, Style::reset()); // Not testing style here
        assert_eq!(buf, expected);
    }
}
