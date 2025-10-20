use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use ratatui::{
    layout::{Constraint, Layout},
    prelude::{Buffer, Rect},
    style::{Color, Style, Styled, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, StatefulWidget, Widget},
};

use crossterm::event::{self, Event as CrosstermEvent, KeyCode};
use rclrs::NodeNameInfo;

use crate::{
    common::{
        event::{Event, NewNodeEvent, NewPublisherEvent, NewTopicEvent},
        generic_message::InterfaceType,
        style::{HEADER_STYLE, SELECTED_STYLE},
    },
    connections::{Connection, ConnectionType, NodeName, Parameters},
    views::TuiView,
    widgets::{
        list_widget::{ListItemTrait, ListWidget, ListWidgetState},
        parameter_list_widget::ParameterListWidget,
    },
};

impl ListItemTrait for NodeName {
    fn search_text(&self) -> String {
        self.full_name()
    }

    fn to_line(&self, width: usize, selected: bool, indices: Vec<u32>) -> Line {
        let mut line = Line::from(self.full_name());
        if selected {
            line = line.set_style(SELECTED_STYLE);
        }
        // TODO: Highlight indices
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
            match key_event.code {
                KeyCode::Enter => {
                    if let Some(selected) = self.node_list_state.get_selected() {
                        // Switch to node details view
                        return Event::NewNodeDetailView(NewNodeEvent {
                            node: selected.clone(),
                        });
                    }
                    self.needs_redraw = true;
                }
                _ => {}
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
