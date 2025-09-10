use std::{
    cell::RefCell,
    iter::zip,
    rc::Rc,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use ratatui::{
    prelude::{Buffer, Rect, Style, Stylize},
    symbols::Marker,
    text::Line,
    widgets::{Axis, Block, Chart, Dataset, GraphType, Widget},
};
use rclrs::*;

use crate::{
    common::{
        event::Event,
        generic_message::{GenericField, GenericMessage, MessageMetadata, SimpleField},
        style::HEADER_STYLE,
    },
    connections::{Connection, ConnectionType},
    // generic_message::{GenericField, GenericMessage},
    views::TuiView,
};

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEventKind};

pub struct HzPlotWidget;

pub struct HzLineState {
    topic: String,

    stamps: Vec<f64>,
    plot: Vec<(f64, f64)>, // Stores the plots for each field
}

impl HzLineState {
    fn new(topic: String) -> Self {
        Self {
            topic,
            stamps: Vec::new(),
            plot: Vec::new(),
        }
    }
}

pub struct HzPlotState {
    connection: Rc<RefCell<ConnectionType>>,
    lines: Vec<Arc<Mutex<HzLineState>>>,
    max_duration: f64, // Maximum duration for the plot
}

impl HzPlotState {
    pub fn new(topic: String, connection: Rc<RefCell<ConnectionType>>) -> Self {
        let line_state = Arc::new(Mutex::new(HzLineState::new(topic.clone())));
        let line_state_copy = line_state.clone();
        connection
            .borrow_mut()
            .subscribe(
                &topic,
                move |_: GenericMessage, msg_info: MessageMetadata| {
                    let mut mut_line_state = line_state.lock().unwrap();
                    let stamp = msg_info
                        .received_time
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64();
                    mut_line_state.stamps.push(stamp);
                    let window_length = 10; // Number of stamps to consider for frequency calculation
                    if mut_line_state.stamps.len() > window_length {
                        mut_line_state.stamps.remove(0);
                    }
                    if mut_line_state.stamps.len() == window_length {
                        let duration = mut_line_state.stamps.last().unwrap()
                            - mut_line_state.stamps.first().unwrap();
                        if duration > 0.0 {
                            let frequency = (window_length - 1) as f64 / duration;
                            mut_line_state.plot.push((stamp, frequency));
                        }
                    }
                },
            )
            .expect("Failed to subscribe to topic");
        Self {
            lines: vec![line_state_copy],
            max_duration: 10.0, // Default maximum duration for the plot
            connection,
        }
    }

    pub fn add_line(&mut self, topic: String, connection: Rc<RefCell<ConnectionType>>) {
        let line_state = Arc::new(Mutex::new(HzLineState::new(topic.clone())));
        let line_state_copy = line_state.clone();
        connection
            .borrow_mut()
            .subscribe(
                &topic,
                move |_: GenericMessage, msg_info: MessageMetadata| {
                    let mut mut_line_state = line_state.lock().unwrap();
                    let stamp = msg_info
                        .received_time
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64();
                    mut_line_state.stamps.push(stamp);
                    let window_length = 10; // Number of stamps to consider for frequency calculation
                    if mut_line_state.stamps.len() > window_length {
                        mut_line_state.stamps.remove(0);
                    }
                    if mut_line_state.stamps.len() == window_length {
                        let duration = mut_line_state.stamps.last().unwrap()
                            - mut_line_state.stamps.first().unwrap();
                        if duration > 0.0 {
                            let frequency = (window_length - 1) as f64 / duration;
                            mut_line_state.plot.push((stamp, frequency));
                        }
                    }
                },
            )
            .expect("Failed to subscribe to topic");
        self.lines.push(line_state_copy);
    }
}

impl TuiView for HzPlotState {
    fn handle_event(&mut self, event: Event) -> Event {
        if let Event::Key(CrosstermEvent::Key(key_event)) = event {
            if key_event.kind != KeyEventKind::Press {
                return event;
            }
            match key_event.code {
                KeyCode::Char('h') | KeyCode::Left => {
                    self.max_duration += 1.0; // Increase the maximum duration
                    Event::None
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    if self.max_duration > 1.0 {
                        self.max_duration -= 1.0; // Decrease the maximum duration
                    }
                    Event::None
                }
                _ => event,
            }
        } else {
            event
        }
    }

    fn name(&self) -> String {
        format!("Frequency - {}s - window: 10 msg", self.max_duration)
    }

    fn get_help_text(&self) -> String {
        "Hz Plot View Help:\n\
        - 'h' or ←: Increase the time window for the frequency plot.\n\
        - 'l' or →: Decrease the time window for the frequency plot."
            .to_string()
    }
}

impl HzPlotWidget {
    pub fn render(area: Rect, buf: &mut Buffer, state: &mut HzPlotState) {
        for line in &state.lines {
            // Ensure the plot does not exceed the maximum duration
            let mut hz_line = line.lock().unwrap();
            let current_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            hz_line
                .plot
                .retain(|&(stamp, _)| current_time - stamp <= state.max_duration);
        }

        let block = Block::bordered()
            .title(Line::raw("Frequency Plot").centered())
            .border_style(HEADER_STYLE);

        // Find the overall bounds for Y axis
        let mut max_y = f64::MIN;
        for line in &state.lines {
            let hz_line = line.lock().unwrap();
            for &(_, y) in hz_line.plot.iter() {
                if y > max_y {
                    max_y = y;
                }
            }
        }
        if max_y == f64::MIN {
            max_y = 10.0;
        }
        if max_y.abs() < 0.001 {
            max_y += 1.0;
        }

        let bindings = state
            .lines
            .iter()
            .map(|hz_line| hz_line.lock().unwrap().plot.clone())
            .collect::<Vec<_>>();
        let datasets = zip(state.lines.iter(), bindings.iter())
            .enumerate()
            .map(|(i, (line, plot))| {
                Dataset::default()
                    .name(format!("{}", line.lock().unwrap().topic,))
                    .marker(Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(match i % 6 {
                        0 => ratatui::style::Color::Red,
                        1 => ratatui::style::Color::Green,
                        2 => ratatui::style::Color::Yellow,
                        3 => ratatui::style::Color::Blue,
                        4 => ratatui::style::Color::Magenta,
                        _ => ratatui::style::Color::Cyan,
                    }))
                    .data(&plot)
            })
            .collect::<Vec<_>>();

        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let x_axis = Axis::default()
            .style(Style::default().white())
            .bounds([current_time - state.max_duration, current_time])
            .labels(
                (0..=5)
                    .rev()
                    .map(|i| format!("{:.1}", current_time - i as f64 * state.max_duration / 5.0))
                    .collect::<Vec<_>>(),
            );

        // Create the Y axis and define its properties
        let y_axis = Axis::default()
            .style(Style::default().white())
            .bounds([0.0, max_y])
            .labels(
                (0..=5)
                    .map(|i| format!("{:.1}", i as f64 * (max_y) / 5.0))
                    .collect::<Vec<_>>(),
            );

        let chart = Chart::new(datasets)
            .x_axis(x_axis)
            .y_axis(y_axis)
            .show_grid(true)
            .block(block);

        Widget::render(chart, area, buf);
    }
}
