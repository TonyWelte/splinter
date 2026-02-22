use std::{cell::RefCell, rc::Rc};

use ratatui::{
    prelude::{Buffer, Rect},
    text::Line,
    widgets::{Block, BorderType, StatefulWidget},
};

use crossterm::event::{Event as CrosstermEvent, KeyCode};

use crate::{
    common::{event::Event, style::HEADER_STYLE},
    connections::{Connection, ConnectionType, NodeName},
    views::{ConnectionInfo, FromConnection, NodeInfo, TuiView},
    widgets::list_widget::{ListWidget, ListWidgetState},
};

pub struct NodeListWidget;

pub struct NodeListState {
    connection: Rc<RefCell<ConnectionType>>,

    node_list_state: ListWidgetState<NodeName>,

    last_update: std::time::Instant,
    needs_redraw: bool,
}

impl NodeListState {
    pub fn new(connection: Rc<RefCell<ConnectionType>>) -> Self {
        // Initialize node list from connection
        let nodes = connection.borrow().list_nodes().unwrap_or_default();

        Self {
            connection,
            node_list_state: ListWidgetState::new(nodes, None),
            last_update: std::time::Instant::now(),
            needs_redraw: true,
        }
    }

    pub fn update(&mut self) {
        const UPDATE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(500);
        if self.last_update.elapsed() < UPDATE_INTERVAL {
            return;
        }
        self.last_update = std::time::Instant::now();
        let node_list_items: Vec<NodeName> =
            self.connection.borrow().list_nodes().unwrap_or_default();
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
                    return Event::NewNode(NodeInfo {
                        node_name: selected.clone(),
                        connection: self.connection.clone(),
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
        "Node List View Help:\n\
        Normal Mode:\n\
        - 'j' or ↓: Move down in the node list.\n\
        - 'k' or ↑: Move up in the node list.\n\
        - 'Enter': Open the details view for the selected node.\n\
        Search Mode:\n\
        - '/': Enter search mode.\n\
        - Type to filter nodes.\n\
        - 'Backspace': Remove the last character from the search filter.\n\
        - 'Esc'/'Enter': Exit search mode."
            .to_string()
    }

    fn needs_redraw(&mut self) -> bool {
        if self.needs_redraw || self.node_list_state.needs_redraw() {
            self.needs_redraw = false;
            return true;
        }
        false
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        NodeListWidget::render(area, buf, self);
    }
}

impl FromConnection for NodeListState {
    fn from_connection(connection_info: ConnectionInfo) -> Self {
        NodeListState::new(connection_info.connection)
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
