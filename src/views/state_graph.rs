use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, StatefulWidget, Widget},
};

use crate::{
    common::{
        event::Event,
        generic_message::{AnyTypeRef, GenericMessage, MessageMetadata},
        style::HEADER_STYLE,
    },
    connections::{Connection, ConnectionType},
    views::{AcceptsField, FieldInfo, FieldInfoType, FromField, TuiView},
    widgets::state_graph_widget::{
        Axis, StateColorMap, StateDataPoint, StateDataset, StateGraphWidget, StateGraphWidgetState,
    },
};

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEventKind};

/// Shared buffer of (timestamp, state_string) pairs, appended from the subscription callback.
type StateBuffer = Arc<Mutex<Vec<(f64, String)>>>;

/// One tracked field (one lane in the state graph).
struct StateLineState {
    topic: String,
    field_name: String,
    _connection: Rc<RefCell<ConnectionType>>,
    buffer: StateBuffer,
    /// Persistent color map – once a state is assigned a color it keeps it forever.
    color_map: StateColorMap,
}

/// View state that drives the `StateGraphWidget`.
pub struct StateGraphViewState {
    lines: Vec<StateLineState>,
    max_duration: f64,
    label_width: u16,
    scroll_offset: usize,
}

impl StateGraphViewState {
    pub fn new(
        topic: String,
        selected_fields: Vec<usize>,
        field_name: String,
        connection: Rc<RefCell<ConnectionType>>,
    ) -> Self {
        let mut state = Self {
            lines: vec![],
            max_duration: 30.0,
            label_width: 12,
            scroll_offset: 0,
        };
        state.add_line(topic, selected_fields, field_name, connection);
        state
    }

    pub fn add_line(
        &mut self,
        topic: String,
        selected_fields: Vec<usize>,
        field_name: String,
        connection: Rc<RefCell<ConnectionType>>,
    ) {
        let buffer: StateBuffer = Arc::new(Mutex::new(Vec::new()));
        let buffer_clone = buffer.clone();
        let selected_fields_clone = selected_fields.clone();

        connection
            .borrow_mut()
            .subscribe(
                &topic,
                move |msg: GenericMessage, msg_info: MessageMetadata| {
                    let stamp = msg_info
                        .received_time
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .expect("Timestamp before UNIX EPOCH")
                        .as_secs_f64();
                    if let Some(state_str) = get_state_string(&msg, &selected_fields_clone) {
                        let mut buf = buffer_clone.lock().unwrap();
                        // Only record when the state actually changes
                        if buf.last().map_or(true, |(_, prev)| *prev != state_str) {
                            buf.push((stamp, state_str));
                        }
                    }
                },
            )
            .expect("Failed to subscribe to topic");

        // Update the label_width to fit the longest label
        let label_len = format!("{} {}", topic, field_name).len() as u16;
        if label_len > self.label_width {
            self.label_width = label_len;
        }

        self.lines.push(StateLineState {
            topic,
            field_name,
            _connection: connection,
            buffer,
            color_map: StateColorMap::new(),
        });
    }
}

/// Extract a state value as a string from a message field.
/// Supports integer types (rendered as their numeric value) and string fields.
fn get_state_string(message: &GenericMessage, field_index_path: &[usize]) -> Option<String> {
    if field_index_path.is_empty() {
        return None;
    }
    let field = message.get_deep_index(field_index_path).ok()?;
    match field {
        AnyTypeRef::Boolean(v) => Some(if *v { "true" } else { "false" }.to_string()),
        AnyTypeRef::Uint8(v) => Some(v.to_string()),
        AnyTypeRef::Int8(v) => Some(v.to_string()),
        AnyTypeRef::Uint16(v) => Some(v.to_string()),
        AnyTypeRef::Int16(v) => Some(v.to_string()),
        AnyTypeRef::Uint32(v) => Some(v.to_string()),
        AnyTypeRef::Int32(v) => Some(v.to_string()),
        AnyTypeRef::Uint64(v) => Some(v.to_string()),
        AnyTypeRef::Int64(v) => Some(v.to_string()),
        AnyTypeRef::String(v) => Some(v.clone()),
        _ => None,
    }
}

