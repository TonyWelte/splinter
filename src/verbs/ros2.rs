use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use ratatui::text::Line;
use ratatui::widgets::{StatefulWidget, Tabs};

use crate::common::event::{Event, NewGraphLineEvent};
use crate::common::style::SELECTED_STYLE;
use crate::connections::ros2::ConnectionROS2;
use crate::connections::{Connection, ConnectionType};
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

use color_eyre::eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    widgets::{Paragraph, Widget},
    DefaultTerminal,
};

enum PopupView {
    None,
    AddGraphLine(AddGraphLineState),
}

struct AddGraphLineState {
    topic: String,
    field: Vec<usize>,
    candidate_views: Vec<usize>,
    selected_index: usize, // Index out of candidate_views means "Create new view"
}

impl AddGraphLineState {
    pub fn new(topic: String, field: Vec<usize>, candidate_views: Vec<usize>) -> Self {
        Self {
            topic,
            field,
            candidate_views,
            selected_index: 0,
        }
    }

    pub fn handle_event(&mut self, event: Event) -> Event {
        if let Event::Key(CrosstermEvent::Key(key_event)) = event {
            if key_event.kind != KeyEventKind::Press {
                return event;
            }
            match key_event.code {
                KeyCode::Char('k') | KeyCode::Up => {
                    if self.selected_index == 0 {
                        self.selected_index = self.candidate_views.len();
                    } else {
                        self.selected_index -= 1;
                    }
                    return Event::None;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.selected_index =
                        (self.selected_index + 1) % (self.candidate_views.len() + 1);
                    return Event::None;
                }
                KeyCode::Enter => {
                    if self.selected_index == self.candidate_views.len() {
                        return Event::NewGraph(NewGraphLineEvent {
                            topic: self.topic.clone(),
                            field: self.field.clone(),
                            view: None,
                        });
                    } else {
                        return Event::NewGraphLine(NewGraphLineEvent {
                            topic: self.topic.clone(),
                            field: self.field.clone(),
                            view: Some(self.candidate_views[self.selected_index]),
                        });
                    }
                }
                KeyCode::Esc => {
                    return Event::None;
                }
                _ => {}
            }
        }
        Event::None
    }
}

struct AddGraphLinePopup;

impl AddGraphLinePopup {}

impl StatefulWidget for AddGraphLinePopup {
    type State = AddGraphLineState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = ratatui::widgets::Block::default()
            .title(Line::raw("Add to Graph").centered())
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(SELECTED_STYLE);

        let inner_area = block.inner(area);
        block.render(area, buf);

        let options: Vec<String> = state
            .candidate_views
            .iter()
            .map(|i| format!("Add to View {}", i + 1))
            .chain(std::iter::once("Create New View".to_string()))
            .collect();

        let options_widget = Tabs::new(options)
            .select(state.selected_index)
            .block(ratatui::widgets::Block::default())
            .highlight_style(SELECTED_STYLE)
            .divider(" | ");

        options_widget.render(inner_area, buf);
    }
}

struct App {
    should_exit: bool,
    pub connection: Rc<RefCell<ConnectionType>>,
    widgets: Vec<Views>,
    active_widget_index: usize,

    popup_view: PopupView,
}

