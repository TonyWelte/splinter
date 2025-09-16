use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use ratatui::{
    layout::{Constraint, Layout},
    prelude::{Buffer, Rect},
    style::{Color, Style},
    symbols,
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, List, ListItem, Widget},
};

use crossterm::event::{Event as CrosstermEvent, KeyCode};
use rclrs::NodeNameInfo;

use crate::{
    common::{
        event::Event,
        style::{HEADER_STYLE, SELECTED_STYLE},
    },
    connections::{Connection, ConnectionType},
    views::TuiView,
};

#[derive(PartialEq)]
struct NodeDetails {
    node_name: NodeNameInfo,
    publishers: BTreeMap<String, Vec<String>>,
    subscribers: BTreeMap<String, Vec<String>>,
    clients: BTreeMap<String, Vec<String>>,
    services: BTreeMap<String, Vec<String>>,
    parameters: Option<Vec<String>>,
}

impl NodeDetails {
    fn count(&self) -> usize {
        self.publishers.len()
            + self.subscribers.len()
            + self.clients.len()
            + self.services.len()
            + self.parameters.as_ref().map_or(0, |p| p.len())
    }
}

pub struct NodeListWidget;

#[derive(Debug, Clone, Copy, PartialEq)]
enum NodeListSections {
    List,
    Details,
}

pub struct NodeListState {
    connection: Rc<RefCell<ConnectionType>>,
    nodes: Vec<NodeDetails>,
    selected_node: usize,
    selected_node_details: Option<usize>,
    active_section: NodeListSections,
    needs_redraw: bool,
}

impl NodeListState {
    pub fn new(connection: Rc<RefCell<ConnectionType>>) -> Self {
        let mut state = Self {
            connection,
            nodes: Vec::new(),
            selected_node: 0,
            selected_node_details: None,
            active_section: NodeListSections::List,
            needs_redraw: true,
        };
        state.nodes = state.list_nodes_details();
        state
    }

    fn list_nodes_details(&self) -> Vec<NodeDetails> {
        self.connection
            .borrow()
            .list_nodes()
            .into_iter()
            .map(|node| NodeDetails {
                node_name: NodeNameInfo {
                    name: node.name.clone(),
                    namespace: node.namespace.clone(),
                },
                publishers: self
                    .connection
                    .borrow()
                    .get_publisher_names_and_types_by_node(&node)
                    .unwrap_or_default()
                    .into_iter()
                    .collect(),
                subscribers: self
                    .connection
                    .borrow()
                    .get_subscription_names_and_types_by_node(&node)
                    .unwrap_or_default()
                    .into_iter()
                    .collect(),
                clients: self
                    .connection
                    .borrow()
                    .get_client_names_and_types_by_node(&node)
                    .unwrap_or_default()
                    .into_iter()
                    .collect(),
                services: self
                    .connection
                    .borrow()
                    .get_service_names_and_types_by_node(&node)
                    .unwrap_or_default()
                    .into_iter()
                    .collect(),
                parameters: None, // Parameters can be fetched if needed
            })
            .collect()
    }

    pub fn next_node(&mut self) {
        if !self.nodes.is_empty() {
            self.selected_node = (self.selected_node + 1) % self.nodes.len();
        }
        self.needs_redraw = true;
    }

    pub fn previous_node(&mut self) {
        if !self.nodes.is_empty() {
            self.selected_node = (self.selected_node + self.nodes.len() - 1) % self.nodes.len();
        }
        self.needs_redraw = true;
    }

    pub fn next_detail(&mut self) {
        if let Some(selected_node) = self.nodes.get(self.selected_node) {
            let details_count = selected_node.count();
            if let Some(details_index) = self.selected_node_details {
                if details_index < details_count - 1 {
                    self.selected_node_details = Some(details_index + 1);
                }
            } else if details_count > 0 {
                self.selected_node_details = Some(0);
            } else {
                self.selected_node_details = None;
            }
        }
        self.needs_redraw = true;
    }

