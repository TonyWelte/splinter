use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, StatefulWidget, Widget},
};

use crossterm::event::{Event as CrosstermEvent, KeyCode};

use crate::{
    common::{
        event::{Event, NewPublisherEvent, NewTopicEvent},
        generic_message::InterfaceType,
        style::{HEADER_STYLE, SELECTED_STYLE},
    },
    connections::{Connection, ConnectionType, NodeName, Parameters},
    views::TuiView,
    widgets::{
        list_widget::{ListWidget, ListWidgetState},
        parameter_list_widget::ParameterListWidget,
    },
};

mod interface_list_item;
use interface_list_item::InterfaceListItem;

#[derive(Debug, Clone, Copy, PartialEq)]
enum DetailSection {
    Publishers,
    Subscribers,
    Clients,
    Services,
    Parameters,
}

enum MainDetailSection {
    Section(DetailSection),
    SubSection(DetailSection),
}

#[derive(Debug, Clone, PartialEq)]
enum ParameterListMode {
    Normal,
    Editing(String), // Current edit string
}

#[derive(Clone)]
struct ParameterListView {
    connection: Rc<RefCell<ConnectionType>>,
    node: NodeName,
    parameters: BTreeMap<String, Parameters>,
    selected: Option<usize>,
    mode: ParameterListMode,
}

impl ParameterListView {
    fn new(
        connection: Rc<RefCell<ConnectionType>>,
        node: NodeName,
        parameters: BTreeMap<String, Parameters>,
        selected: Option<usize>,
    ) -> Self {
        Self {
            connection,
            node,
            parameters,
            selected,
            mode: ParameterListMode::Normal,
        }
    }

    fn reset(&mut self) {
        self.parameters.clear();
        self.selected = None;
        self.mode = ParameterListMode::Normal;
    }

    fn update(&mut self, parameters: BTreeMap<String, Parameters>) {
        // Get name of currently selected parameter
        let selected_param_name = self
            .selected
            .and_then(|idx| self.parameters.keys().nth(idx).cloned());

        // Update parameters
        self.parameters = parameters;

        // Restore selection if possible
        if let Some(name) = selected_param_name {
            self.selected = self
                .parameters
                .keys()
                .position(|param_name| param_name == &name);
        } else {
            self.selected = None;
        }
    }

    fn next_item(&mut self) {
        if let Some(selected) = self.selected {
            if selected + 1 < self.parameters.len() {
                self.selected = Some(selected + 1);
            }
        } else if self.parameters.is_empty() {
            self.selected = None;
        } else {
            self.selected = Some(0);
        }
    }

    fn previous_item(&mut self) {
        if let Some(selected) = self.selected {
            self.selected = Some(selected.saturating_sub(1));
        } else if self.parameters.is_empty() {
            self.selected = None;
        } else {
            self.selected = Some(self.parameters.len() - 1);
        }
    }

    fn unselect(&mut self) {
        self.selected = None;
        self.mode = ParameterListMode::Normal;
    }

    fn commit_parameter_edit(&mut self) -> Result<(), String> {
        let new_value = if let ParameterListMode::Editing(edit) = &self.mode {
            edit.clone()
        } else {
            Err("Not in editing mode")?
        };
        let selected = if let Some(s) = self.selected {
            s
        } else {
            Err("No parameter selected")?
        };
        let param_name = if let Some(name) = self.parameters.keys().nth(selected) {
            name.clone()
        } else {
            Err("Selected index out of bounds in parameters")?
        };
        let param = if let Some(p) = self.parameters.get(&param_name) {
            p.clone()
        } else {
            Err("Selected parameter not found")?
        };

        let new_param = match param {
            Parameters::Bool(_) => {
                let bool_value = match new_value.to_lowercase().as_str() {
                    "true" | "1" | "yes" => true,
                    "false" | "0" | "no" => false,
                    _ => Err("Invalid boolean value")?,
                };
                Parameters::Bool(bool_value)
            }
            Parameters::Integer(_) => {
                let int_value: i64 = new_value.parse().map_err(|_| "Invalid integer value")?;
                Parameters::Integer(int_value)
            }
            Parameters::Double(_) => {
                let float_value: f64 = new_value.parse().map_err(|_| "Invalid double value")?;
                Parameters::Double(float_value)
            }
            Parameters::String(_) => Parameters::String(new_value),
            Parameters::ByteArray(_) => Err("Editing byte array parameters is not supported")?,
            Parameters::BoolArray(_) => Err("Editing bool array parameters is not supported")?,
            Parameters::IntegerArray(_) => {
                Err("Editing integer array parameters is not supported")?
            }
            Parameters::DoubleArray(_) => Err("Editing double array parameters is not supported")?,
            Parameters::StringArray(_) => Err("Editing string array parameters is not supported")?,
        };

        self.connection
            .borrow_mut()
            .set_parameter_by_node(&self.node, &param_name, new_param)?;
        self.mode = ParameterListMode::Normal;
        Ok(())
    }