impl App {
    pub fn new() -> Self {
        let should_exit = false;
        let connection = Rc::new(RefCell::new(ConnectionType::ROS2(ConnectionROS2::new())));
        let topic_list = TopicListState::new(connection.clone());
        let node_list = NodeListState::new(connection.clone());
        let topic_publisher = TopicPublisherState::new("chatter".to_string(), connection.clone());
        Self {
            should_exit,
            connection,
            widgets: vec![
                Views::TopicList(topic_list),
                Views::NodeList(node_list),
                Views::TopicPublisher(topic_publisher),
            ],
            active_widget_index: 0,
            popup_view: PopupView::None,
        }
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Ok(is_event_available) = event::poll(Duration::from_millis(100)) {
                if is_event_available {
                    let event = event::read()?;
                    self.handle_event(Event::Key(event));
                }
            }
        }
        Ok(())
    }

    fn handle_event(&mut self, event: Event) {
        let event = match &mut self.popup_view {
            PopupView::None => self.widgets[self.active_widget_index].handle_event(event),
            PopupView::AddGraphLine(data) => {
                let new_event = data.handle_event(event);
                match new_event {
                    Event::None => Event::None,
                    other_event => {
                        self.popup_view = PopupView::None;
                        other_event
                    }
                }
            }
        };

        match event {
            Event::Key(CrosstermEvent::Key(key_event)) => {
                if key_event.kind != KeyEventKind::Press {
                    return;
                }
                match key_event.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.should_exit = true;
                        return;
                    }
                    KeyCode::Char('x') => {
                        if self.widgets.len() > 1 {
                            self.widgets.remove(self.active_widget_index);
                            if self.active_widget_index != 0 {
                                self.active_widget_index -= 1;
                            }
                        }
                        return;
                    }
                    KeyCode::Tab => {
                        self.active_widget_index =
                            (self.active_widget_index + 1) % self.widgets.len();
                        return;
                    }
                    KeyCode::BackTab => {
                        if self.active_widget_index == 0 {
                            self.active_widget_index = self.widgets.len() - 1;
                        } else {
                            self.active_widget_index -= 1;
                        }
                        return;
                    }
                    _ => {}
                }
            }
            Event::NewGraphLine(new_graph_event) => {
                if new_graph_event.view.is_some() {
                    let topic = new_graph_event.topic;
                    let field = new_graph_event.field;
                    let view = new_graph_event.view.unwrap();
                    let connection = self.connection.clone();
                    if let Some(Views::LivePlot(live_plot_state)) = self.widgets.get_mut(view) {
                        live_plot_state.add_graph_line(topic, field, connection);
                    }
                    return;
                }
                let topic = new_graph_event.topic;
                let field = new_graph_event.field;
                let connection = self.connection.clone();
                let candidate_views: Vec<usize> = self
                    .widgets
                    .iter()
                    .enumerate()
                    .filter_map(|(i, w)| match w {
                        Views::LivePlot(_) => Some(i),
                        _ => None,
                    })
                    .collect();
                self.popup_view =
                    PopupView::AddGraphLine(AddGraphLineState::new(topic, field, candidate_views));
                return;
            }
            Event::NewGraph(new_graph_event) => {
                let topic = new_graph_event.topic;
                let field = new_graph_event.field;
                let connection = self.connection.clone();
                let live_plot_state = LivePlotState::new(topic, field, connection);
                let widget = Views::LivePlot(live_plot_state);
                self.widgets.push(widget);
                self.active_widget_index = self.widgets.len() - 1;
            }
            Event::NewTopic(new_topic_event) => {
                let topic = new_topic_event.topic;
                let connection = self.connection.clone();
                let raw_message_state = RawMessageState::new(topic, connection);
                let widget = Views::RawMessage(raw_message_state);
                self.widgets.push(widget);
                self.active_widget_index = self.widgets.len() - 1;
            }
            Event::NewHzPlot(new_topic_event) => {
                let topic = new_topic_event.topic;
                let connection = self.connection.clone();
                let hz_plot_state = HzPlotState::new(topic, connection);
                let widget = Views::HzPlot(hz_plot_state);
                self.widgets.push(widget);
                self.active_widget_index = self.widgets.len() - 1;
            }
            Event::Key(_) => {}
            Event::None => {}
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header_area, content_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [tab_area, widget_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(content_area);

        App::render_header(header_area, buf);
        App::render_footer(footer_area, buf);

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
        }

        if let PopupView::AddGraphLine(state) = &mut self.popup_view {
            let popup_area = Rect {
                x: area.width / 4,
                y: area.height / 4,
                width: area.width / 2,
                height: 3,
            };
            AddGraphLinePopup.render(popup_area, buf, state);
        }
    }
}

/// Rendering logic for the app
impl App {
    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("ros2tui - MCAP Viewer")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new(
            "Use Tab to switch active pannel, ↓↑ to move, → for actions, q or Esc to exit",
        )
        .centered()
        .render(area, buf);
    }
}

pub fn run() -> color_eyre::eyre::Result<()> {
    let app = App::new();

    color_eyre::install()?;
    let terminal = ratatui::init();
    app.run(terminal)?;
    ratatui::restore();

    Ok(())
}
