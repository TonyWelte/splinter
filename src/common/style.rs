use ratatui::style::{Modifier, Style};

pub const HEADER_STYLE: Style = Style::new().add_modifier(Modifier::BOLD);
pub const SELECTED_STYLE: Style = Style::new()
    .add_modifier(Modifier::REVERSED)
    .add_modifier(Modifier::BOLD);