    fn handle_event_in_normal(&mut self, event: Event) -> Event {
        // TODO: Review this function (AI generated)
        if let Event::Key(CrosstermEvent::Key(key_event)) = event {
            match key_event.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    self.next_item();
                    return Event::None;
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.previous_item();
                    return Event::None;
                }
                KeyCode::Enter => {
                    // Enter edit mode if a parameter is selected
                    if let Some(selected) = self.selected {
                        if self.parameters.keys().nth(selected).is_some() {
                            self.mode = ParameterListMode::Editing(String::new());
                            return Event::None;
                        }
                    }
                }
                _ => {}
            }
        }
        event
    }

    fn handle_event_in_editing(&mut self, event: Event) -> Event {
        let selected = if let Some(s) = self.selected {
            s
        } else {
            panic!("No parameter selected while in editing mode")
        };

        let param_name = if let Some(name) = self.parameters.keys().nth(selected) {
            name.clone()
        } else {
            panic!("Selected index out of bounds in parameters")
        };

        if let Event::Key(CrosstermEvent::Key(key_event)) = event {
            match key_event.code {
                KeyCode::Char(c) => {
                    if let ParameterListMode::Editing(ref mut new_value) = self.mode {
                        new_value.push(c);
                    }
                    return Event::None;
                }
                KeyCode::Backspace => {
                    if let ParameterListMode::Editing(ref mut new_value) = self.mode {
                        new_value.pop();
                    }
                    return Event::None;
                }
                KeyCode::Enter => {
                    // Commit the edit
                    // Here we would normally set the parameter via the connection
                    // For now, just print to console (or handle as needed)
                    match self.commit_parameter_edit() {
                        Ok(_) => {
                            return Event::None;
                        }
                        Err(err) => {
                            // Return an error event if setting the parameter failed
                            return Event::Error(format!(
                                "Failed to set parameter '{}': {}",
                                param_name, err
                            ));
                        }
                    }
                }
                KeyCode::Esc => {
                    // Cancel editing
                    self.mode = ParameterListMode::Normal;
                    return Event::None;
                }
                _ => {}
            }
        }
        event
    }

    fn handle_event(&mut self, event: Event) -> Event {
        match &self.mode {
            ParameterListMode::Normal => self.handle_event_in_normal(event),
            ParameterListMode::Editing(_) => self.handle_event_in_editing(event),
        }
    }
}

pub struct NodeDetailState {
    connection: Rc<RefCell<ConnectionType>>,
    node: NodeName,

    publisher_list_state: ListWidgetState<InterfaceListItem>,
    subscriber_list_state: ListWidgetState<InterfaceListItem>,
    client_list_state: ListWidgetState<InterfaceListItem>,
    service_list_state: ListWidgetState<InterfaceListItem>,
    parameter_list_state: ParameterListView,

    active_section: MainDetailSection,

    needs_redraw: bool,
}

impl NodeDetailState {
    pub fn new(node: NodeName, connection: Rc<RefCell<ConnectionType>>) -> Self {
        let mut state = Self {
            connection: connection.clone(),
            node: node.clone(),
            publisher_list_state: ListWidgetState::new(vec![], None),
            subscriber_list_state: ListWidgetState::new(vec![], None),
            client_list_state: ListWidgetState::new(vec![], None),
            service_list_state: ListWidgetState::new(vec![], None),
            parameter_list_state: ParameterListView::new(connection, node, BTreeMap::new(), None),
            active_section: MainDetailSection::Section(DetailSection::Publishers),
            needs_redraw: true,
        };
        state.update();
        state
    }

