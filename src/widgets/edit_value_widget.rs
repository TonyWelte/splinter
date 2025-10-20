use ratatui::{buffer::Buffer, layout::Rect, style::Color, text::Span, widgets::Widget};

use crate::common::style::SELECTED_STYLE;

pub struct EditValueState<'a, T> {
    _value: &'a T,
    edit: &'a String,
}

impl<'a, T> EditValueState<'a, T>
where
    T: std::str::FromStr,
{
    pub fn new(value: &'a T, edit: &'a String) -> Self {
        Self {
            _value: value,
            edit,
        }
    }
}

impl<'a, T: std::str::FromStr> From<EditValueState<'a, T>> for Span<'a> {
    fn from(edit_value_state: EditValueState<'a, T>) -> Self {
        let is_edit_valid = edit_value_state.edit.parse::<T>().is_ok();

        let style = if is_edit_valid {
            SELECTED_STYLE.fg(Color::Green)
        } else {
            SELECTED_STYLE.fg(Color::Red)
        };

        Span::raw(edit_value_state.edit.as_str()).style(style)
    }
}

impl<'a, T: std::str::FromStr> Widget for EditValueState<'a, T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let span: Span = self.into();
        span.render(area, buf);
    }
}

// TODO: Use this widget in message_widget
