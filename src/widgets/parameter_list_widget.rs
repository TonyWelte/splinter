use std::collections::BTreeMap;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Styled,
    text::{Line, Span},
    widgets::{Block, Widget},
};

use crate::{
    common::style::SELECTED_STYLE, connections::Parameters,
    widgets::edit_value_widget::EditValueState,
};

pub struct ParameterListWidget<'a> {
    parameters: &'a BTreeMap<String, Parameters>,
    selected: Option<usize>,
    edit: Option<String>,
    block: Option<Block<'a>>,
}

impl<'a> ParameterListWidget<'a> {
    pub fn new(parameters: &'a BTreeMap<String, Parameters>) -> Self {
        Self {
            parameters,
            selected: None,
            edit: None,
            block: None,
        }
    }

    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    pub fn edit(mut self, edit: Option<String>) -> Self {
        self.edit = edit;
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn height(&self) -> u16 {
        self.parameters.len() as u16
    }
}

impl Widget for ParameterListWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = self.block.unwrap_or_else(Block::default);
        let inner_area = block.inner(area);
        block.render(area, buf);

        let mut y = inner_area.y;
        let mut index = 0;
        for (name, value) in self.parameters {
            if y >= inner_area.bottom() {
                break;
            }

            let is_selected = self.selected == Some(index);

            let param_name_widget = Span::raw(format!("{}: ", name));

            let value_widget = if is_selected {
                if let Some(edit) = &self.edit {
                    match value {
                        Parameters::Bool(v) => EditValueState::new(v, edit).into(),
                        Parameters::Integer(v) => EditValueState::new(v, edit).into(),
                        Parameters::Double(v) => EditValueState::new(v, edit).into(),
                        Parameters::String(v) => EditValueState::new(v, edit).into(),
                        Parameters::BoolArray(v) => Span::raw(format!("{:?}", v)),
                        Parameters::IntegerArray(v) => Span::raw(format!("{:?}", v)),
                        Parameters::DoubleArray(v) => Span::raw(format!("{:?}", v)),
                        Parameters::StringArray(v) => Span::raw(format!("{:?}", v)),
                        Parameters::ByteArray(v) => Span::raw(format!("{:?}", v)),
                    }
                } else {
                    Span::raw(value.to_string()).set_style(SELECTED_STYLE)
                }
            } else {
                Span::raw(value.to_string())
            };
            Line::from(vec![param_name_widget, value_widget]).render(
                Rect {
                    x: inner_area.x,
                    y,
                    width: inner_area.width,
                    height: 1,
                },
                buf,
            );

            y += 1;
            index += 1;
        }
    }
}
