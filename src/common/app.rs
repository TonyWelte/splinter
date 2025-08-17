use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use clap::Args;
use ratatui::widgets::Tabs;

use crate::common::style::SELECTED_STYLE;
use crate::connections::ros2::ConnectionROS2;
use crate::connections::{Connection, ConnectionType};
use crate::widgets::node_list;
use crate::widgets::{
    live_plot::LivePlotWidget,
    node_list::{NodeListState, NodeListWidget},
    raw_message::RawMessageWidget,
    topic_list::{TopicList, TopicListState},
    TuiWidget, Widgets,
};

use color_eyre::eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    widgets::{Paragraph, Widget},
    DefaultTerminal,
};

struct App {
    should_exit: bool,
    views: Selectable<Widgets>,
}

impl App {
    pub fn new(views: Vec<Widgets>) -> Self {
        let should_exit = false;
        Self {
            should_exit,
            views: Selectable::new(views),
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
                        self.views.remove_selected();
                        return;
                    }
                    KeyCode::Tab => {
                        self.views.next();
                        return;
                    }
                    KeyCode::BackTab => {
                        self.views.previous();
                        return;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        if let Some(widget) = self.widgets[self.active_widget_index].handle_event(event) {
            self.widgets.push(widget);
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
        if let Some(widget) = &mut self.widgets[self.active_widget_index] {
            match widget {
                Widgets::TopicList(topic_list) => {
                    TopicList::render(widget_area, buf, topic_list);
                }
                Widgets::RawMessage(raw_message) => {
                    RawMessageWidget::render(widget_area, buf, raw_message);
                }
                Widgets::LivePlot(live_plot) => {
                    LivePlotWidget::render(widget_area, buf, live_plot);
                }
                Widgets::NodeList(node_list) => {
                    NodeListWidget::render(widget_area, buf, node_list);
                }
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