    pub fn update(&mut self) {
        let connection = self.connection.borrow();
        let publishers = connection
            .get_publisher_names_and_types_by_node(&self.node)
            .unwrap_or_default()
            .iter()
            .map(|(topic, types)| InterfaceListItem {
                full_name: topic.clone(),
                type_name: InterfaceType::new(&types.first().cloned().unwrap_or_default()),
            })
            .collect();
        let subscriptions = connection
            .get_subscription_names_and_types_by_node(&self.node)
            .unwrap_or_default()
            .iter()
            .map(|(topic, types)| InterfaceListItem {
                full_name: topic.clone(),
                type_name: InterfaceType::new(&types.first().cloned().unwrap_or_default()),
            })
            .collect();
        let clients = connection
            .get_client_names_and_types_by_node(&self.node)
            .unwrap_or_default()
            .iter()
            .map(|(service, types)| InterfaceListItem {
                full_name: service.clone(),
                type_name: InterfaceType::new(&types.first().cloned().unwrap_or_default()),
            })
            .collect();
        let services = connection
            .get_service_names_and_types_by_node(&self.node)
            .unwrap_or_default()
            .iter()
            .map(|(service, types)| InterfaceListItem {
                full_name: service.clone(),
                type_name: InterfaceType::new(&types.first().cloned().unwrap_or_default()),
            })
            .collect();

        self.publisher_list_state.update(publishers);
        self.subscriber_list_state.update(subscriptions);
        self.client_list_state.update(clients);
        self.service_list_state.update(services);

        // Update parameters
        match self.connection.borrow().get_parameters_by_node(&self.node) {
            Ok(params) => {
                // Convert HashMap to BTreeMap for consistent ordering
                let params = params.into_iter().collect();
                self.parameter_list_state.update(params);
                self.needs_redraw = true;
            }
            Err(_err) => {
                // TODO: Error handling
                self.needs_redraw = true;
            }
        }

        self.needs_redraw = true;
    }

    pub fn fetch_parameters_for_selected_node(&mut self) {}

    pub fn next_detail(&mut self) {
        match &mut self.active_section {
            MainDetailSection::Section(DetailSection::Publishers) => {
                self.active_section = MainDetailSection::Section(DetailSection::Subscribers);
            }
            MainDetailSection::Section(DetailSection::Subscribers) => {
                self.active_section = MainDetailSection::Section(DetailSection::Clients);
            }
            MainDetailSection::Section(DetailSection::Clients) => {
                self.active_section = MainDetailSection::Section(DetailSection::Services);
            }
            MainDetailSection::Section(DetailSection::Services) => {
                self.active_section = MainDetailSection::Section(DetailSection::Parameters);
            }
            MainDetailSection::Section(DetailSection::Parameters) => {
                self.active_section = MainDetailSection::Section(DetailSection::Publishers);
            }
            MainDetailSection::SubSection(_) => {
                panic!("Should not be in sub detail section when moving to next detail");
            }
        }
    }

    pub fn previous_detail(&mut self) {
        match &mut self.active_section {
            MainDetailSection::Section(DetailSection::Publishers) => {
                self.active_section = MainDetailSection::Section(DetailSection::Parameters);
            }
            MainDetailSection::Section(DetailSection::Subscribers) => {
                self.active_section = MainDetailSection::Section(DetailSection::Publishers);
            }
            MainDetailSection::Section(DetailSection::Clients) => {
                self.active_section = MainDetailSection::Section(DetailSection::Subscribers);
            }
            MainDetailSection::Section(DetailSection::Services) => {
                self.active_section = MainDetailSection::Section(DetailSection::Clients);
            }
            MainDetailSection::Section(DetailSection::Parameters) => {
                self.active_section = MainDetailSection::Section(DetailSection::Services);
            }
            MainDetailSection::SubSection(_) => {
                panic!("Should not be in sub detail section when moving to previous detail");
            }
        }
    }