    pub fn previous_detail(&mut self) {
        if let Some(selected_node) = self.nodes.get(self.selected_node) {
            let details_count = selected_node.count();
            if let Some(details_index) = self.selected_node_details {
                if details_index > 0 {
                    self.selected_node_details = Some(details_index - 1);
                }
            } else if details_count > 0 {
                self.selected_node_details = Some(0);
            } else {
                self.selected_node_details = None;
            }
        }
        self.needs_redraw = true;
    }

    pub fn update(&mut self) {
        let mut new_nodes = self.list_nodes_details();
        new_nodes.sort_by(|a, b| {
            if a.node_name.namespace == b.node_name.namespace {
                a.node_name.name.cmp(&b.node_name.name)
            } else {
                a.node_name.namespace.cmp(&b.node_name.namespace)
            }
        });
        if self.nodes.is_empty() {
            self.nodes = new_nodes;
            self.selected_node = 0;
            self.needs_redraw = true;
        } else if self.nodes != new_nodes {
            let selected_node = self.nodes.get(self.selected_node).unwrap();
            let new_index = new_nodes
                .iter()
                .position(|node| {
                    node.node_name.namespace == selected_node.node_name.namespace
                        && node.node_name.name == selected_node.node_name.name
                })
                .unwrap_or(0);
            self.nodes = new_nodes;
            self.selected_node = new_index;
            self.needs_redraw = true;
        }
    }
}

impl TuiView for NodeListState {
    fn handle_event(&mut self, event: Event) -> Event {
        self.update();

        if let Event::Key(CrosstermEvent::Key(key_event)) = event {
            if key_event.kind != crossterm::event::KeyEventKind::Press {
                return event;
            }

            match key_event.code {
                KeyCode::Char('j') | KeyCode::Down => match self.active_section {
                    NodeListSections::List => {
                        self.next_node();
                        Event::None
                    }
                    NodeListSections::Details => {
                        self.next_detail();
                        Event::None
                    }
                },
                KeyCode::Char('k') | KeyCode::Up => match self.active_section {
                    NodeListSections::List => {
                        self.previous_node();
                        Event::None
                    }
                    NodeListSections::Details => {
                        self.previous_detail();
                        Event::None
                    }
                },
                KeyCode::Char('l') | KeyCode::Right => {
                    if self.active_section == NodeListSections::List {
                        self.active_section = NodeListSections::Details;
                        if let Some(selected_node) = self.nodes.get(self.selected_node) {
                            if selected_node.count() > 0 {
                                self.selected_node_details = Some(0);
                            } else {
                                self.selected_node_details = None;
                            }
                        } else {
                            self.selected_node_details = None;
                        }
                        self.needs_redraw = true;
                        return Event::None;
                    }
                    event
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    if self.active_section == NodeListSections::Details {
                        self.active_section = NodeListSections::List;
                        self.selected_node_details = None;
                        self.needs_redraw = true;
                        return Event::None;
                    }
                    event
                }
                _ => event,
            }
        } else {
            event
        }
    }

    fn name(&self) -> String {
        "Nodes".to_string()
    }

