use std::collections::BTreeMap;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Widget},
};

use crate::{common::style::SELECTED_STYLE, connections::Parameters};

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
        let block = self
            .block
            .unwrap_or_else(|| Block::default().borders(Borders::ALL));
        let inner_area = block.inner(area);
        block.render(area, buf);

        let mut y = inner_area.y;
        let mut index = 0;
        for (name, value) in self.parameters {
            if y >= inner_area.bottom() {
                break;
            }

            let is_selected = self.selected == Some(index);
            let style = if is_selected {
                SELECTED_STYLE
            } else {
                Style::default()
            };

            let display_value = if is_selected {
                if let Some(edit) = &self.edit {
                    edit.clone()
                } else {
                    value.to_string()
                }
            } else {
                value.to_string()
            };

            let line = format!("{}: {}", name, display_value);
            let line = if line.len() > inner_area.width as usize {
                &line[..inner_area.width as usize]
            } else {
                &line
            };

            buf.set_string(inner_area.x, y, line, style);
            y += 1;
            index += 1;
        }
    }
}