    fn handle_event(&mut self, event: Event) -> Event {
        if let Event::Key(CrosstermEvent::Key(key_event)) = event {
            match key_event.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    self.next_detail();
                    self.needs_redraw = true;
                    return Event::None;
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.previous_detail();
                    self.needs_redraw = true;
                    return Event::None;
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    if let MainDetailSection::Section(active_detail) = &self.active_section {
                        match active_detail {
                            DetailSection::Publishers => self.publisher_list_state.next_item(),
                            DetailSection::Subscribers => self.subscriber_list_state.next_item(),
                            DetailSection::Clients => self.client_list_state.next_item(),
                            DetailSection::Services => self.service_list_state.next_item(),
                            DetailSection::Parameters => {
                                self.parameter_list_state.next_item();
                            }
                        }
                        self.active_section = MainDetailSection::SubSection(*active_detail);
                        self.needs_redraw = true;
                    }
                    return Event::None;
                }
                _ => {}
            }
        }
        event
    }

    fn handle_event_in_sub_section(&mut self, event: Event) -> Event {
        let active_detail =
            if let MainDetailSection::SubSection(active_detail) = &self.active_section {
                *active_detail
            } else {
                panic!("Not in sub detail section");
            };
        let new_event = match active_detail {
            DetailSection::Publishers => self.publisher_list_state.handle_event(event),
            DetailSection::Subscribers => self.subscriber_list_state.handle_event(event),
            DetailSection::Clients => self.client_list_state.handle_event(event),
            DetailSection::Services => self.service_list_state.handle_event(event),
            DetailSection::Parameters => self.parameter_list_state.handle_event(event),
        };
        if let Event::Key(CrosstermEvent::Key(key_event)) = new_event {
            match key_event.code {
                KeyCode::Char('h') | KeyCode::Left => {
                    if let MainDetailSection::SubSection(active_detail) = &self.active_section {
                        match active_detail {
                            DetailSection::Publishers => self.publisher_list_state.unselect(),
                            DetailSection::Subscribers => self.subscriber_list_state.unselect(),
                            DetailSection::Clients => self.client_list_state.unselect(),
                            DetailSection::Services => self.service_list_state.unselect(),
                            DetailSection::Parameters => {
                                self.parameter_list_state.unselect();
                            }
                        }
                        self.active_section = MainDetailSection::Section(*active_detail);
                        self.needs_redraw = true;
                    }
                    return Event::None;
                }
                KeyCode::Enter => {
                    /* If publisher is selected, open raw view for that publisher
                     * If subscriber is selected, open publisher for that subscriber
                     * If parameters is selected, edit that parameter
                     */
                    match active_detail {
                        DetailSection::Publishers => {
                            if let Some(item) = self.publisher_list_state.get_selected() {
                                return Event::NewMessageView(NewTopicEvent {
                                    topic: item.full_name.clone(),
                                    message_type: item.type_name.clone(),
                                });
                            }
                        }
                        DetailSection::Subscribers => {
                            if let Some(item) = self.subscriber_list_state.get_selected() {
                                return Event::NewPublisher(NewPublisherEvent {
                                    topic: item.full_name.clone(),
                                    message_type: item.type_name.clone(),
                                });
                            }
                        }
                        DetailSection::Clients => {
                            return Event::Error(
                                "Client interaction not implemented yet".to_string(),
                            );
                        }
                        DetailSection::Services => {
                            return Event::Error(
                                "Service interaction not implemented yet".to_string(),
                            );
                        }
                        DetailSection::Parameters => {
                            // TODO
                        }
                    }
                }
                _ => {}
            }
        }
        new_event
    }
}

impl TuiView for NodeDetailState {
    fn handle_event(&mut self, event: Event) -> Event {
        self.update();

        match &mut self.active_section {
            MainDetailSection::Section(_) => self.handle_event(event),
            MainDetailSection::SubSection(_) => self.handle_event_in_sub_section(event),
        }
    }

    fn name(&self) -> String {
        format!("Node - {}", self.node.full_name()).to_string()
    }

    fn get_help_text(&self) -> String {
        "TODO".to_string()
    }

    fn needs_redraw(&mut self) -> bool {
        if self.needs_redraw {
            self.needs_redraw = false;
            return true;
        }
        false
    }
}

pub struct NodeDetailWidget;

