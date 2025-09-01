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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::generic_message::BoundedSequenceField;

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
}