/// Generate evenly-spaced time labels for the X axis, showing relative seconds
/// (e.g. "-30s", "-20s", "-10s", "0s"). Aims for roughly 5 labels.
fn generate_time_labels(max_duration: f64) -> Vec<Span<'static>> {
    let n_labels = 5usize;
    (0..n_labels)
        .map(|i| {
            let frac = i as f64 / (n_labels - 1) as f64;
            let offset = -max_duration * (1.0 - frac);
            let text = if offset.abs() < 0.01 {
                "0s".to_string()
            } else {
                format!("{:.0}s", offset)
            };
            Span::raw(text)
        })
        .collect()
}

// ── TuiView implementation ──────────────────────────────────────────

impl TuiView for StateGraphViewState {
    fn handle_event(&mut self, event: Event) -> Event {
        if let Event::Key(CrosstermEvent::Key(key_event)) = event {
            if key_event.kind != KeyEventKind::Press {
                return event;
            }
            match key_event.code {
                KeyCode::Char('h') | KeyCode::Left => {
                    self.max_duration += 5.0;
                    Event::None
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    if self.max_duration > 5.0 {
                        self.max_duration -= 5.0;
                    }
                    Event::None
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    let max_offset = self.lines.len().saturating_sub(1);
                    if self.scroll_offset < max_offset {
                        self.scroll_offset += 1;
                    }
                    Event::None
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.scroll_offset = self.scroll_offset.saturating_sub(1);
                    Event::None
                }
                _ => event,
            }
        } else {
            event
        }
    }

    fn name(&self) -> String {
        format!("State Graph - {}s", self.max_duration)
    }

    fn get_help_text(&self) -> String {
        "State Graph View Help:\n\
        - 'h' or ←: Increase the time window.\n\
        - 'l' or →: Decrease the time window.\n\
        - 'j' or ↓: Scroll down.\n\
        - 'k' or ↑: Scroll up.\n\n\
        Each row shows a field's state over time.\n\
        Colors change at state transitions."
            .to_string()
    }

    fn needs_redraw(&mut self) -> bool {
        true // Always redraw since data is streaming
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Timestamp before UNIX EPOCH")
            .as_secs_f64();

        let x_min = current_time - self.max_duration;
        let x_max = current_time;

        // Build datasets from live buffers
        let datasets: Vec<StateDataset> = self
            .lines
            .iter_mut()
            .map(|line| {
                let buf_lock = line.buffer.lock().unwrap();
                // Retain only points within the time window (plus one before for continuity)
                let first_visible = buf_lock.iter().rposition(|&(t, _)| t <= x_min).unwrap_or(0);
                let visible_data: Vec<StateDataPoint> = buf_lock[first_visible..]
                    .iter()
                    .map(|(t, s)| StateDataPoint {
                        x: *t,
                        state: s.clone(),
                    })
                    .collect();

                // Pre-register all visible states so the persistent color map
                // assigns them stable colors.
                for dp in &visible_data {
                    line.color_map.color_for(&dp.state);
                }

                StateDataset::new(format!("{} {}", line.topic, line.field_name))
                    .data(visible_data)
                    .color_map(line.color_map.clone())
            })
            .collect();

        let mut widget_state = StateGraphWidgetState::new(datasets);
        widget_state.scroll_offset = self.scroll_offset;

        let block = Block::bordered()
            .title(Line::raw("State Graph").centered())
            .border_style(HEADER_STYLE)
            .border_type(BorderType::Rounded);

        // Build X-axis labels showing relative time
        let x_axis_labels = generate_time_labels(self.max_duration);
        let x_axis = Axis::new()
            .labels(x_axis_labels)
            .style(Style::default().fg(Color::DarkGray));

        let widget = StateGraphWidget::new()
            .block(block)
            .x_bounds([x_min, x_max])
            .label_width(self.label_width)
            .x_axis(x_axis);

        widget.render(area, buf, &mut widget_state);
    }

    fn as_field_acceptor(&mut self) -> Option<&mut dyn AcceptsField> {
        Some(self)
    }
}

// ── FromField / AcceptsField ────────────────────────────────────────

impl FromField for StateGraphViewState {
    fn from_field(field_info: FieldInfo) -> Self {
        StateGraphViewState::new(
            field_info.topic,
            field_info.field,
            field_info.field_name,
            field_info.connection,
        )
    }
}

impl AcceptsField for StateGraphViewState {
    fn accepts_field(&mut self, field_info: FieldInfo) {
        self.add_line(
            field_info.topic,
            field_info.field,
            field_info.field_name,
            field_info.connection,
        );
    }

    fn accepts_field_type(&self, field_type: &FieldInfoType) -> bool {
        matches!(field_type, FieldInfoType::Integer | FieldInfoType::String)
    }
}