impl NodeDetailWidget {
    pub fn render(area: Rect, buf: &mut Buffer, state: &mut NodeDetailState) {
        let block = Block::bordered()
            .title(Line::raw("Node Details").centered())
            .border_style(HEADER_STYLE);

        let mut inner_details_area = block.inner(area);
        Widget::render(block, area, buf);

        // Create the node details view
        // Render publisher
        let publisher_title_style = match state.active_section {
            MainDetailSection::Section(DetailSection::Publishers) => SELECTED_STYLE,
            MainDetailSection::SubSection(DetailSection::Publishers) => {
                SELECTED_STYLE.fg(Color::DarkGray)
            }
            _ => Style::default(),
        }
        .bold();
        Span::raw("Publishers:")
            .style(publisher_title_style)
            .render(inner_details_area, buf);
        inner_details_area.y += 1;
        inner_details_area.height = inner_details_area.height.saturating_sub(1);
        let publisher_list = ListWidget::<InterfaceListItem>::new()
            .auto_scroll(false)
            .enable_search(false)
            .show_mode(false);
        let publisher_list_height = publisher_list.height(&state.publisher_list_state) as u16;
        StatefulWidget::render(
            publisher_list,
            Rect {
                x: inner_details_area.x + 2,
                y: inner_details_area.y,
                width: inner_details_area.width.saturating_sub(2),
                height: inner_details_area.height,
            },
            buf,
            &mut state.publisher_list_state,
        );
        inner_details_area.y += publisher_list_height;
        inner_details_area.height = inner_details_area
            .height
            .saturating_sub(publisher_list_height);

        // Render subscribers
        let subscriber_title_style = match state.active_section {
            MainDetailSection::Section(DetailSection::Subscribers) => SELECTED_STYLE,
            MainDetailSection::SubSection(DetailSection::Subscribers) => {
                SELECTED_STYLE.fg(Color::DarkGray)
            }
            _ => Style::default(),
        }
        .bold();
        Span::raw("Subscriptions:")
            .style(subscriber_title_style)
            .render(inner_details_area, buf);
        inner_details_area.y += 1;
        inner_details_area.height = inner_details_area.height.saturating_sub(1);
        let subscriber_list = ListWidget::<InterfaceListItem>::new()
            .auto_scroll(false)
            .enable_search(false)
            .show_mode(false);
        let subscriber_list_height = subscriber_list.height(&state.subscriber_list_state) as u16;
        StatefulWidget::render(
            subscriber_list,
            Rect {
                x: inner_details_area.x + 2,
                y: inner_details_area.y,
                width: inner_details_area.width.saturating_sub(2),
                height: inner_details_area.height,
            },
            buf,
            &mut state.subscriber_list_state,
        );
        inner_details_area.y += subscriber_list_height;
        inner_details_area.height = inner_details_area
            .height
            .saturating_sub(subscriber_list_height);

        // Render clients
        let client_title_style = match state.active_section {
            MainDetailSection::Section(DetailSection::Clients) => SELECTED_STYLE,
            MainDetailSection::SubSection(DetailSection::Clients) => {
                SELECTED_STYLE.fg(Color::DarkGray)
            }
            _ => Style::default(),
        }
        .bold();
        Span::raw("Clients:")
            .style(client_title_style)
            .render(inner_details_area, buf);
        inner_details_area.y += 1;
        inner_details_area.height = inner_details_area.height.saturating_sub(1);
        let client_list = ListWidget::<InterfaceListItem>::new()
            .auto_scroll(false)
            .enable_search(false)
            .show_mode(false);
        let client_list_height = client_list.height(&state.client_list_state) as u16;
        StatefulWidget::render(
            client_list,
            Rect {
                x: inner_details_area.x + 2,
                y: inner_details_area.y,
                width: inner_details_area.width.saturating_sub(2),
                height: inner_details_area.height,
            },
            buf,
            &mut state.client_list_state,
        );
        inner_details_area.y += client_list_height;
        inner_details_area.height = inner_details_area.height.saturating_sub(client_list_height);

        // Render services
        let service_title_style = match state.active_section {
            MainDetailSection::Section(DetailSection::Services) => SELECTED_STYLE,
            MainDetailSection::SubSection(DetailSection::Services) => {
                SELECTED_STYLE.fg(Color::DarkGray)
            }
            _ => Style::default(),
        }
        .bold();
        Span::raw("Services:")
            .style(service_title_style)
            .render(inner_details_area, buf);
        inner_details_area.y += 1;
        inner_details_area.height = inner_details_area.height.saturating_sub(1);
        let service_list = ListWidget::<InterfaceListItem>::new()
            .auto_scroll(false)
            .enable_search(false)
            .show_mode(false);
        let service_list_height = service_list.height(&state.service_list_state) as u16;
        StatefulWidget::render(
            service_list,
            Rect {
                x: inner_details_area.x + 2,
                y: inner_details_area.y,
                width: inner_details_area.width.saturating_sub(2),
                height: inner_details_area.height,
            },
            buf,
            &mut state.service_list_state,
        );
        inner_details_area.y += service_list_height;
        inner_details_area.height = inner_details_area
            .height
            .saturating_sub(service_list_height);

        // Render parameters
        let parameter_title_style = match state.active_section {
            MainDetailSection::Section(DetailSection::Parameters) => SELECTED_STYLE,
            MainDetailSection::SubSection(DetailSection::Parameters) => {
                SELECTED_STYLE.fg(Color::DarkGray)
            }
            _ => Style::default(),
        }
        .bold();
        Span::raw("Parameters:")
            .style(parameter_title_style)
            .render(inner_details_area, buf);
        inner_details_area.y += 1;
        inner_details_area.height = inner_details_area.height.saturating_sub(1);
        let param_widget = ParameterListWidget::new(&state.parameter_list_state.parameters)
            .selected(state.parameter_list_state.selected)
            .edit(match &state.parameter_list_state.mode {
                ParameterListMode::Editing(edit) => Some(edit.clone()),
                ParameterListMode::Normal => None,
            });
        param_widget.render(
            Rect {
                x: inner_details_area.x + 2,
                y: inner_details_area.y,
                width: inner_details_area.width.saturating_sub(2),
                height: inner_details_area.height,
            },
            buf,
        );
    }
}
