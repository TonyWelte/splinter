use ratatui::{style::Styled, text::Line};

use crate::{
    common::{generic_message::InterfaceType, style::SELECTED_STYLE},
    widgets::list_widget::ListItemTrait,
};

#[derive(Clone)]
pub struct InterfaceListItem {
    pub full_name: String,
    pub type_name: InterfaceType,
}

impl ListItemTrait for InterfaceListItem {
    fn search_text(&self) -> String {
        self.full_name.clone()
    }

    fn to_line(&self, width: usize, selected: bool, indices: Vec<u32>) -> Line {
        let type_name_str = self.type_name.as_str();
        let space_width = width.saturating_sub(self.full_name.len() + type_name_str.len());
        let mut line = Line::from(format!(
            "{}{}{}",
            self.full_name,
            " ".repeat(space_width),
            type_name_str
        ));
        if selected {
            line = line.set_style(SELECTED_STYLE);
        }
        line
    }
}
