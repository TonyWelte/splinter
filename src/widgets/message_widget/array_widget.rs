use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::{
    common::{
        generic_message::{ArrayField, Length},
        style::SELECTED_STYLE,
    },
    widgets::message_widget::{AsStrVec, MessageWidget},
};

pub struct ArrayWidget<'a> {
    name: &'a str,
    value: &'a ArrayField,
    selection: Option<&'a [usize]>,
    edit: Option<&'a str>,
}

impl<'a> ArrayWidget<'a> {
    pub fn new(name: &'a str, value: &'a ArrayField) -> Self {
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
            ArrayField::Message(inner_messages) => {
                let mut height = 1;
                for inner_message in inner_messages {
                    height += MessageWidget::new(inner_message).height(width);
                }
                height
            }
            ArrayField::String(inner_strings)
            | ArrayField::WString(inner_strings)
            | ArrayField::BoundedString(inner_strings)
            | ArrayField::BoundedWString(inner_strings) => {
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

impl<'a> Widget for ArrayWidget<'a> {
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
            Span::raw(format!(" {} elements", self.value.len()))
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
            ArrayField::Message(inner_messages) => {
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
            ArrayField::String(inner_strings)
            | ArrayField::WString(inner_strings)
            | ArrayField::BoundedString(inner_strings)
            | ArrayField::BoundedWString(inner_strings) => {
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
                        if self.selection.is_some()
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
                        format!("- \"{}\"", value),
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
                                // TODO: Edit validation for arrays
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
                        if self.selection.is_some()
                            && !self.selection.unwrap().is_empty()
                            && self.selection.unwrap()[0] == i
                        {
                            let is_edit_valid = match self.value {
                                ArrayField::Float(_) => {
                                    edit.parse::<f32>().is_ok() && !edit.contains(' ')
                                }
                                ArrayField::Double(_) => {
                                    edit.parse::<f64>().is_ok() && !edit.contains(' ')
                                }
                                ArrayField::LongDouble(_) => false,
                                ArrayField::Char(_) => edit.len() == 1,
                                ArrayField::WChar(_) => edit.len() == 1,
                                ArrayField::Boolean(_) => {
                                    let lower = edit.to_lowercase();
                                    lower == "true" || lower == "false"
                                }
                                ArrayField::Octet(_) => {
                                    edit.parse::<u8>().is_ok() && !edit.contains(' ')
                                }
                                ArrayField::Uint8(_) => {
                                    edit.parse::<u8>().is_ok() && !edit.contains(' ')
                                }
                                ArrayField::Int8(_) => {
                                    edit.parse::<i8>().is_ok() && !edit.contains(' ')
                                }
                                ArrayField::Uint16(_) => {
                                    edit.parse::<u16>().is_ok() && !edit.contains(' ')
                                }
                                ArrayField::Int16(_) => {
                                    edit.parse::<i16>().is_ok() && !edit.contains(' ')
                                }
                                ArrayField::Uint32(_) => {
                                    edit.parse::<u32>().is_ok() && !edit.contains(' ')
                                }
                                ArrayField::Int32(_) => {
                                    edit.parse::<i32>().is_ok() && !edit.contains(' ')
                                }
                                ArrayField::Uint64(_) => {
                                    edit.parse::<u64>().is_ok() && !edit.contains(' ')
                                }
                                ArrayField::Int64(_) => {
                                    edit.parse::<i64>().is_ok() && !edit.contains(' ')
                                }
                                ArrayField::String(_) => true, // Any string is valid
                                ArrayField::BoundedString(_) => true, // Any string is valid
                                ArrayField::WString(_) => true, // Any string is valid
                                ArrayField::BoundedWString(_) => true, // Any string is valid
                                ArrayField::Message(_) => false, // Should not happen
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

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use super::*;
    use crate::common::generic_message::{
        ArrayField, GenericField, GenericMessage, InterfaceType, SimpleField,
    };

    #[test]
    fn test_array_widget_height() {
        let array_field = ArrayField::Uint32(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let array_widget = ArrayWidget::new("test_array", &array_field);
        assert_eq!(array_widget.height(200), 2); // 10 elements, all fit in one line
        assert_eq!(array_widget.height(50), 3); // 10 elements, 5 per line + 1 for name
        assert_eq!(array_widget.height(30), 5); // 10 elements, 3 per line + 1 for name
    }

    #[test]
    fn test_array_widget_render() {
        let array_field = ArrayField::Uint32(vec![1, 2, 3, 4, 5]);
        let array_widget = ArrayWidget::new("test_array", &array_field);
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));
        array_widget.render(buf.area, &mut buf);
        let expected = Buffer::with_lines([
            "test_array: 5 elements                            ",
            "1         2         3         4         5         ",
            "                                                  ",
            "                                                  ",
        ]);

        buf.set_style(buf.area, Style::reset()); // Not testing style here
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_array_widget_render_overflow() {
        let array_field = ArrayField::Uint32((1..=20).collect());
        let array_widget = ArrayWidget::new("test_array", &array_field);
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));
        array_widget.render(buf.area, &mut buf);
        let expected = Buffer::with_lines([
            "test_array: 20 elements                           ",
            "1         2         3         4         5         ",
            "6         7         8         9         10        ",
            "11        12        13        14        15        ",
        ]);

        buf.set_style(buf.area, Style::reset()); // Not testing style here
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_array_of_messages_render() {
        let array_field = ArrayField::Message(vec![
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
        ]);
        let array_widget = ArrayWidget::new("test_array", &array_field);
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 10));
        array_widget.render(buf.area, &mut buf);
        let expected = Buffer::with_lines([
            "test_array: 2 elements                            ",
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
