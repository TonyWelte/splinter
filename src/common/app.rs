use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use color_eyre::eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEventKind},
    layout::{Alignment, Constraint, Layout, Rect},
    style::Stylize,
    widgets::{Clear, Paragraph, Tabs, Widget},
    DefaultTerminal,
};

use crate::common::style::SELECTED_STYLE;
use crate::connections::ros2::ConnectionROS2;
use crate::connections::ConnectionType;
use crate::popups::text_popup::TextPopup;
use crate::popups::PopupView;
use crate::popups::{add_hz_popup::AddHzState, add_line_popup::AddLineState};
use crate::views::hz_plot::{HzPlotState, HzPlotWidget};
use crate::views::live_plot::LivePlotState;
use crate::views::raw_message::RawMessageState;
use crate::views::topic_publisher::TopicPublisherState;
use crate::views::{
    live_plot::LivePlotWidget,
    node_list::{NodeListState, NodeListWidget},
    raw_message::RawMessageWidget,
    topic_list::{TopicList, TopicListState},
    topic_publisher::TopicPublisherWidget,
    TuiView, Views,
};
use crate::{common::event::Event, views::node_details::NodeDetailState};
use crate::{common::generic_message::InterfaceType, views::node_details::NodeDetailWidget};

pub struct AppMetrics {
    pub draw_count: u32,
    pub events: Vec<Event>,
}

impl Default for AppMetrics {
    fn default() -> Self {
        Self {
            draw_count: 0,
            events: vec![],
        }
    }
}

pub struct App {
    should_exit: bool,
    pub connection: Rc<RefCell<ConnectionType>>,
    widgets: Vec<Views>,
    active_widget_index: usize,

    popup_view: PopupView,

    needs_redraw: bool,

    metrics: AppMetrics,
}

pub enum AppArgs {
    TopicList,
    NodeList,
    RawMessage(String),
    TopicPublisher(String, String),
    HzPlot(String),
}

impl Default for App {
    fn default() -> Self {
        let should_exit = false;
        let connection = Rc::new(RefCell::new(ConnectionType::ROS2(ConnectionROS2::new())));
        let topic_list = TopicListState::new(connection.clone());
        let node_list = NodeListState::new(connection.clone());
        Self {
            should_exit,
            connection,
            widgets: vec![Views::TopicList(topic_list), Views::NodeList(node_list)],
            active_widget_index: 0,
            popup_view: PopupView::None,
            needs_redraw: true,
            metrics: AppMetrics::default(),
        }
    }
}

fn check_ctrl_c(event: &CrosstermEvent) -> bool {
    if let CrosstermEvent::Key(key_event) = event {
        if key_event.code == KeyCode::Char('c')
            && key_event
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL)
        {
            return true;
        }
    }
    false
}

impl App {
    pub fn new(args: AppArgs) -> Self {
        let should_exit = false;
        let connection = Rc::new(RefCell::new(ConnectionType::ROS2(ConnectionROS2::new())));
        let view = match args {
            AppArgs::TopicList => {
                let topic_list = TopicListState::new(connection.clone());
                Views::TopicList(topic_list)
            }
            AppArgs::NodeList => {
                let node_list = NodeListState::new(connection.clone());
                Views::NodeList(node_list)
            }
            AppArgs::RawMessage(topic) => {
                let raw_message_state = RawMessageState::new(topic, connection.clone());
                Views::RawMessage(raw_message_state)
            }
            AppArgs::TopicPublisher(topic, topic_type) => {
                let topic_type = InterfaceType::new(&topic_type);
                let topic_publisher_state =
                    TopicPublisherState::new(topic, topic_type, connection.clone());
                Views::TopicPublisher(topic_publisher_state)
            }
            AppArgs::HzPlot(topic) => {
                let hz_plot_state = HzPlotState::new(topic, connection.clone());
                Views::HzPlot(hz_plot_state)
            }
        };

        Self {
            should_exit,
            connection,
            widgets: vec![view],
            active_widget_index: 0,
            popup_view: PopupView::None,
            needs_redraw: true,
            metrics: AppMetrics::default(),
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            let popup_needs_redraw = match &mut self.popup_view {
                PopupView::None => false,
                PopupView::AddLine(state) => state.needs_redraw(),
                PopupView::AddHz(state) => state.needs_redraw(),
                PopupView::Error(state) => state.needs_redraw(),
            };
            if self.widgets[self.active_widget_index].needs_redraw()
                || self.needs_redraw
                || popup_needs_redraw
            {
                self.needs_redraw = false;
                self.metrics.draw_count += 1;
                terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            }
            if let Ok(is_event_available) = event::poll(Duration::from_millis(100)) {
                if is_event_available {
                    let event = event::read()?;
                    if check_ctrl_c(&event) {
                        self.should_exit = true;
                        continue;
                    }
                    self.handle_event(Event::Key(event));
                } else {
                    self.handle_event(Event::None);
                }
            }
        }
        Ok(())
    }