    fn get_help_text(&self) -> String {
        "Node List View Help:\n\
        - 'j' or ↓: Move down in the node list or details.\n\
        - 'k' or ↑: Move up in the node list or details.\n\
        - 'l' or →: Switch to details view.\n\
        - 'h' or ←: Switch back to node list view."
            .to_string()
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
        let layout = Layout::horizontal([Constraint::Fill(0), Constraint::Fill(0)]);
        let [list_area, details_area] = layout.areas(area);

        let border_set = symbols::border::Set {
            top_left: symbols::line::NORMAL.horizontal_down,
            bottom_left: symbols::line::NORMAL.horizontal_up,
            ..symbols::border::PLAIN
        };

        let block_list = Block::bordered()
            .title(Line::raw("Node List").centered())
            .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
            .border_style(HEADER_STYLE)
            .border_type(BorderType::Rounded);

        let block_details = Block::bordered()
            .title(Line::raw("Node Details").centered())
            .border_set(border_set)
            .border_style(HEADER_STYLE)
            .border_type(BorderType::Rounded);

        // Iterate through all elements in the `items` and stylize them.
        let node_items: Vec<ListItem> = state
            .nodes
            .iter()
            .enumerate()
            .map(|(i, node_item)| {
                let namespace = if node_item.node_name.namespace.ends_with('/') {
                    node_item.node_name.namespace.clone()
                } else {
                    format!("{}/", node_item.node_name.namespace)
                };
                let node_full_name = format!(
                    "{}{}",
                    node_item.node_name.namespace, node_item.node_name.name
                );
                let style = if i == state.selected_node {
                    if state.active_section == NodeListSections::List {
                        SELECTED_STYLE
                    } else {
                        SELECTED_STYLE.fg(Color::DarkGray)
                    }
                } else {
                    Style::default()
                };
                ListItem::new(node_full_name).style(style)
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let node_list = List::new(node_items).block(block_list);

        // Create the node details view
        let details_list = if let Some(selected_node) = state.nodes.get(state.selected_node) {
            let mut details_count = 0;
            let mut details_items: Vec<ListItem> = Vec::new();
            details_items.push(ListItem::new(Text::from("Publishers:")));
            for (topic, types) in &selected_node.publishers {
                let style = if let Some(details_index) = state.selected_node_details {
                    if details_index == details_count {
                        SELECTED_STYLE
                    } else {
                        Style::default()
                    }
                } else {
                    Style::default()
                };
                details_count += 1;
                details_items.push(ListItem::new(
                    Text::from(format!("  {}: {}", topic, types.join(", "))).style(style),
                ));
            }
            details_items.push(ListItem::new(Text::from("Subscribers:")));
            for (topic, types) in &selected_node.subscribers {
                let style = if let Some(details_index) = state.selected_node_details {
                    if details_index == details_count {
                        SELECTED_STYLE
                    } else {
                        Style::default()
                    }
                } else {
                    Style::default()
                };
                details_count += 1;
                details_items.push(ListItem::new(
                    Text::from(format!("  {}: {}", topic, types.join(", "))).style(style),
                ));
            }
            details_items.push(ListItem::new(Text::from("Clients:")));
            for (service, types) in &selected_node.clients {
                let style = if let Some(details_index) = state.selected_node_details {
                    if details_index == details_count {
                        SELECTED_STYLE
                    } else {
                        Style::default()
                    }
                } else {
                    Style::default()
                };
                details_count += 1;
                details_items.push(ListItem::new(
                    Text::from(format!("  {}: {}", service, types.join(", "))).style(style),
                ));
            }
            details_items.push(ListItem::new(Text::from("Services:")));
            for (service, types) in &selected_node.services {
                let style = if let Some(details_index) = state.selected_node_details {
                    if details_index == details_count {
                        SELECTED_STYLE
                    } else {
                        Style::default()
                    }
                } else {
                    Style::default()
                };
                details_count += 1;
                details_items.push(ListItem::new(
                    Text::from(format!("  {}: {}", service, types.join(", "))).style(style),
                ));
            }
            if let Some(params) = &selected_node.parameters {
                details_items.push(ListItem::new(Text::from("Parameters:")));
                for param in params {
                    let style = if let Some(details_index) = state.selected_node_details {
                        if details_index == details_count {
                            SELECTED_STYLE
                        } else {
                            Style::default()
                        }
                    } else {
                        Style::default()
                    };
                    details_count += 1;
                    details_items
                        .push(ListItem::new(Text::from(format!("  {}", param))).style(style));
                }
            } else {
                details_items.push(ListItem::new(Text::from("Parameters:")));
                details_items.push(ListItem::new(Text::from("  <Not fetched>")));
            }

            List::new(details_items).block(block_details)
        } else {
            List::new([ListItem::new(Text::from("No node selected"))]).block(block_details)
        };

        Widget::render(node_list, list_area, buf);
        Widget::render(details_list, details_area, buf);
    }
}
