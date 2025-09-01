use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::{
    common::{
        generic_message::{
            ArrayField, BoundedSequenceField, GenericField, SequenceField, SimpleField,
        },
        style::SELECTED_STYLE,
    },
    widgets::message_widget::{ArrayWidget, BoundedSequenceWidget, MessageWidget, SequenceWidget},
};

pub struct ValueWidget<'a> {
    name: &'a str,
    value: &'a GenericField,
    selection: Option<&'a [usize]>,
    edit: Option<&'a str>,
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
                ArrayWidget::new(self.name, array_value).height(width)
                // -2 for the indentation
            }
            GenericField::Sequence(SequenceField::Message(inner_messages)) => {
                let mut height = 1; // +1 for the field name
                for inner_message in inner_messages {
                    height += MessageWidget::new(inner_message).height(width);
                }
                height
            }
            GenericField::Sequence(sequence_value) => {
                SequenceWidget::new(self.name, sequence_value).height(width)
                // +1 for the field name
                // -2 for the indentation
            }
            GenericField::BoundedSequence(BoundedSequenceField::Message(inner_messages, _)) => {
                let mut height = 1; // +1 for the field name
                for inner_message in inner_messages {
                    height += MessageWidget::new(inner_message).height(width);
                }
                height
            }
            GenericField::BoundedSequence(sequence_value) => {
                BoundedSequenceWidget::new(self.name, sequence_value).height(width)
                // +1 for the field name
                // -2 for the indentation
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
                buf.set_stringn(
                    area.x,
                    area.y,
                    format!("{}: ", self.name),
                    area.width as usize - area.x as usize,
                    style,
                );
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
                buf.set_stringn(
                    area.x,
                    area.y,
                    format!("{}: ", self.name),
                    area.width as usize - area.x as usize,
                    style,
                );
                if let Some(edit) = self.edit {
                    if !self.selection.is_none() && self.selection.unwrap().is_empty() {
                        let is_edit_valid = match simple_value {
                            SimpleField::Float(_) => {
                                edit.parse::<f32>().is_ok() && !edit.contains(' ')
                            }
                            SimpleField::Double(_) => {
                                edit.parse::<f64>().is_ok() && !edit.contains(' ')
                            }
                            SimpleField::LongDouble(_) => false,
                            SimpleField::Char(_) => edit.len() == 1,
                            SimpleField::WChar(_) => edit.len() == 1,
                            SimpleField::Boolean(_) => {
                                let lower = edit.to_lowercase();
                                lower == "true" || lower == "false"
                            }
                            SimpleField::Octet(_) => {
                                edit.parse::<u8>().is_ok() && !edit.contains(' ')
                            }
                            SimpleField::Uint8(_) => {
                                edit.parse::<u8>().is_ok() && !edit.contains(' ')
                            }
                            SimpleField::Int8(_) => {
                                edit.parse::<i8>().is_ok() && !edit.contains(' ')
                            }
                            SimpleField::Uint16(_) => {
                                edit.parse::<u16>().is_ok() && !edit.contains(' ')
                            }
                            SimpleField::Int16(_) => {
                                edit.parse::<i16>().is_ok() && !edit.contains(' ')
                            }
                            SimpleField::Uint32(_) => {
                                edit.parse::<u32>().is_ok() && !edit.contains(' ')
                            }
                            SimpleField::Int32(_) => {
                                edit.parse::<i32>().is_ok() && !edit.contains(' ')
                            }
                            SimpleField::Uint64(_) => {
                                edit.parse::<u64>().is_ok() && !edit.contains(' ')
                            }
                            SimpleField::Int64(_) => {
                                edit.parse::<i64>().is_ok() && !edit.contains(' ')
                            }
                            SimpleField::String(_) => true, // Any string is valid
                            SimpleField::BoundedString(_) => true, // Any string is valid
                            SimpleField::WString(_) => true, // Any string is valid
                            SimpleField::BoundedWString(_) => true, // Any string is valid
                            SimpleField::Message(_) => false, // Should not happen
                        };
                        buf.set_stringn(
                            area_right.x,
                            area_right.y,
                            edit,
                            area_right.width as usize,
                            style
                                .add_modifier(Modifier::SLOW_BLINK)
                                .fg(if is_edit_valid {
                                    Color::Green
                                } else {
                                    Color::Red
                                }),
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
                if area.height == 0 {
                    return; // Nothing to render if the area is empty
                }
                let mut inner_widget = ArrayWidget::new(self.name, array_value);
                if let Some(selection) = self.selection {
                    inner_widget = inner_widget.with_selection(selection);
                    if let Some(edit) = self.edit {
                        inner_widget = inner_widget.with_edit(edit);
                    }
                }
                inner_widget.render(area, buf);
            }
            GenericField::Sequence(sequence_value) => {
                if area.height == 0 {
                    return; // Nothing to render if the area is empty
                }
                let mut inner_widget = SequenceWidget::new(self.name, sequence_value);
                if let Some(selection) = self.selection {
                    inner_widget = inner_widget.with_selection(selection);
                    if let Some(edit) = self.edit {
                        inner_widget = inner_widget.with_edit(edit);
                    }
                }
                inner_widget.render(area, buf);
            }
            GenericField::BoundedSequence(bounded_sequence_value) => {
                if area.height == 0 {
                    return; // Nothing to render if the area is empty
                }
                let mut inner_widget =
                    BoundedSequenceWidget::new(self.name, bounded_sequence_value);
                if let Some(selection) = self.selection {
                    inner_widget = inner_widget.with_selection(selection);
                    if let Some(edit) = self.edit {
                        inner_widget = inner_widget.with_edit(edit);
                    }
                }
                inner_widget.render(area, buf);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use super::*;

    use crate::common::generic_message::{GenericMessage, InterfaceType};

    #[test]
    fn test_value_widget_height() {
        let simple_field = GenericField::Simple(SimpleField::Int32(42));
        let value_widget = ValueWidget::new("simple_field", &simple_field);
        assert_eq!(value_widget.height(50), 1);
        assert_eq!(value_widget.height(10), 1);
    }

    #[test]
    fn test_value_widget_height_message() {
        let inner_message = GenericMessage::new(
            InterfaceType {
                package_name: "test_pkg".to_string(),
                catergory: "msg".to_string(),
                type_name: "InnerMessage".to_string(),
            },
            IndexMap::from([
                (
                    "field1".to_string(),
                    GenericField::Simple(SimpleField::Int32(1)),
                ),
                (
                    "field2".to_string(),
                    GenericField::Simple(SimpleField::String("test".to_string())),
                ),
            ]),
        );
        let message_field = GenericField::Simple(SimpleField::Message(inner_message));
        let value_widget = ValueWidget::new("message_field", &message_field);
        assert_eq!(value_widget.height(50), 3);
        assert_eq!(value_widget.height(25), 3);
    }

    #[test]
    fn test_value_widget_height_array() {
        let array_field = GenericField::Array(ArrayField::Int32(vec![1, 2, 3]));
        let value_widget = ValueWidget::new("array_field", &array_field);
        assert_eq!(value_widget.height(50), 2);
        assert_eq!(value_widget.height(25), 3);
    }

    #[test]
    fn test_value_widget_height_sequence() {
        let sequence_field = GenericField::Sequence(SequenceField::Int32(vec![1, 2, 3]));
        let value_widget = ValueWidget::new("sequence_field", &sequence_field);
        assert_eq!(value_widget.height(50), 2); // Sequence fits in one line
        assert_eq!(value_widget.height(25), 3); // 2 per line
    }

    #[test]
    fn test_value_widget_height_bounded_sequence() {
        let bounded_sequence_field =
            GenericField::BoundedSequence(BoundedSequenceField::Int32(vec![1, 2, 3], 5));
        let value_widget = ValueWidget::new("bounded_sequence_field", &bounded_sequence_field);
        assert_eq!(value_widget.height(50), 2); // BoundedSequence fits in one line
        assert_eq!(value_widget.height(25), 3); // 2 per line
    }

    #[test]
    fn test_render_simple_field() {
        let simple_field = GenericField::Simple(SimpleField::Int32(42));
        let value_widget = ValueWidget::new("simple_field", &simple_field);

        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));
        value_widget.render(Rect::new(0, 0, 20, 1), &mut buf);

        let expected = Buffer::with_lines(["simple_field: 42    "]);

        assert_eq!(buf, expected);
    }
}
