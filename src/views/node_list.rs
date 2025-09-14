use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use ratatui::{
    layout::{Constraint, Layout},
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    symbols,
    text::Line,
    widgets::{Block, BorderType, Borders, List, ListItem, Widget},
};

use crossterm::event::{Event as CrosstermEvent, KeyCode};
use rclrs::NodeNameInfo;

use crate::{
    common::{
        event::{Event, NewPublisherEvent, NewTopicEvent},
        generic_message::InterfaceType,
        style::{HEADER_STYLE, SELECTED_STYLE},
    },
    connections::{Connection, ConnectionType, Parameters},
    views::TuiView,
    widgets::parameter_list_widget::ParameterListWidget,
};

#[derive(Debug, Clone, PartialEq)]
enum ParameterState {
    NotFetched,
    FailedToFetch,
    Fetched(BTreeMap<String, Parameters>),
}

impl ParameterState {
    fn as_ref(&self) -> Option<&BTreeMap<String, Parameters>> {
        match self {
            ParameterState::Fetched(map) => Some(map),
            _ => None,
        }
    }
}

#[derive(PartialEq)]
struct NodeDetails {
    node_name: NodeNameInfo,
    publishers: BTreeMap<String, Vec<String>>,
    subscribers: BTreeMap<String, Vec<String>>,
    clients: BTreeMap<String, Vec<String>>,
    services: BTreeMap<String, Vec<String>>,
    parameters: ParameterState,
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
enum ParameterSectionMode {
    Normal,
    Editing,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum DetailSection {
    Publishers,
    Subscribers,
    Clients,
    Services,
    Parameters(ParameterSectionMode),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum NodeListSections {
    List,
    Details,
    SubDetails(DetailSection),
}

// TODO: Redo selection architecture
pub struct NodeListState {
    connection: Rc<RefCell<ConnectionType>>,
    nodes: BTreeMap<String, NodeDetails>,

    selected_node: Option<String>,
    selected_node_details: Option<DetailSection>,
    selected_node_sub_details: Option<usize>,
    edit: Option<String>,

    active_section: NodeListSections,
    needs_redraw: bool,
}

impl NodeListState {
    pub fn new(connection: Rc<RefCell<ConnectionType>>) -> Self {
        let mut state = Self {
            connection,
            nodes: BTreeMap::new(),
            selected_node: None,
            selected_node_details: None,
            selected_node_sub_details: None,
            edit: None,
            active_section: NodeListSections::List,
            needs_redraw: true,
        };
        state.nodes = state.list_nodes_details();
        state
    }

    fn list_nodes_details(&self) -> BTreeMap<String, NodeDetails> {
        let my_connection = self.connection.borrow();
        my_connection
            .list_nodes()
            .into_iter()
            .map(|node| {
                let namespace = if node.namespace.ends_with('/') {
                    node.namespace.clone()
                } else {
                    format!("{}/", node.namespace)
                };
                let node_full_name = format!("{}{}", namespace, node.name);
                (
                    node_full_name,
                    NodeDetails {
                        node_name: NodeNameInfo {
                            name: node.name.clone(),
                            namespace: node.namespace.clone(),
                        },
                        publishers: my_connection
                            .get_publisher_names_and_types_by_node(&node)
                            .unwrap_or_default()
                            .into_iter()
                            .collect(),
                        subscribers: my_connection
                            .get_subscription_names_and_types_by_node(&node)
                            .unwrap_or_default()
                            .into_iter()
                            .collect(),
                        clients: my_connection
                            .get_client_names_and_types_by_node(&node)
                            .unwrap_or_default()
                            .into_iter()
                            .collect(),
                        services: my_connection
                            .get_service_names_and_types_by_node(&node)
                            .unwrap_or_default()
                            .into_iter()
                            .collect(),
                        parameters: ParameterState::NotFetched, // Parameters will be fetched on demand
                    },
                )
            })
            .collect()
    }

    pub fn next_node(&mut self) {
        let mut found_node = false;
        if let Some(selected_node) = &self.selected_node {
            for node_name in self.nodes.keys() {
                if found_node {
                    self.selected_node = Some(node_name.clone());
                    break;
                }
                if node_name == selected_node {
                    found_node = true;
                }
            }
        }
        if !found_node && !self.nodes.is_empty() {
            self.selected_node = Some(self.nodes.first_key_value().unwrap().0.clone());
        }
        self.needs_redraw = true;
    }

    pub fn previous_node(&mut self) {
        let mut found_node = false;
        if let Some(selected_node) = &self.selected_node {
            for node_name in self.nodes.keys().rev() {
                if found_node {
                    self.selected_node = Some(node_name.clone());
                    break;
                }
                if node_name == selected_node {
                    found_node = true;
                }
            }
        }
        if !found_node && !self.nodes.is_empty() {
            self.selected_node = Some(self.nodes.last_key_value().unwrap().0.clone());
        }
        self.needs_redraw = true;
    }

    pub fn next_detail(&mut self) {
        match &self.selected_node_details {
            None => self.selected_node_details = Some(DetailSection::Publishers),
            Some(DetailSection::Publishers) => {
                self.selected_node_details = Some(DetailSection::Subscribers)
            }
            Some(DetailSection::Subscribers) => {
                self.selected_node_details = Some(DetailSection::Clients)
            }
            Some(DetailSection::Clients) => {
                self.selected_node_details = Some(DetailSection::Services)
            }
            Some(DetailSection::Services) => {
                self.selected_node_details =
                    Some(DetailSection::Parameters(ParameterSectionMode::Normal))
            }
            Some(DetailSection::Parameters(_)) => {}
        }
        self.needs_redraw = true;
    }

    pub fn previous_detail(&mut self) {
        match &self.selected_node_details {
            None => {
                self.selected_node_details =
                    Some(DetailSection::Parameters(ParameterSectionMode::Normal))
            }
            Some(DetailSection::Parameters(_)) => {
                self.selected_node_details = Some(DetailSection::Services)
            }
            Some(DetailSection::Services) => {
                self.selected_node_details = Some(DetailSection::Clients)
            }
            Some(DetailSection::Clients) => {
                self.selected_node_details = Some(DetailSection::Subscribers)
            }
            Some(DetailSection::Subscribers) => {
                self.selected_node_details = Some(DetailSection::Publishers)
            }
            Some(DetailSection::Publishers) => {}
        }
        self.needs_redraw = true;
    }

    pub fn next_sub_detail(&mut self) {
        if let Some(selected_node) = &self.selected_node {
            if let Some(selected_section) = &mut self.selected_node_details {
                if let Some(node_details) = self.nodes.get(selected_node) {
                    let section_count = match selected_section {
                        DetailSection::Publishers => node_details.publishers.len(),
                        DetailSection::Subscribers => node_details.subscribers.len(),
                        DetailSection::Clients => node_details.clients.len(),
                        DetailSection::Services => node_details.services.len(),
                        DetailSection::Parameters(mode) => {
                            *mode = ParameterSectionMode::Normal;
                            node_details.parameters.as_ref().map_or(0, |p| p.len())
                        }
                    };
                    if section_count == 0 {
                        self.selected_node_sub_details = None;
                    } else {
                        self.selected_node_sub_details = Some(
                            self.selected_node_sub_details
                                .map_or(0, |idx| std::cmp::min(idx + 1, section_count - 1)),
                        );
                    }
                    self.needs_redraw = true;
                }
            }
        }
    }

    pub fn previous_sub_detail(&mut self) {
        if let Some(selected_node) = &self.selected_node {
            if let Some(selected_section) = &mut self.selected_node_details {
                if let Some(node_details) = self.nodes.get(selected_node) {
                    let section_count = match selected_section {
                        DetailSection::Publishers => node_details.publishers.len(),
                        DetailSection::Subscribers => node_details.subscribers.len(),
                        DetailSection::Clients => node_details.clients.len(),
                        DetailSection::Services => node_details.services.len(),
                        DetailSection::Parameters(mode) => {
                            *mode = ParameterSectionMode::Normal;
                            node_details.parameters.as_ref().map_or(0, |p| p.len())
                        }
                    };
                    if section_count == 0 {
                        self.selected_node_sub_details = None;
                    } else {
                        self.selected_node_sub_details = Some(
                            self.selected_node_sub_details
                                .map_or(section_count - 1, |idx| idx.saturating_sub(1)),
                        );
                    }
                    self.needs_redraw = true;
                }
            }
        }
    }

    pub fn update(&mut self) {
        let new_nodes = self.list_nodes_details();
        if self.nodes.is_empty() {
            self.nodes = new_nodes;
            self.selected_node = self.nodes.first_key_value().map(|(k, _)| k.clone());
            self.needs_redraw = true;
        } else if self.nodes.keys().cloned().collect::<Vec<String>>()
            != new_nodes.keys().cloned().collect::<Vec<String>>()
        {
            for (node_name, node_details) in new_nodes {
                self.nodes.entry(node_name).or_insert(node_details);
            }
            self.needs_redraw = true;
        }
    }

    pub fn fetch_parameters_for_selected_node(&mut self) {
        if let Some(selected_node) = &self.selected_node {
            if let Some(node_details) = self.nodes.get_mut(selected_node) {
                match node_details.parameters {
                    ParameterState::NotFetched => {
                        match self
                            .connection
                            .borrow()
                            .get_parameters_by_node(&node_details.node_name)
                        {
                            Ok(params) => {
                                node_details.parameters =
                                    ParameterState::Fetched(params.into_iter().collect());
                                self.needs_redraw = true;
                            }
                            Err(err) => {
                                node_details.parameters = ParameterState::FailedToFetch;
                                self.needs_redraw = true;
                            }
                        }
                    }
                    ParameterState::FailedToFetch | ParameterState::Fetched(_) => {}
                }
            }
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

            eprintln!("Active section: {:?}", self.active_section);

            if let NodeListSections::SubDetails(DetailSection::Parameters(
                ParameterSectionMode::Editing,
            )) = self.active_section
            {
                eprintln!("Editing mode key: {:?}", key_event.code);
                match key_event.code {
                    KeyCode::Char(c) => {
                        if let Some(edit) = &mut self.edit {
                            edit.push(c);
                        } else {
                            self.edit = Some(c.to_string());
                        }
                        self.needs_redraw = true;
                        return Event::None;
                    }
                    KeyCode::Backspace => {
                        if let Some(edit) = &mut self.edit {
                            edit.pop();
                            self.needs_redraw = true;
                        }
                        return Event::None;
                    }
                    KeyCode::Esc => {
                        if let Some(selected_node) = &self.selected_node {
                            if let Some(selected_section) = &mut self.selected_node_details {
                                if let Some(node_details) = self.nodes.get_mut(selected_node) {
                                    if let DetailSection::Parameters(mode) = selected_section {
                                        *mode = ParameterSectionMode::Normal;
                                        self.edit = None;
                                        self.needs_redraw = true;
                                    }
                                }
                            }
                        }
                        return Event::None;
                    }
                    _ => {}
                }
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
                    NodeListSections::SubDetails(_) => {
                        self.next_sub_detail();
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
                    NodeListSections::SubDetails(_) => {
                        self.previous_sub_detail();
                        Event::None
                    }
                },
                KeyCode::Char('l') | KeyCode::Right => {
                    self.fetch_parameters_for_selected_node();
                    if let Some(selected_node) = &self.selected_node {
                        match self.active_section {
                            NodeListSections::List => {
                                self.active_section = NodeListSections::Details;
                                self.selected_node_details = Some(DetailSection::Publishers);
                                self.selected_node_sub_details = None;
                            }
                            NodeListSections::Details => {
                                if let Some(selected_section) = &self.selected_node_details {
                                    self.active_section =
                                        NodeListSections::SubDetails(*selected_section);
                                    let sub_details_count = match selected_section {
                                        DetailSection::Publishers => self
                                            .nodes
                                            .get(selected_node)
                                            .map_or(0, |n| n.publishers.len()),
                                        DetailSection::Subscribers => self
                                            .nodes
                                            .get(selected_node)
                                            .map_or(0, |n| n.subscribers.len()),
                                        DetailSection::Clients => self
                                            .nodes
                                            .get(selected_node)
                                            .map_or(0, |n| n.clients.len()),
                                        DetailSection::Services => self
                                            .nodes
                                            .get(selected_node)
                                            .map_or(0, |n| n.services.len()),
                                        DetailSection::Parameters(_) => {
                                            self.nodes.get(selected_node).map_or(0, |n| {
                                                n.parameters.as_ref().map_or(0, |p| p.len())
                                            })
                                        }
                                    };
                                    if sub_details_count == 0 {
                                        self.selected_node_sub_details = None;
                                    } else {
                                        self.selected_node_sub_details = Some(0);
                                    }
                                }
                            }
                            NodeListSections::SubDetails(_) => {}
                        }
                        self.needs_redraw = true;
                        return Event::None;
                    }
                    event
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    match self.active_section {
                        NodeListSections::List => {}
                        NodeListSections::Details => {
                            self.active_section = NodeListSections::List;
                            self.selected_node_details = None;
                            self.selected_node_sub_details = None;
                            self.needs_redraw = true;
                            return Event::None;
                        }
                        NodeListSections::SubDetails(_) => {
                            self.active_section = NodeListSections::Details;
                            self.selected_node_sub_details = None;
                            self.needs_redraw = true;
                            return Event::None;
                        }
                    }
                    event
                }
                KeyCode::Enter => {
                    /* If publisher is selected, open raw view for that publisher
                     * If subscriber is selected, open publisher for that subscriber
                     * If parameters is selected, edit that parameter
                     */
                    if let Some(selected_node) = &self.selected_node {
                        if let NodeListSections::SubDetails(selected_section) =
                            &mut self.active_section
                        {
                            if let Some(node_details) = self.nodes.get_mut(selected_node) {
                                match selected_section {
                                    DetailSection::Publishers => {
                                        if let Some(idx) = self.selected_node_sub_details {
                                            if let Some((topic, types)) =
                                                node_details.publishers.iter().nth(idx)
                                            {
                                                return Event::NewMessageView(NewTopicEvent {
                                                    topic: topic.clone(),
                                                    message_type: InterfaceType::new(
                                                        &types.first().cloned().unwrap_or_default(),
                                                    ),
                                                });
                                            }
                                        }
                                    }
                                    DetailSection::Subscribers => {
                                        if let Some(idx) = self.selected_node_sub_details {
                                            if let Some((topic, types)) =
                                                node_details.subscribers.iter().nth(idx)
                                            {
                                                return Event::NewPublisher(NewPublisherEvent {
                                                    topic: topic.clone(),
                                                    message_type: InterfaceType::new(
                                                        &types.first().cloned().unwrap_or_default(),
                                                    ),
                                                });
                                            }
                                        }
                                    }
                                    DetailSection::Parameters(mode) => {
                                        // TODO: Some many nests, refactor
                                        if let Some(idx) = self.selected_node_sub_details {
                                            if let ParameterState::Fetched(params) =
                                                &mut node_details.parameters
                                            {
                                                if let Some((param_name, param_value)) =
                                                    params.iter_mut().nth(idx)
                                                {
                                                    match param_value {
                                                        Parameters::Bool(_)
                                                        | Parameters::Integer(_)
                                                        | Parameters::Double(_) => {
                                                            match mode {
                                                                ParameterSectionMode::Normal => {
                                                                    *mode =
                                                                    ParameterSectionMode::Editing;
                                                                    eprintln!(
                                                                        "Editing param: {}",
                                                                        param_name
                                                                    );
                                                                    self.edit = Some(
                                                                        param_value.to_string(),
                                                                    );
                                                                    self.needs_redraw = true;
                                                                }
                                                                ParameterSectionMode::Editing => {
                                                                    eprintln!(
                                                                        "Setting param: {} to {:?}",
                                                                        param_name, self.edit
                                                                    );
                                                                    if let Some(edit) = &self.edit {
                                                                        let new_param =
                                                                            if let Ok(b) =
                                                                                edit.parse::<bool>()
                                                                            {
                                                                                Parameters::Bool(b)
                                                                            } else if let Ok(i) =
                                                                                edit.parse::<i64>()
                                                                            {
                                                                                Parameters::Integer(
                                                                                    i,
                                                                                )
                                                                            } else if let Ok(f) =
                                                                                edit.parse::<f64>()
                                                                            {
                                                                                Parameters::Double(
                                                                                    f,
                                                                                )
                                                                            } else {
                                                                                // Invalid input, ignore
                                                                                return event;
                                                                            };
                                                                        if let Err(err) = self
                                                                            .connection
                                                                            .borrow_mut()
                                                                            .set_parameter_by_node(
                                                                                &node_details
                                                                                    .node_name,
                                                                                param_name,
                                                                                new_param.clone(),
                                                                            )
                                                                        {
                                                                            return Event::Error(format!(
                                                                            "Failed to set parameter: {}",
                                                                            err
                                                                        ));
                                                                        } else {
                                                                            // Update parameters
                                                                            eprintln!(
                                                                            "Parameter set successfully"
                                                                        );
                                                                            *param_value =
                                                                                new_param;
                                                                        }
                                                                    }
                                                                    *mode =
                                                                    ParameterSectionMode::Normal;
                                                                    self.edit = None;
                                                                    self.needs_redraw = true;
                                                                }
                                                            }
                                                        }
                                                        _ => {
                                                            // Non-editable parameter types
                                                            return Event::Error(
                                                                "Only bool, integer, and double parameters can be edited"
                                                                    .to_string(),
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
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
        - 'h' or ←: Switch back to node list view.\n\
        \n\
        - 'Enter': Execute action based on selected detail:\n\
        - If a publisher is selected, open a raw view for that publisher.\n\
        - If a subscriber is selected, open a publisher for that subscriber.\n\
        - If a parameter is selected, edit that parameter."
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
        let layout = Layout::horizontal([Constraint::Ratio(1, 3), Constraint::Fill(0)]);
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
            .keys()
            .map(|node_full_name| {
                let style = if let Some(selected_node) = &state.selected_node {
                    if selected_node == node_full_name {
                        if state.active_section == NodeListSections::List {
                            SELECTED_STYLE
                        } else {
                            SELECTED_STYLE.fg(Color::DarkGray)
                        }
                    } else {
                        Style::default()
                    }
                } else {
                    Style::default()
                };
                ListItem::new(node_full_name.clone()).style(style)
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let node_list = List::new(node_items).block(block_list);
        Widget::render(node_list, list_area, buf);

        let inner_details_area = block_details.inner(details_area);
        Widget::render(block_details, details_area, buf);

        // Create the node details view
        if let Some(selected_node) = &state.selected_node {
            let selected_node = state.nodes.get(selected_node).unwrap();
            let mut y_offset = inner_details_area.top();

            // TODO: Share code with topic_list.rs
            let mut render_section =
                |title: &str, items: &BTreeMap<String, Vec<String>>, section: DetailSection| {
                    let title_style = if state.selected_node_details == Some(section) {
                        if state.active_section == NodeListSections::SubDetails(section) {
                            SELECTED_STYLE.fg(Color::DarkGray)
                        } else {
                            SELECTED_STYLE
                        }
                    } else {
                        Style::default()
                    }
                    .bold();
                    buf.set_string(inner_details_area.left(), y_offset, title, title_style);
                    y_offset += 1;

                    for (i, (key, values)) in items.iter().enumerate() {
                        let style = if NodeListSections::SubDetails(section) == state.active_section
                            && state.selected_node_sub_details.is_some()
                            && state.selected_node_sub_details.unwrap() == i
                        {
                            SELECTED_STYLE
                        } else {
                            Style::default()
                        };
                        let type_name = values.first().cloned().unwrap_or_default();
                        buf.set_stringn(
                            inner_details_area.left() + 2,
                            y_offset,
                            key,
                            inner_details_area.width.saturating_sub(2) as usize,
                            style,
                        );
                        buf.set_stringn(
                            inner_details_area.left()
                                + inner_details_area
                                    .width
                                    .saturating_sub(type_name.len() as u16),
                            y_offset,
                            &type_name,
                            inner_details_area.width.saturating_sub(2) as usize,
                            style,
                        );
                        buf.set_style(
                            Rect {
                                x: inner_details_area.left() + 2,
                                y: y_offset,
                                width: inner_details_area.width.saturating_sub(2),
                                height: 1,
                            },
                            style,
                        );
                        y_offset += 1;
                    }
                };

            render_section(
                "Publishers:",
                &selected_node.publishers,
                DetailSection::Publishers,
            );
            render_section(
                "Subscribers:",
                &selected_node.subscribers,
                DetailSection::Subscribers,
            );
            render_section("Clients:", &selected_node.clients, DetailSection::Clients);
            render_section(
                "Services:",
                &selected_node.services,
                DetailSection::Services,
            );

            buf.set_string(
                inner_details_area.left(),
                y_offset,
                "Parameters:",
                if let Some(DetailSection::Parameters(_)) = state.selected_node_details {
                    if let NodeListSections::SubDetails(DetailSection::Parameters(_)) =
                        state.active_section
                    {
                        SELECTED_STYLE.fg(Color::DarkGray)
                    } else {
                        SELECTED_STYLE
                    }
                } else {
                    Style::default()
                }
                .bold(),
            );
            y_offset += 1;

            match &selected_node.parameters {
                ParameterState::Fetched(params) => {
                    let mut parameter_widget = ParameterListWidget::new(params)
                        .block(Block::default().borders(Borders::NONE))
                        .edit(None);
                    if let NodeListSections::SubDetails(DetailSection::Parameters(mode)) =
                        state.active_section
                    {
                        parameter_widget =
                            parameter_widget.selected(state.selected_node_sub_details);
                        if let ParameterSectionMode::Editing = mode {
                            parameter_widget = parameter_widget.edit(state.edit.clone());
                        }
                    }
                    parameter_widget.render(
                        Rect {
                            x: inner_details_area.left() + 2,
                            y: y_offset,
                            width: inner_details_area.width,
                            height: inner_details_area.height
                                - (y_offset - inner_details_area.top()),
                        },
                        buf,
                    );
                }
                ParameterState::NotFetched => {
                    buf.set_string(
                        inner_details_area.left(),
                        y_offset,
                        "  <Not fetched>",
                        Style::default(),
                    );
                    y_offset += 1;
                }
                ParameterState::FailedToFetch => {
                    buf.set_string(
                        inner_details_area.left(),
                        y_offset,
                        "  <Failed to fetch>",
                        Style::default(),
                    );
                    y_offset += 1;
                }
            }
        } else {
            buf.set_string(
                inner_details_area.left(),
                inner_details_area.top(),
                "No node selected",
                Style::default(),
            );
        }
    }
}
