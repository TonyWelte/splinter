use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use ratatui::widgets::Tabs;

use crate::common::style::SELECTED_STYLE;
use crate::connections::ros2::ConnectionROS2;
use crate::connections::{Connection, ConnectionType};
use crate::widgets::{
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
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    widgets::{Paragraph, Widget},
    DefaultTerminal,
};

struct App {
    should_exit: bool,
    pub connection: Rc<RefCell<ConnectionType>>,
    widgets: Vec<Views>,
    active_widget_index: usize,
}

impl App {
    pub fn new() -> Self {
        let should_exit = false;
        let connection = Rc::new(RefCell::new(ConnectionType::ROS2(ConnectionROS2::new())));
        let topic_list = TopicListState::new(connection.clone());
        let node_list = NodeListState::new(connection.clone());
        Self {
            should_exit,
            connection,
            widgets: vec![Views::TopicList(topic_list), Views::NodeList(node_list)],
            active_widget_index: 0,
        }
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Ok(is_event_available) = event::poll(Duration::from_millis(100)) {
                if is_event_available {
                    let event = event::read()?;
                    self.handle_event(event);
                }
            }
        }
        Ok(())
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    return;
                }
                match key.code {
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
            _ => {}
        }
        if let Some(widget) = self.widgets[self.active_widget_index].handle_event(event) {
            self.widgets.push(widget);
            self.active_widget_index = self.widgets.len() - 1;
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