    fn handle_event(&mut self, event: Event) {
        // Log all events for debugging purposes
        self.metrics.events.push(event.clone());

        let event = match &mut self.popup_view {
            PopupView::None => self.widgets[self.active_widget_index].handle_event(event),
            PopupView::AddLine(data) => {
                let new_event = data.handle_event(event);
                match new_event {
                    Event::None => Event::None,
                    other_event => {
                        self.popup_view = PopupView::None;
                        other_event
                    }
                }
            }
            PopupView::AddHz(data) => {
                let new_event = data.handle_event(event);
                match new_event {
                    Event::None => Event::None,
                    other_event => {
                        self.popup_view = PopupView::None;
                        other_event
                    }
                }
            }
            PopupView::Error(data) => data.handle_event(event),
        };

        match event {
            Event::Key(CrosstermEvent::Resize(_, _)) => {
                self.needs_redraw = true;
            }
            Event::Key(CrosstermEvent::Key(key_event)) => {
                if key_event.kind != KeyEventKind::Press {
                    return;
                }
                match key_event.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.should_exit = true;
                    }
                    KeyCode::Char('x') => {
                        if self.widgets.len() > 1 {
                            self.widgets.remove(self.active_widget_index);
                            if self.active_widget_index != 0 {
                                self.active_widget_index -= 1;
                            }
                        }
                        self.needs_redraw = true;
                    }
                    KeyCode::Tab => {
                        self.active_widget_index =
                            (self.active_widget_index + 1) % self.widgets.len();
                        self.needs_redraw = true;
                    }
                    KeyCode::BackTab => {
                        if self.active_widget_index == 0 {
                            self.active_widget_index = self.widgets.len() - 1;
                        } else {
                            self.active_widget_index -= 1;
                        }
                        self.needs_redraw = true;
                    }
                    KeyCode::Char('?') => {
                        if let Some(active_view) = self.widgets.get(self.active_widget_index) {
                            let mut help_text = active_view.get_help_text();
                            help_text.push_str("\n\n");
                            help_text.push_str(&self.get_help_test());
                            self.popup_view = PopupView::Error(TextPopup::info(help_text));
                        }
                    }
                    _ => {}
                }
            }
            Event::NewLine(new_graph_event) => {
                let topic = new_graph_event.topic;
                let field = new_graph_event.field;
                let field_name = new_graph_event.field_name;
                if new_graph_event.view.is_some() {
                    let view = new_graph_event.view.unwrap();
                    let connection = self.connection.clone();
                    if let Some(Views::LivePlot(live_plot_state)) = self.widgets.get_mut(view) {
                        live_plot_state.add_graph_line(topic, field, field_name, connection);
                        self.active_widget_index = view;
                    }
                    return;
                }
                let candidate_views: Vec<(usize, String)> = self
                    .widgets
                    .iter()
                    .enumerate()
                    .filter_map(|(i, w)| match w {
                        Views::LivePlot(_) => Some((i, w.name())),
                        _ => None,
                    })
                    .collect();
                self.popup_view = PopupView::AddLine(AddLineState::new(
                    topic,
                    field,
                    field_name,
                    candidate_views,
                ));
            }
            Event::NewLinePlot(new_graph_event) => {
                let topic = new_graph_event.topic;
                let field = new_graph_event.field;
                let field_name = new_graph_event.field_name;
                let connection = self.connection.clone();
                let live_plot_state = LivePlotState::new(topic, field, field_name, connection);
                let widget = Views::LivePlot(live_plot_state);
                self.widgets.push(widget);
                self.active_widget_index = self.widgets.len() - 1;
            }
            Event::NewMessageView(new_topic_event) => {
                let topic = new_topic_event.topic;
                let connection = self.connection.clone();
                let raw_message_state = RawMessageState::new(topic, connection);
                let widget = Views::RawMessage(raw_message_state);
                self.widgets.push(widget);
                self.active_widget_index = self.widgets.len() - 1;
            }
            Event::NewHz(new_hz_event) => {
                if new_hz_event.view.is_some() {
                    let topic = new_hz_event.topic;
                    let view = new_hz_event.view.unwrap();
                    let connection = self.connection.clone();
                    if let Some(Views::HzPlot(hz_plot_state)) = self.widgets.get_mut(view) {
                        hz_plot_state.add_line(topic, connection);
                        self.active_widget_index = view;
                    }
                    return;
                }
                let topic = new_hz_event.topic;
                let candidate_views: Vec<(usize, String)> = self
                    .widgets
                    .iter()
                    .enumerate()
                    .filter_map(|(i, w)| match w {
                        Views::HzPlot(_) => Some((i, w.name())),
                        _ => None,
                    })
                    .collect();
                self.popup_view = PopupView::AddHz(AddHzState::new(topic, candidate_views));
            }
            Event::NewHzPlot(new_topic_event) => {
                let topic = new_topic_event.topic;
                let connection = self.connection.clone();
                let hz_plot_state = HzPlotState::new(topic, connection);
                let widget = Views::HzPlot(hz_plot_state);
                self.widgets.push(widget);
                self.active_widget_index = self.widgets.len() - 1;
            }
            Event::NewPublisher(new_publisher_event) => {
                let topic = new_publisher_event.topic;
                let message_type = new_publisher_event.message_type;
                let connection = self.connection.clone();
                let topic_publisher_state =
                    TopicPublisherState::new(topic, message_type, connection);
                let widget = Views::TopicPublisher(topic_publisher_state);
                self.widgets.push(widget);
                self.active_widget_index = self.widgets.len() - 1;
            }
            Event::NewNodeDetailView(new_node_event) => {
                let node_name = new_node_event.node;
                let connection = self.connection.clone();
                let node_details_state = NodeDetailState::new(node_name, connection);
                let widget = Views::NodeDetails(node_details_state);
                self.widgets.push(widget);
                self.active_widget_index = self.widgets.len() - 1;
            }
            Event::ClosePopup => {
                self.popup_view = PopupView::None;
                self.needs_redraw = true;
            }
            Event::Error(err_msg) => {
                self.popup_view = PopupView::Error(TextPopup::error(err_msg));
            }
            Event::Key(_) => {}
            Event::None => {}
        }
    }

    fn get_help_test(&self) -> String {
        "App Help:\n\
        - 'Tab': Switch to the next panel.\n\
        - 'Shift+Tab': Switch to the previous panel.\n\
        - 'q' or 'Esc': Exit the application.\n\
        - 'x': Close the current panel (if multiple panels are open).\n\
        - '?': Show this help message."
            .to_string()
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header_area, content_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

        let [tab_area, widget_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(content_area);

        App::render_header(header_area, buf);

        let widget_names = self.widgets.iter().map(|w| w.name());
        Tabs::new(widget_names)
            .highlight_style(SELECTED_STYLE)
            .select(self.active_widget_index)
            .divider(" ")
            .render(tab_area, buf);
        let widget = &mut self.widgets[self.active_widget_index];
        match widget {
            Views::TopicList(topic_list) => {
                TopicList::render(widget_area, buf, topic_list);
            }
            Views::RawMessage(raw_message) => {
                RawMessageWidget::render(widget_area, buf, raw_message);
            }
            Views::LivePlot(live_plot) => {
                LivePlotWidget::render(widget_area, buf, live_plot);
            }
            Views::NodeList(node_list) => {
                NodeListWidget::render(widget_area, buf, node_list);
            }
            Views::TopicPublisher(topic_publisher) => {
                TopicPublisherWidget::render(widget_area, buf, topic_publisher);
            }
            Views::HzPlot(hz_plot_state) => {
                HzPlotWidget::render(widget_area, buf, hz_plot_state);
            }
            Views::NodeDetails(node_details_state) => {
                NodeDetailWidget::render(widget_area, buf, node_details_state);
            }
        }

        let popup_area = Rect {
            x: area.width / 4,
            y: area.height / 4,
            width: area.width / 2,
            height: area.height / 2,
        };
        match &mut self.popup_view {
            PopupView::AddLine(state) => {
                state.render(popup_area, buf);
            }
            PopupView::AddHz(state) => {
                state.render(popup_area, buf);
            }
            PopupView::Error(state) => {
                state.render(popup_area, buf);
            }
            PopupView::None => {}
        }

        // Uncomment to show pressed keys at the bottom for debugging / creating animated GIFs
        // self.render_event(area, buf);
    }
}

