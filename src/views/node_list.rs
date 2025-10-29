use std::{cell::RefCell, rc::Rc};

use ratatui::{
    prelude::{Buffer, Rect},
    style::{Style, Styled, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, StatefulWidget},
};

use crossterm::event::{Event as CrosstermEvent, KeyCode};

use crate::{
    common::{
        event::{Event, NewNodeEvent},
        style::{HEADER_STYLE, SELECTED_STYLE},
        utils::truncate_namespaces,
    },
    connections::{Connection, ConnectionType, NodeName},
    views::TuiView,
    widgets::list_widget::{ListItemTrait, ListWidget, ListWidgetState},
};

impl ListItemTrait for NodeName {
    fn search_text(&self) -> String {
        self.full_name()
    }

    fn to_line(&self, width: usize, selected: bool, indices: Vec<u32>) -> Line {
        let (truncated_name, new_indices) = truncate_namespaces(&self.full_name(), &indices, width);

        let mut spans = vec![];
        if new_indices.is_empty() {
            spans.push(Span::raw(truncated_name));
        } else {
            let first_idx = new_indices.first().unwrap();
            if *first_idx != 0 {
                spans.push(Span::raw(truncated_name[..*first_idx as usize].to_string()));
            }

            for window in new_indices.windows(2) {
                let idx = window[0] as usize;
                let next_idx = window[1] as usize;
                spans.push(Span::styled(
                    truncated_name[idx..idx + 1].to_string(),
                    Style::default().bold(),
                ));
                if next_idx > idx + 1 {
                    spans.push(Span::raw(truncated_name[idx + 1..next_idx].to_string()));
                }
            }

            let last_idx = new_indices.last().unwrap();
            let idx = *last_idx as usize;
            spans.push(Span::styled(
                truncated_name[idx..idx + 1].to_string(),
                Style::default().bold(),
            ));
            if truncated_name.len() > idx + 1 {
                spans.push(Span::raw(truncated_name[idx + 1..].to_string()));
            }
        }

        let mut line = Line::from(spans);

        if selected {
            line = line.set_style(SELECTED_STYLE);
        }

        line
    }
}

pub struct NodeListWidget;

pub struct NodeListState {
    connection: Rc<RefCell<ConnectionType>>,

    node_list_state: ListWidgetState<NodeName>,

    needs_redraw: bool,
}

impl NodeListState {
    pub fn new(connection: Rc<RefCell<ConnectionType>>) -> Self {
        // Initialize node list from connection
        let nodes = connection.borrow().list_nodes();

        Self {
            connection,
            node_list_state: ListWidgetState::new(nodes, None),
            needs_redraw: true,
        }
    }

    pub fn update(&mut self) {
        let node_list_items: Vec<NodeName> = self.connection.borrow().list_nodes();
        self.node_list_state.update(node_list_items);
        self.needs_redraw = true;
    }

    fn handle_event(&mut self, event: Event) -> Event {
        // List view event handling
        let new_event = self.node_list_state.handle_event(event);

        // Node list specific key handling
        if let Event::Key(CrosstermEvent::Key(key_event)) = &new_event {
            if key_event.code == KeyCode::Enter {
                if let Some(selected) = self.node_list_state.get_selected() {
                    // Switch to node details view
                    return Event::NewNodeDetailView(NewNodeEvent {
                        node: selected.clone(),
                    });
                }
                self.needs_redraw = true;
            }
        }

        new_event
    }
}

impl TuiView for NodeListState {
    fn handle_event(&mut self, event: Event) -> Event {
        self.update();

        self.handle_event(event)
    }

    fn name(&self) -> String {
        "Nodes".to_string()
    }

    fn get_help_text(&self) -> String {
        todo!("Add help text")
    }

    fn needs_redraw(&mut self) -> bool {
        if self.needs_redraw {
            self.needs_redraw = false;
            return true;
        }
        false
    }
}

impl NodeListWidget {
    pub fn render(area: Rect, buf: &mut Buffer, state: &mut NodeListState) {
        let border = Block::bordered()
            .title(Line::raw("Node List").centered())
            .border_style(HEADER_STYLE)
            .border_type(BorderType::Rounded);

        // Create a List from all list items and highlight the currently selected one
        let node_list = ListWidget::<NodeName>::new()
            .block(border)
            .auto_scroll(true)
            .enable_search(true)
            .show_mode(true);
        StatefulWidget::render(node_list, area, buf, &mut state.node_list_state);
    }
}
