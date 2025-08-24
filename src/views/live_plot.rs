use std::{
    cell::RefCell,
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

pub struct LivePlotWidget;

pub struct LivePlotState {
    topic: String,
    selected_fields: Vec<usize>,
    connection: Rc<RefCell<ConnectionType>>,
    plot: Arc<Mutex<Vec<(f64, f64)>>>, // Stores the plots for each field
    max_duration: f64,                 // Maximum duration for the plot
}

fn message_from_bytes(
    raw_message: &[u8],
    message_type_name: &MessageTypeName,
) -> Option<DynamicMessage> {
    DynamicMessageMetadata::new(message_type_name.clone())
        .and_then(|metadata| metadata.create())
        .ok()
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
            topic,
            selected_fields,
            connection,
            plot,
            max_duration: 10.0, // Default maximum duration for the plot
        }
    }
}

fn get_field(message: &GenericMessage, field_index_path: &[usize]) -> Option<f64> {
    if field_index_path.is_empty() {
        return None; // No field index provided
    }
    let field_index = field_index_path.first()?;
    let field = message.get_index(*field_index).unwrap();
    match field {
        GenericField::Simple(SimpleField::Double(value)) => Some(*value),
        GenericField::Simple(SimpleField::Float(value)) => Some(*value as f64),
        GenericField::Simple(SimpleField::Int64(value)) => Some(*value as f64),
        GenericField::Simple(SimpleField::Uint64(value)) => Some(*value as f64),
        GenericField::Simple(SimpleField::Int32(value)) => Some(*value as f64),
        GenericField::Simple(SimpleField::Uint32(value)) => Some(*value as f64),
        GenericField::Simple(SimpleField::Message(msg)) => {
            // If the field is a nested message, recursively get the field value
            get_field(&msg, &field_index_path[1..])
        }
        _ => None, // Handle other types as needed
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
}

impl LivePlotWidget {
    pub fn render(area: Rect, buf: &mut Buffer, state: &mut LivePlotState) {
        {
            // Ensure the plot does not exceed the maximum duration
            let mut plot = state.plot.lock().unwrap();
            let current_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            plot.retain(|&(stamp, _)| current_time - stamp <= state.max_duration);
        }

        let block = Block::bordered()
            .title(Line::raw("Live Plot").centered())
            .border_style(HEADER_STYLE);

        let n_messages = state.plot.lock().unwrap().len();

        let binding = state.plot.lock().unwrap();
        let dataset = Dataset::default()
            .name("Live Data")
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(ratatui::style::Color::Green))
            .data(&binding);

        // Create the X axis and define its properties
        let mut bounds = binding
            .iter()
            .fold(None, |bounds: Option<[f64; 2]>, x| {
                if let Some(mut bounds) = bounds {
                    bounds[0] = bounds[0].min(x.1);
                    bounds[1] = bounds[1].max(x.1);
                    Some(bounds)
                } else {
                    Some([x.1, x.1])
                }
            })
            .unwrap_or([0.0, 10.0]);
        if (bounds[0] - bounds[1]).abs() < 0.001 {
            bounds[0] -= 1.0;
            bounds[1] += 1.0;
        }
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
            .bounds(bounds)
            .labels(
                (0..=5)
                    .map(|i| {
                        format!(
                            "{:.1}",
                            bounds[0] + i as f64 * (bounds[1] - bounds[0]) / 5.0
                        )
                    })
                    .collect::<Vec<_>>(),
            );

        let chart = Chart::new(vec![dataset])
            .x_axis(x_axis)
            .y_axis(y_axis)
            .show_grid(true)
            .block(block);

        Widget::render(chart, area, buf);
    }
}