/// Rendering logic for the app
impl App {
    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Splinter - ROS2 TUI")
            .bold()
            .centered()
            .render(area, buf);
    }

    pub fn render_event(&self, area: Rect, buf: &mut Buffer) {
        // Display all key events at the bottom in a bordered text box for the animated GIF
        let event_log_area = Rect {
            x: area.x + area.width / 3,
            y: area.y + area.height - 5,
            width: area.width / 3,
            height: 3,
        };
        let event_to_character = |event: &Event| match event {
            Event::Key(CrosstermEvent::Key(key_event)) => match key_event.code {
                KeyCode::Char(c) => c.to_string(),
                KeyCode::Enter => "⏎".to_string(),
                KeyCode::Tab => "⇥".to_string(),
                KeyCode::BackTab => "⇤".to_string(),
                KeyCode::Esc => "⎋".to_string(),
                KeyCode::Backspace => "⌫".to_string(),
                KeyCode::Left => "←".to_string(),
                KeyCode::Right => "→".to_string(),
                KeyCode::Up => "↑".to_string(),
                KeyCode::Down => "↓".to_string(),
                _ => "Unknown".to_string(),
            },
            _ => "".to_string(),
        };
        let recent_events = self
            .metrics
            .events
            .iter()
            .rev()
            .filter(|e| matches!(e, Event::Key(_)))
            .take(event_log_area.width as usize / 2 - 1)
            .map(event_to_character)
            .collect::<Vec<String>>();
        let event_log_text = recent_events
            .into_iter()
            .rev()
            .collect::<Vec<String>>()
            .join(" ");
        Clear.render(event_log_area, buf);
        Paragraph::new(event_log_text)
            .block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title("Event Log"),
            )
            .alignment(Alignment::Right)
            .render(event_log_area, buf);
    }
}
