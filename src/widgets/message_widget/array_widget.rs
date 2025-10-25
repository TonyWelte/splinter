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
    widgets::message_widget::{AsStrIter, MessageWidget},
};

const ARRAY_ELEMENT_WIDTH: u16 = 10; // Fixed width for each array element when rendering (including space)

pub struct ArrayWidget<'a> {
    name: &'a str,
    value: &'a ArrayField,
    selection: Option<&'a [usize]>,
    edit: Option<&'a str>,
    max_elements: usize,
    max_strings: usize,
    max_objects: usize,
}

impl<'a> ArrayWidget<'a> {
    pub fn new(name: &'a str, value: &'a ArrayField) -> Self {
        Self {
            name,
            value,
            edit: None,
            selection: None,
            max_elements: 100,
            max_strings: 10,
            max_objects: 10,
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

    pub fn with_max_visible_elements(
        mut self,
        max_elements: usize,
        max_strings: usize,
        max_objects: usize,
    ) -> Self {
        self.max_elements = max_elements;
        self.max_strings = max_strings;
        self.max_objects = max_objects;
        self
    }

    pub fn height(&self, width: u16) -> u16 {
        match &self.value {
            ArrayField::Message(inner_messages) => {
                let mut height = 1;
                for inner_message in inner_messages.iter().take(self.max_objects) {
                    height += MessageWidget::new(inner_message).height(width);
                }
                if inner_messages.len() > self.max_objects {
                    height += 1; // For the "..." line
                }
                height
            }
            ArrayField::String(inner_strings)
            | ArrayField::WString(inner_strings)
            | ArrayField::BoundedString(inner_strings)
            | ArrayField::BoundedWString(inner_strings) => {
                let mut height = 1 + (inner_strings.len().min(self.max_strings) as u16);
                if inner_strings.len() > self.max_strings {
                    height += 1; // For the "..." line
                }
                height
            }
            _ => {
                let visible_elements = self.value.len().min(self.max_elements);
                let quot = (visible_elements as u16) / (width / ARRAY_ELEMENT_WIDTH);
                let rem = (visible_elements as u16) % (width / ARRAY_ELEMENT_WIDTH);
                let mut height = 1 + quot + if rem > 0 { 1 } else { 0 }; // +1 for the field name
                if self.value.len() > self.max_elements {
                    height += 1; // For the "..." line
                }
                height
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
                for (i, inner_message) in inner_messages.iter().take(self.max_objects).enumerate() {
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
                if inner_messages.len() > self.max_objects {
                    Span::raw("...")
                        .style(Style::default().fg(Color::DarkGray))
                        .render(area_remaining, buf);
                }
            }
            ArrayField::String(inner_strings)
            | ArrayField::WString(inner_strings)
            | ArrayField::BoundedString(inner_strings)
            | ArrayField::BoundedWString(inner_strings) => {
                // Print each string on its own line, with quotes
                let mut y = area_remaining.y;
                for (i, value) in inner_strings.iter().take(self.max_strings).enumerate() {
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
                    if self.edit.is_some()
                        && self.selection.is_some()
                        && self.selection.unwrap().first() == Some(&i)
                    {
                        let edit = self.edit.unwrap(); // unwrap: checked above
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
                    buf.set_stringn(
                        area_remaining.x,
                        y,
                        format!("- \"{}\"", value),
                        area_remaining.width as usize,
                        style,
                    );
                    y += 1;
                }
                if inner_strings.len() > self.max_strings {
                    Span::raw("...")
                        .style(Style::default().fg(Color::DarkGray))
                        .render(
                            Rect {
                                x: area_remaining.x,
                                y,
                                width: area_remaining.width,
                                height: 1,
                            },
                            buf,
                        );
                }
            }
            _ => {
                let mut x = area_remaining.x;
                for (i, value) in self.value.as_str_iter().take(self.max_elements).enumerate() {
                    if x + ARRAY_ELEMENT_WIDTH - 1 > area_remaining.x + area_remaining.width {
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
                        if self.selection.is_some() && self.selection.unwrap().first() == Some(&i) {
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
                                ARRAY_ELEMENT_WIDTH as usize - 1,
                                style.fg(if is_edit_valid {
                                    Color::Green
                                } else {
                                    Color::Red
                                }),
                            );
                            x += ARRAY_ELEMENT_WIDTH;
                            continue;
                        }
                    }
                    buf.set_stringn(
                        x,
                        area_remaining.y,
                        value,
                        ARRAY_ELEMENT_WIDTH as usize - 1,
                        style,
                    );
                    x += ARRAY_ELEMENT_WIDTH;
                }
                if self.value.len() > self.max_elements {
                    Span::raw("...")
                        .style(Style::default().fg(Color::DarkGray))
                        .render(
                            Rect {
                                x: area_remaining.x,
                                y: area_remaining.y + 1,
                                width: area_remaining.width,
                                height: 1,
                            },
                            buf,
                        );
                }
            }
        }
    }
}

impl AsStrIter for ArrayField {
    fn as_str_iter(&self) -> Box<dyn Iterator<Item = String> + '_> {
        match self {
            ArrayField::Float(values) => Box::new(values.iter().map(|v| format!("{}", v))),
            ArrayField::Double(values) => Box::new(values.iter().map(|v| format!("{}", v))),
            ArrayField::LongDouble(_) => Box::new(std::iter::empty()),
            ArrayField::Char(values) => Box::new(values.iter().map(|v| format!("{}", v))),
            ArrayField::WChar(values) => Box::new(values.iter().map(|v| format!("{}", v))),
            ArrayField::Boolean(values) => Box::new(values.iter().map(|v| format!("{}", v))),
            ArrayField::Octet(values) => Box::new(values.iter().map(|v| format!("{}", v))),
            ArrayField::Uint8(values) => Box::new(values.iter().map(|v| format!("{}", v))),
            ArrayField::Int8(values) => Box::new(values.iter().map(|v| format!("{}", v))),
            ArrayField::Uint16(values) => Box::new(values.iter().map(|v| format!("{}", v))),
            ArrayField::Int16(values) => Box::new(values.iter().map(|v| format!("{}", v))),
            ArrayField::Uint32(values) => Box::new(values.iter().map(|v| format!("{}", v))),
            ArrayField::Int32(values) => Box::new(values.iter().map(|v| format!("{}", v))),
            ArrayField::Uint64(values) => Box::new(values.iter().map(|v| format!("{}", v))),
            ArrayField::Int64(values) => Box::new(values.iter().map(|v| format!("{}", v))),
            ArrayField::String(values) => Box::new(values.iter().map(|v| format!("\"{}\"", v))),
            ArrayField::BoundedString(values) => {
                Box::new(values.iter().map(|v| format!("\"{}\"", v)))
            }
            ArrayField::WString(values) => Box::new(values.iter().map(|v| format!("\"{}\"", v))),
            ArrayField::BoundedWString(values) => {
                Box::new(values.iter().map(|v| format!("\"{}\"", v)))
            }
            _ => Box::new(std::iter::empty()),
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
        assert_eq!(
            array_widget.with_max_visible_elements(5, 1, 1).height(30),
            4
        ); // 5 visible elements, 3 per line + 1 for "..." + 1 for name
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

    #[test]
    fn test_array_widget_render_with_max_elements() {
        let array_field = ArrayField::Uint32((1..=150).collect());
        let array_widget =
            ArrayWidget::new("test_array", &array_field).with_max_visible_elements(50, 1, 1);
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 12));
        array_widget.render(buf.area, &mut buf);
        let expected = Buffer::with_lines([
            "test_array: 150 elements                          ",
            "1         2         3         4         5         ",
            "6         7         8         9         10        ",
            "11        12        13        14        15        ",
            "16        17        18        19        20        ",
            "21        22        23        24        25        ",
            "26        27        28        29        30        ",
            "31        32        33        34        35        ",
            "36        37        38        39        40        ",
            "41        42        43        44        45        ",
            "46        47        48        49        50        ",
            "...                                               ",
        ]);

        buf.set_style(buf.area, Style::reset()); // Not testing style here
        assert_eq!(buf, expected);
    }
}
