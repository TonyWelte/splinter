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
    widgets::{Axis, Block, BorderType, Chart, Dataset, GraphType, Widget},
};
use rclrs::*;

use crate::{
    common::{
        event::Event,
        generic_message::{
            AnyTypeRef, GenericField, GenericMessage, Length, MessageMetadata, SimpleField,
        },
        style::HEADER_STYLE,
    },
    connections::{Connection, ConnectionType},
    // generic_message::{GenericField, GenericMessage},
    views::TuiView,
};

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEventKind};

pub struct LivePlotWidget;

pub struct GraphLineState {
    topic: String,
    selected_fields: Vec<usize>,
    connection: Rc<RefCell<ConnectionType>>,
    plot: Arc<Mutex<Vec<(f64, f64)>>>, // Stores the plots for each field
}

pub struct LivePlotState {
    lines: Vec<GraphLineState>,
    max_duration: f64, // Maximum duration for the plot
}

impl LivePlotState {
    pub fn new(
        topic: String,
        selected_fields: Vec<usize>,
        connection: Rc<RefCell<ConnectionType>>,
    ) -> Self {
        let plot = Arc::new(Mutex::new(Vec::new()));
        let plot_copy = plot.clone();
        let selected_fields_copy = selected_fields.clone();
        connection
            .borrow_mut()
            .subscribe(
                &topic,
                move |msg: GenericMessage, msg_info: MessageMetadata| {
                    let mut mut_plot = plot_copy.lock().unwrap();
                    let stamp = msg_info
                        .received_time
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64();
                    let value = get_field(&msg, &selected_fields_copy).unwrap();
                    mut_plot.push((stamp, value));
                },
            )
            .expect("Failed to subscribe to topic");
        Self {
            lines: vec![GraphLineState {
                topic,
                selected_fields,
                connection,
                plot,
            }],
            max_duration: 10.0, // Default maximum duration for the plot
        }
    }

    pub fn add_graph_line(
        &mut self,
        topic: String,
        selected_fields: Vec<usize>,
        connection: Rc<RefCell<ConnectionType>>,
    ) {
        let plot = Arc::new(Mutex::new(Vec::new()));
        let plot_copy = plot.clone();
        let selected_fields_copy = selected_fields.clone();
        connection
            .borrow_mut()
            .subscribe(
                &topic,
                move |msg: GenericMessage, msg_info: MessageMetadata| {
                    let mut mut_plot = plot_copy.lock().unwrap();
                    let stamp = msg_info
                        .received_time
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64();
                    let value = get_field(&msg, &selected_fields_copy).unwrap();
                    mut_plot.push((stamp, value));
                },
            )
            .expect("Failed to subscribe to topic");
        self.lines.push(GraphLineState {
            topic,
            selected_fields,
            connection,
            plot,
        });
    }
}

fn get_field(message: &GenericMessage, field_index_path: &[usize]) -> Option<f64> {
    if field_index_path.is_empty() {
        return None; // No field index provided
    }
    let field = message.get_deep_index(field_index_path).unwrap();
    match field {
        AnyTypeRef::Float(v) => Some(*v as f64),
        AnyTypeRef::Double(v) => Some(*v as f64),
        AnyTypeRef::Boolean(v) => {
            if *v {
                Some(1.0)
            } else {
                Some(0.0)
            }
        }
        AnyTypeRef::Uint8(v) => Some(*v as f64),
        AnyTypeRef::Int8(v) => Some(*v as f64),
        AnyTypeRef::Uint16(v) => Some(*v as f64),
        AnyTypeRef::Int16(v) => Some(*v as f64),
        AnyTypeRef::Uint32(v) => Some(*v as f64),
        AnyTypeRef::Int32(v) => Some(*v as f64),
        AnyTypeRef::Uint64(v) => Some(*v as f64),
        AnyTypeRef::Int64(v) => Some(*v as f64),
        AnyTypeRef::String(v) => Some(v.len() as f64),
        AnyTypeRef::Array(v) => Some(v.len() as f64),
        AnyTypeRef::Sequence(v) => Some(v.len() as f64),
        AnyTypeRef::BoundedSequence(v) => Some(v.len() as f64),
    }
}

impl TuiView for LivePlotState {
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
        format!("Live Data - {}s", self.max_duration)
    }

    fn get_help_text(&self) -> String {
        "Live Plot View Help:\n\
        - 'h' or ←: Increase the time window for the live plot.\n\
        - 'l' or →: Decrease the time window for the live plot."
            .to_string()
    }
}

impl LivePlotWidget {
    pub fn render(area: Rect, buf: &mut Buffer, state: &mut LivePlotState) {
        for line in &state.lines {
            // Ensure the plot does not exceed the maximum duration
            let mut plot = line.plot.lock().unwrap();
            let current_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            plot.retain(|&(stamp, _)| current_time - stamp <= state.max_duration);
        }

        let block = Block::bordered()
            .title(Line::raw("Live Plot").centered())
            .border_style(HEADER_STYLE)
            .border_type(BorderType::Rounded);

        // Find the overall bounds for Y axis
        let (mut min_y, mut max_y) = (f64::MAX, f64::MIN);
        for line in &state.lines {
            let plot = line.plot.lock().unwrap();
            for &(_, y) in plot.iter() {
                if y < min_y {
                    min_y = y;
                }
                if y > max_y {
                    max_y = y;
                }
            }
        }
        if min_y == f64::MAX || max_y == f64::MIN {
            min_y = 0.0;
            max_y = 10.0;
        }
        if (min_y - max_y).abs() < 0.001 {
            min_y -= 1.0;
            max_y += 1.0;
        }

        let bindings = state
            .lines
            .iter()
            .map(|line| line.plot.lock().unwrap())
            .collect::<Vec<_>>();
        let datasets = zip(state.lines.iter(), bindings.iter())
            .enumerate()
            .map(|(i, (line, plot))| {
                Dataset::default()
                    .name(format!(
                        "{} {:?}",
                        line.topic,
                        line.selected_fields
                            .iter()
                            .map(|idx| idx.to_string())
                            .collect::<Vec<_>>()
                    ))
                    .marker(Marker::Dot)
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
            .bounds([min_y, max_y])
            .labels(
                (0..=5)
                    .map(|i| format!("{:.1}", min_y + i as f64 * (max_y - min_y) / 5.0))
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
