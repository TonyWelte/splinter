use std::collections::HashMap;

use ratatui::{
    prelude::{BlockExt, Buffer, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, StatefulWidget, Widget},
};

/// A single data point: x position and a state value.
#[derive(Debug, Clone)]
pub struct StateDataPoint {
    pub x: f64,
    pub state: String,
}

/// A data source (one horizontal lane in the graph).
#[derive(Debug, Clone)]
pub struct StateDataset {
    /// Label shown on the y-axis for this lane.
    pub label: String,
    /// Ordered sequence of (x, state) data points. Must be sorted by x.
    pub data: Vec<StateDataPoint>,
    /// Color mapping for this dataset's states (independent from other datasets).
    pub color_map: StateColorMap,
}

impl StateDataset {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            data: Vec::new(),
            color_map: StateColorMap::new(),
        }
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn data(mut self, data: Vec<StateDataPoint>) -> Self {
        self.data = data;
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn color_map(mut self, color_map: StateColorMap) -> Self {
        self.color_map = color_map;
        self
    }
}

/// Maps state values (strings) to colors.
#[derive(Debug, Clone, Default)]
pub struct StateColorMap {
    map: HashMap<String, Color>,
    default_palette: Vec<Color>,
}

impl StateColorMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            default_palette: vec![
                Color::Green,
                Color::Red,
                Color::Yellow,
                Color::Blue,
                Color::Magenta,
                Color::Cyan,
                Color::LightGreen,
                Color::LightRed,
                Color::LightYellow,
                Color::LightBlue,
                Color::LightMagenta,
                Color::LightCyan,
            ],
        }
    }

    /// Explicitly assign a color to a state value.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn map(mut self, state: impl Into<String>, color: Color) -> Self {
        self.map.insert(state.into(), color);
        self
    }

    /// Get the color for a state value. If not explicitly mapped, assign one
    /// from the default palette based on insertion order.
    pub fn color_for(&mut self, state: &str) -> Color {
        if let Some(&color) = self.map.get(state) {
            return color;
        }
        let idx = self.map.len() % self.default_palette.len();
        let color = self.default_palette[idx];
        self.map.insert(state.to_string(), color);
        color
    }
}

/// X-axis configuration (similar to the Chart widget).
#[derive(Debug, Clone, Default)]
pub struct Axis<'a> {
    /// Labels to display along the axis. First is placed at the left edge,
    /// last at the right edge, and the rest are evenly distributed in between.
    labels: Vec<Span<'a>>,
    /// Style for the axis line and tick marks.
    style: Style,
}

impl<'a> Axis<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the axis labels. Must have at least 2 labels for ticks to render.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn labels(mut self, labels: Vec<Span<'a>>) -> Self {
        self.labels = labels;
        self
    }

    /// Set the style for the axis line and tick marks.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

/// Controls vertical spacing between dataset rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Spacing {
    /// No blank lines between rows (original behaviour).
    Dense,
    /// One blank line between each row.
    Spaced,
    /// Use `Spaced` if all datasets fit, otherwise fall back to `Dense`.
    #[default]
    Auto,
}

/// The widget itself (stateless rendering config, following Ratatui conventions).
pub struct StateGraphWidget<'a> {
    block: Option<Block<'a>>,
    /// Visible x-axis range [min, max].
    x_bounds: [f64; 2],
    /// Width reserved for the y-axis labels (in columns).
    label_width: u16,
    /// Vertical spacing between dataset rows.
    spacing: Spacing,
    /// Optional X-axis with labels and tick marks.
    x_axis: Option<Axis<'a>>,
}

impl<'a> Default for StateGraphWidget<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StateGraphWidget<'a> {
    pub fn new() -> Self {
        Self {
            block: None,
            x_bounds: [0.0, 100.0],
            label_width: 12,
            spacing: Spacing::default(),
            x_axis: None,
        }
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn spacing(mut self, spacing: Spacing) -> Self {
        self.spacing = spacing;
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn x_bounds(mut self, bounds: [f64; 2]) -> Self {
        self.x_bounds = bounds;
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn label_width(mut self, width: u16) -> Self {
        self.label_width = width;
        self
    }

    /// Set the X-axis configuration (labels, tick marks, title).
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn x_axis(mut self, axis: Axis<'a>) -> Self {
        self.x_axis = Some(axis);
        self
    }
}

/// State for the StateGraphWidget (following StatefulWidget pattern).
pub struct StateGraphWidgetState {
    pub datasets: Vec<StateDataset>,
    pub scroll_offset: usize,
    needs_redraw: bool,
}

impl Default for StateGraphWidgetState {
    fn default() -> Self {
        Self {
            datasets: Vec::new(),
            scroll_offset: 0,
            needs_redraw: true,
        }
    }
}

impl StateGraphWidgetState {
    pub fn new(datasets: Vec<StateDataset>) -> Self {
        Self {
            datasets,
            scroll_offset: 0,
            needs_redraw: true,
        }
    }

    pub fn update(&mut self, datasets: Vec<StateDataset>) {
        self.datasets = datasets;
        self.needs_redraw = true;
    }

    pub fn needs_redraw(&mut self) -> bool {
        if self.needs_redraw {
            self.needs_redraw = false;
            true
        } else {
            false
        }
    }
}

/// The 9 left-aligned block elements, from empty to full block.
/// Index 0 = empty, index 1 = 1/8 filled, index 8 = fully filled.
const LEFT_BLOCKS: [char; 9] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];

/// Pick the left-block character for a fractional fill (0.0 – 1.0).
fn left_block_char(fraction: f64) -> char {
    let level = (fraction * 8.0).round() as usize;
    LEFT_BLOCKS[level.min(8)]
}

/// A state transition at a fractional column position within the timeline.
struct StateTransition {
    /// Fractional column where this state begins (sub-character precision).
    col: f64,
    /// The state value that starts at this column.
    state: String,
}

/// A contiguous segment of a single state within the timeline.
struct TimelineSegment<'a> {
    /// Fractional column where this segment begins.
    start: f64,
    /// Fractional column where this segment ends.
    end: f64,
    /// The state value for this segment.
    state: &'a str,
}

/// An ordered sequence of state transitions within a timeline of a given width.
/// Each state implicitly extends from its `col` to the next transition's `col`
/// (or to `width` for the last transition). Non-overlapping by construction.
struct Timeline {
    transitions: Vec<StateTransition>,
    /// Total width of the timeline in fractional columns.
    width: f64,
}

impl Timeline {
    /// Start column of the transition at index `i`.
    #[inline]
    fn transition_start(&self, i: usize) -> f64 {
        self.transitions[i].col
    }

    /// End column of the transition at index `i`.
    #[inline]
    fn transition_end(&self, i: usize) -> f64 {
        if i + 1 < self.transitions.len() {
            self.transitions[i + 1].col
        } else {
            self.width
        }
    }

    /// Iterate over contiguous segments, each carrying its start, end, and state.
    fn segments(&self) -> impl Iterator<Item = TimelineSegment<'_>> {
        self.transitions
            .iter()
            .enumerate()
            .map(|(i, t)| TimelineSegment {
                start: t.col,
                end: self.transition_end(i),
                state: &t.state,
            })
    }
}

/// Pre-computed layout dimensions for rendering the state graph.
struct GraphLayout {
    /// Full inner area (may include X-axis rows).
    inner: Rect,
    /// Width of the y-axis label column.
    label_w: u16,
    /// Row stride: 1 for Dense, 2 for Spaced.
    row_stride: u16,
    /// Number of datasets that fit in the visible height.
    visible_count: usize,
    /// Whether the X-axis has enough labels to render.
    has_axis_labels: bool,
    /// Visible x-axis bounds [min, max].
    x_bounds: [f64; 2],
}

impl GraphLayout {
    /// Area available for dataset rows (excludes X-axis rows).
    #[inline]
    fn graph_area(&self) -> Rect {
        Rect {
            height: self.inner.height.saturating_sub(self.axis_rows()),
            ..self.inner
        }
    }

    /// Width of the timeline area.
    #[inline]
    fn timeline_w(&self) -> u16 {
        self.inner.width.saturating_sub(self.label_w + 1)
    }

    /// X coordinate where the timeline starts.
    #[inline]
    fn timeline_x(&self) -> u16 {
        self.inner.x + self.label_w + 1
    }

    /// X coordinate of the y-axis separator.
    #[inline]
    fn sep_x(&self) -> u16 {
        self.inner.x + self.label_w
    }

    /// Rows reserved for the X-axis (0 or 2).
    #[inline]
    fn axis_rows(&self) -> u16 {
        if self.has_axis_labels {
            2
        } else {
            0
        }
    }

    /// Data range along the x-axis.
    #[inline]
    fn x_range(&self) -> f64 {
        self.x_bounds[1] - self.x_bounds[0]
    }
}

fn compute_layout(
    widget: &StateGraphWidget,
    inner: Rect,
    state: &StateGraphWidgetState,
) -> GraphLayout {
    let has_axis_labels = widget.x_axis.as_ref().is_some_and(|a| a.labels.len() >= 2);

    // The label column width (clamped to leave room for the separator).
    let label_w = widget.label_width.min(inner.width.saturating_sub(1));

    // Resolve spacing mode
    let graph_height = inner
        .height
        .saturating_sub(if has_axis_labels { 2 } else { 0 });
    let total_datasets = state.datasets.len();
    let height = graph_height as usize;
    let use_spaced = match widget.spacing {
        Spacing::Dense => false,
        Spacing::Spaced => true,
        Spacing::Auto => {
            // Spaced needs: n rows + (n-1) blank lines = 2n - 1 rows.
            // Fall back to Dense if that doesn't fit.
            total_datasets == 0 || (2 * total_datasets - 1) <= height
        }
    };
    let row_stride: u16 = if use_spaced { 2 } else { 1 };

    // How many datasets fit in the available height?
    let visible_count = if use_spaced {
        // n datasets need 2n - 1 rows → n = (height + 1) / 2
        (height + 1) / 2
    } else {
        height
    };

    GraphLayout {
        inner,
        label_w,
        row_stride,
        visible_count,
        has_axis_labels,
        x_bounds: widget.x_bounds,
    }
}

fn render_y_separator(buf: &mut Buffer, layout: &GraphLayout) {
    let sep_style = Style::default().fg(Color::DarkGray);
    let graph_area = layout.graph_area();
    let sep_x = layout.sep_x();
    for row in 0..graph_area.height {
        buf[(sep_x, graph_area.y + row)]
            .set_symbol(symbols::line::VERTICAL)
            .set_style(sep_style);
    }
    if layout.has_axis_labels {
        let axis_line_y = graph_area.y + graph_area.height;
        if axis_line_y < layout.inner.y + layout.inner.height {
            buf[(sep_x, axis_line_y)]
                .set_symbol(symbols::line::BOTTOM_LEFT)
                .set_style(sep_style);
        }
    }
}

fn render_dataset_label(buf: &mut Buffer, dataset: &StateDataset, layout: &GraphLayout, y: u16) {
    let label: String = dataset
        .label
        .chars()
        .take(layout.label_w as usize)
        .collect();
    let label_padding = layout.label_w as usize - label.chars().count();
    let label_span = Span::raw(format!("{}{}", " ".repeat(label_padding), label))
        .style(Style::default().fg(Color::White));
    let label_area = Rect {
        x: layout.inner.x,
        y,
        width: layout.label_w,
        height: 1,
    };
    label_span.render(label_area, buf);
}

fn build_timeline(
    data: &[StateDataPoint],
    x_bounds: [f64; 2],
    x_range: f64,
    timeline_w: f64,
) -> Timeline {
    let transitions = data
        .iter()
        .map(|point| StateTransition {
            col: ((point.x - x_bounds[0]) / x_range * timeline_w).clamp(0.0, timeline_w),
            state: point.state.clone(),
        })
        .collect();
    Timeline {
        transitions,
        width: timeline_w,
    }
}

/// Render a dataset row's timeline using a sweep cursor across transitions.
///
/// A cursor `ti` tracks the first transition whose span hasn't ended yet.
/// It only advances forward, so total work is O(W + T) instead of O(W × T).
fn render_timeline(
    buf: &mut Buffer,
    timeline: &Timeline,
    color_map: &mut StateColorMap,
    layout: &GraphLayout,
    y: u16,
) {
    let timeline_w = layout.timeline_w();
    let timeline_x = layout.timeline_x();
    let n = timeline.transitions.len();
    if n == 0 || timeline_w == 0 {
        return;
    }

    // `ti` is the first transition whose span hasn't ended before the
    // current column.
    let mut ti = 0usize;

    for col in 0..timeline_w {
        let col_f = col as f64;
        let col_end_f = col_f + 1.0;

        // Advance cursor past transitions whose spans ended before this column.
        while ti < n && timeline.transition_end(ti) <= col_f {
            ti += 1;
        }
        if ti >= n {
            break;
        }
        // If the first candidate starts at or past the right edge, nothing to draw.
        if timeline.transition_start(ti) >= col_end_f {
            continue;
        }

        // Count consecutive overlapping transitions from `ti`.
        // Because spans are contiguous, these are the only ones that can overlap.
        let mut count = 0usize;
        while ti + count < n && timeline.transition_start(ti + count) < col_end_f {
            count += 1;
        }

        let cell = &mut buf[(timeline_x + col, y)];

        if count > 2 {
            // 3+ states in one character → shade
            cell.set_char('░').set_fg(Color::DarkGray);
        } else if count == 2 {
            // Two states share this cell → partial-block transition
            let first_end = timeline.transition_end(ti);
            let first_color = color_map.color_for(timeline.transitions[ti].state.as_str());
            let second_color = color_map.color_for(timeline.transitions[ti + 1].state.as_str());

            let boundary = first_end.clamp(col_f, col_end_f);
            let first_fraction = boundary - col_f;
            let block_char = left_block_char(first_fraction);
            cell.set_char(block_char)
                .set_fg(first_color)
                .set_bg(second_color);
        } else {
            // Single state in this cell
            let start = timeline.transition_start(ti);
            let color = color_map.color_for(timeline.transitions[ti].state.as_str());

            if start > col_f + 0.001 {
                // Leading edge: state starts partway into this cell.
                let void_fraction = start - col_f;
                let block_char = left_block_char(void_fraction);
                cell.set_char(block_char)
                    .set_style(Style::default().fg(color).add_modifier(Modifier::REVERSED));
            } else {
                // Single state in this cell → full block
                cell.set_char('█').set_fg(color);
            }
        }
    }
}

fn render_state_labels(
    buf: &mut Buffer,
    timeline: &Timeline,
    color_map: &mut StateColorMap,
    layout: &GraphLayout,
    y: u16,
) {
    // ── Render state-value labels at the start of each section ──
    for seg in timeline.segments() {
        let col_start = seg.start.ceil() as u16;
        // Add 1 column padding after a transition so the label doesn't
        // overwrite the partial-block transition character.
        let label_col = if seg.start.fract() > 0.001 {
            col_start.saturating_add(1)
        } else {
            col_start
        };

        let col_end = (seg.end.floor() as u16).min(layout.timeline_w());
        if label_col >= col_end {
            continue; // Not enough room for even one character
        }
        let available = (col_end - label_col) as usize;
        let display_len = seg.state.len().min(available);
        if display_len == 0 {
            continue;
        }
        let color = color_map.color_for(seg.state);

        // Compute a contrasting text color (black or white) based on the
        // background color's perceived luminance.
        let text_color = contrasting_text_color(color);

        for (ci, ch) in seg.state.chars().take(display_len).enumerate() {
            let cell = &mut buf[(layout.timeline_x() + label_col + ci as u16, y)];
            cell.set_char(ch).set_fg(text_color).set_bg(color);
        }
    }
}

fn render_x_axis(buf: &mut Buffer, x_axis: &Axis, layout: &GraphLayout) {
    let timeline_w = layout.timeline_w();
    if !layout.has_axis_labels || timeline_w == 0 {
        return;
    }

    let graph_area = layout.graph_area();
    let timeline_x = layout.timeline_x();
    let inner_right = layout.inner.x + layout.inner.width;
    let axis_line_y = graph_area.y + graph_area.height;
    let axis_label_y = axis_line_y + 1;

    // Draw the axis line with "─" characters across the timeline
    for col in 0..timeline_w {
        let x = timeline_x + col;
        if x < inner_right {
            buf[(x, axis_line_y)]
                .set_symbol(symbols::line::HORIZONTAL)
                .set_style(x_axis.style);
        }
    }

    // Place tick marks (┴) and labels
    let labels = &x_axis.labels;
    let n_labels = labels.len();
    if n_labels >= 2 {
        let label_row_limit = layout.inner.y + layout.inner.height + layout.axis_rows();
        for (i, label) in labels.iter().enumerate() {
            // Compute the tick position within the timeline
            let frac = i as f64 / (n_labels - 1) as f64;
            let tick_col = (frac * (timeline_w.saturating_sub(1)) as f64).round() as u16;

            let abs_x = timeline_x + tick_col;

            // Draw tick mark on the axis line
            if abs_x < inner_right {
                buf[(abs_x, axis_line_y)]
                    .set_symbol("┴")
                    .set_style(x_axis.style);
            }

            // Draw label on the label row
            if axis_label_y < label_row_limit {
                let label_str = label.content.as_ref();
                let label_len = label_str.len() as u16;

                // Position the label: first=left-aligned, last=right-aligned,
                // middle=centered on the tick.
                let label_x = if i == 0 {
                    abs_x
                } else if i == n_labels - 1 {
                    abs_x.saturating_sub(label_len.saturating_sub(1))
                } else {
                    abs_x.saturating_sub(label_len / 2)
                };

                // Clamp to timeline area
                let label_x = label_x.max(timeline_x);
                let max_len = inner_right.saturating_sub(label_x);
                let draw_len = label_len.min(max_len) as usize;

                for (ci, ch) in label_str.chars().take(draw_len).enumerate() {
                    let cx = label_x + ci as u16;
                    if cx < inner_right {
                        buf[(cx, axis_label_y)].set_char(ch).set_style(label.style);
                    }
                }
            }
        }
    }
}

impl<'a> StatefulWidget for StateGraphWidget<'a> {
    type State = StateGraphWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.block.as_ref().render(area, buf);
        let inner = self.block.inner_if_some(area);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let layout = compute_layout(&self, inner, state);
        render_y_separator(buf, &layout);

        let graph_area = layout.graph_area();
        let timeline_w = layout.timeline_w();
        let x_range = layout.x_range();

        let datasets_to_render = state
            .datasets
            .iter_mut()
            .skip(state.scroll_offset)
            .take(layout.visible_count);

        for (row_idx, dataset) in datasets_to_render.enumerate() {
            let y = graph_area.y + (row_idx as u16) * layout.row_stride;
            if y >= graph_area.y + graph_area.height {
                break;
            }

            render_dataset_label(buf, dataset, &layout, y);

            if timeline_w == 0 || dataset.data.is_empty() || x_range <= 0.0 {
                continue;
            }

            let tw = timeline_w as f64;
            let timeline = build_timeline(&dataset.data, layout.x_bounds, x_range, tw);
            render_timeline(buf, &timeline, &mut dataset.color_map, &layout, y);
            render_state_labels(buf, &timeline, &mut dataset.color_map, &layout, y);
        }

        if let Some(ref x_axis) = self.x_axis {
            render_x_axis(buf, x_axis, &layout);
        }
    }
}

/// Return black or white depending on which contrasts better with the given color.
fn contrasting_text_color(color: Color) -> Color {
    let (r, g, b) = match color {
        Color::Black => (0, 0, 0),
        Color::Red => (205, 0, 0),
        Color::Green => (0, 205, 0),
        Color::Yellow => (205, 205, 0),
        Color::Blue => (0, 0, 238),
        Color::Magenta => (205, 0, 205),
        Color::Cyan => (0, 205, 205),
        Color::White => (229, 229, 229),
        Color::DarkGray => (127, 127, 127),
        Color::LightRed => (255, 95, 95),
        Color::LightGreen => (95, 255, 95),
        Color::LightYellow => (255, 255, 95),
        Color::LightBlue => (95, 95, 255),
        Color::LightMagenta => (255, 95, 255),
        Color::LightCyan => (95, 255, 255),
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Reset => (229, 229, 229),
        _ => (200, 200, 200),
    };
    // Perceived luminance (ITU-R BT.601)
    let luminance = 0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64;
    if luminance > 128.0 {
        Color::Black
    } else {
        Color::White
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::prelude::Buffer;

    // ── Rendering tests with full buffer + style assertions ──────────

    #[test]
    fn test_render_single_state() {
        // One dataset, one state "on" covering the entire timeline.
        // The label "on" is overlaid at cols 6-7 (fg=White, bg=Green).
        let datasets = vec![StateDataset::new("led")
            .data(vec![StateDataPoint {
                x: 0.0,
                state: "on".into(),
            }])
            .color_map(StateColorMap::new().map("on", Color::Green))];
        let mut state = StateGraphWidgetState::new(datasets);
        let area = Rect::new(0, 0, 20, 2);
        let mut buf = Buffer::empty(area);

        StateGraphWidget::new()
            .x_bounds([0.0, 10.0])
            .label_width(5)
            .render(area, &mut buf, &mut state);

        let mut expected = Buffer::with_lines(["  led│on████████████", "     │              "]);
        expected.set_style(Rect::new(0, 0, 5, 1), Style::default().fg(Color::White));
        expected.set_style(Rect::new(5, 0, 1, 2), Style::default().fg(Color::DarkGray));
        // Label "on" at cols 6-7
        expected.set_style(
            Rect::new(6, 0, 2, 1),
            Style::default().fg(Color::White).bg(Color::Green),
        );
        // Rest of the blocks
        expected.set_style(Rect::new(8, 0, 12, 1), Style::default().fg(Color::Green));

        assert_eq!(buf, expected);
    }

    #[test]
    fn test_render_state_transition() {
        // Transition at x=5 in [0,10]. timeline_w=14.
        // Segment "X": cols 0-6 (label "X" at col 6), segment "Y": cols 7-13 (label "Y" at col 13).
        let datasets = vec![StateDataset::new("ab")
            .data(vec![
                StateDataPoint {
                    x: 0.0,
                    state: "X".into(),
                },
                StateDataPoint {
                    x: 5.0,
                    state: "Y".into(),
                },
            ])
            .color_map(
                StateColorMap::new()
                    .map("X", Color::Red)
                    .map("Y", Color::Green),
            )];
        let mut state = StateGraphWidgetState::new(datasets);
        let area = Rect::new(0, 0, 20, 2);
        let mut buf = Buffer::empty(area);

        StateGraphWidget::new()
            .x_bounds([0.0, 10.0])
            .label_width(5)
            .render(area, &mut buf, &mut state);

        let mut expected = Buffer::with_lines(["   ab│X██████Y██████", "     │              "]);
        expected.set_style(Rect::new(0, 0, 5, 1), Style::default().fg(Color::White));
        expected.set_style(Rect::new(5, 0, 1, 2), Style::default().fg(Color::DarkGray));
        // Label "X" at col 6
        expected.set_style(
            Rect::new(6, 0, 1, 1),
            Style::default().fg(Color::White).bg(Color::Red),
        );
        // "X" blocks at cols 7-12
        expected.set_style(Rect::new(7, 0, 6, 1), Style::default().fg(Color::Red));
        // Label "Y" at col 13
        expected.set_style(
            Rect::new(13, 0, 1, 1),
            Style::default().fg(Color::White).bg(Color::Green),
        );
        // "Y" blocks at cols 14-19
        expected.set_style(Rect::new(14, 0, 6, 1), Style::default().fg(Color::Green));

        assert_eq!(buf, expected);
    }

    #[test]
    fn test_render_independent_colormaps() {
        // Two datasets, same state name "on" with different colors.
        let datasets = vec![
            StateDataset::new("abc")
                .data(vec![StateDataPoint {
                    x: 0.0,
                    state: "on".into(),
                }])
                .color_map(StateColorMap::new().map("on", Color::Red)),
            StateDataset::new("xyz")
                .data(vec![StateDataPoint {
                    x: 0.0,
                    state: "on".into(),
                }])
                .color_map(StateColorMap::new().map("on", Color::Blue)),
        ];
        let mut state = StateGraphWidgetState::new(datasets);
        let area = Rect::new(0, 0, 20, 3);
        let mut buf = Buffer::empty(area);

        StateGraphWidget::new()
            .x_bounds([0.0, 10.0])
            .label_width(5)
            .spacing(Spacing::Dense)
            .render(area, &mut buf, &mut state);

        let mut expected = Buffer::with_lines([
            "  abc│on████████████",
            "  xyz│on████████████",
            "     │              ",
        ]);
        // Row 0: label "on" at cols 6-7 (fg=White, bg=Red), rest Red
        expected.set_style(Rect::new(0, 0, 5, 1), Style::default().fg(Color::White));
        expected.set_style(Rect::new(5, 0, 1, 3), Style::default().fg(Color::DarkGray));
        expected.set_style(
            Rect::new(6, 0, 2, 1),
            Style::default().fg(Color::White).bg(Color::Red),
        );
        expected.set_style(Rect::new(8, 0, 12, 1), Style::default().fg(Color::Red));
        // Row 1: label "on" at cols 6-7 (fg=White, bg=Blue), rest Blue
        expected.set_style(Rect::new(0, 1, 5, 1), Style::default().fg(Color::White));
        expected.set_style(
            Rect::new(6, 1, 2, 1),
            Style::default().fg(Color::White).bg(Color::Blue),
        );
        expected.set_style(Rect::new(8, 1, 12, 1), Style::default().fg(Color::Blue));

        assert_eq!(buf, expected);
    }

    #[test]
    fn test_render_bordered() {
        // Outer 22×4 → inner 20×2. Label "on" at cols 7-8 inside.
        let datasets = vec![StateDataset::new("led")
            .data(vec![StateDataPoint {
                x: 0.0,
                state: "on".into(),
            }])
            .color_map(StateColorMap::new().map("on", Color::Green))];
        let mut state = StateGraphWidgetState::new(datasets);
        let area = Rect::new(0, 0, 22, 4);
        let mut buf = Buffer::empty(area);

        StateGraphWidget::new()
            .block(Block::bordered().title("Graph"))
            .x_bounds([0.0, 10.0])
            .label_width(5)
            .render(area, &mut buf, &mut state);

        let mut expected = Buffer::with_lines([
            "┌Graph───────────────┐",
            "│  led│on████████████│",
            "│     │              │",
            "└────────────────────┘",
        ]);
        expected.set_style(Rect::new(1, 1, 5, 1), Style::default().fg(Color::White));
        expected.set_style(Rect::new(6, 1, 1, 2), Style::default().fg(Color::DarkGray));
        // Label "on" at cols 7-8
        expected.set_style(
            Rect::new(7, 1, 2, 1),
            Style::default().fg(Color::White).bg(Color::Green),
        );
        expected.set_style(Rect::new(9, 1, 12, 1), Style::default().fg(Color::Green));

        assert_eq!(buf, expected);
    }

    #[test]
    fn test_render_empty_datasets() {
        let mut state = StateGraphWidgetState::new(vec![]);
        let area = Rect::new(0, 0, 20, 2);
        let mut buf = Buffer::empty(area);

        StateGraphWidget::new()
            .x_bounds([0.0, 10.0])
            .label_width(5)
            .render(area, &mut buf, &mut state);

        let mut expected = Buffer::with_lines(["     │              ", "     │              "]);
        expected.set_style(Rect::new(5, 0, 1, 2), Style::default().fg(Color::DarkGray));
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_render_scrolled() {
        // 3 datasets, scroll_offset=1, area fits 2 rows.
        let datasets = vec![
            StateDataset::new("aaa")
                .data(vec![StateDataPoint {
                    x: 0.0,
                    state: "A".into(),
                }])
                .color_map(StateColorMap::new().map("A", Color::Red)),
            StateDataset::new("bbb")
                .data(vec![StateDataPoint {
                    x: 0.0,
                    state: "B".into(),
                }])
                .color_map(StateColorMap::new().map("B", Color::Green)),
            StateDataset::new("ccc")
                .data(vec![StateDataPoint {
                    x: 0.0,
                    state: "C".into(),
                }])
                .color_map(StateColorMap::new().map("C", Color::Blue)),
        ];
        let mut state = StateGraphWidgetState::new(datasets);
        state.scroll_offset = 1;
        let area = Rect::new(0, 0, 20, 2);
        let mut buf = Buffer::empty(area);

        StateGraphWidget::new()
            .x_bounds([0.0, 10.0])
            .label_width(5)
            .render(area, &mut buf, &mut state);

        let mut expected = Buffer::with_lines(["  bbb│B█████████████", "  ccc│C█████████████"]);
        // Row 0: label "B" at col 6 (fg=White, bg=Green), rest Green
        expected.set_style(Rect::new(0, 0, 5, 1), Style::default().fg(Color::White));
        expected.set_style(Rect::new(5, 0, 1, 1), Style::default().fg(Color::DarkGray));
        expected.set_style(
            Rect::new(6, 0, 1, 1),
            Style::default().fg(Color::White).bg(Color::Green),
        );
        expected.set_style(Rect::new(7, 0, 13, 1), Style::default().fg(Color::Green));
        // Row 1: label "C" at col 6 (fg=White, bg=Blue), rest Blue
        expected.set_style(Rect::new(0, 1, 5, 1), Style::default().fg(Color::White));
        expected.set_style(Rect::new(5, 1, 1, 1), Style::default().fg(Color::DarkGray));
        expected.set_style(
            Rect::new(6, 1, 1, 1),
            Style::default().fg(Color::White).bg(Color::Blue),
        );
        expected.set_style(Rect::new(7, 1, 13, 1), Style::default().fg(Color::Blue));

        assert_eq!(buf, expected);
    }

    #[test]
    fn test_render_label_truncation() {
        // Label "longname" truncated to "longn". State label "A" at col 6.
        let datasets = vec![StateDataset::new("longname")
            .data(vec![StateDataPoint {
                x: 0.0,
                state: "A".into(),
            }])
            .color_map(StateColorMap::new().map("A", Color::Red))];
        let mut state = StateGraphWidgetState::new(datasets);
        let area = Rect::new(0, 0, 20, 2);
        let mut buf = Buffer::empty(area);

        StateGraphWidget::new()
            .x_bounds([0.0, 10.0])
            .label_width(5)
            .render(area, &mut buf, &mut state);

        let mut expected = Buffer::with_lines(["longn│A█████████████", "     │              "]);
        expected.set_style(Rect::new(0, 0, 5, 1), Style::default().fg(Color::White));
        expected.set_style(Rect::new(5, 0, 1, 2), Style::default().fg(Color::DarkGray));
        // Label "A" at col 6
        expected.set_style(
            Rect::new(6, 0, 1, 1),
            Style::default().fg(Color::White).bg(Color::Red),
        );
        expected.set_style(Rect::new(7, 0, 13, 1), Style::default().fg(Color::Red));

        assert_eq!(buf, expected);
    }

    #[test]
    fn test_render_smooth_transition() {
        // Transition at x=3.5 in [0,10] with timeline_w=10.
        // col_boundary = 3.5/10 * 10 = 3.5 → transition cell at col 3.
        // Cols 0-2: full Red, col 3: partial block (Red/Green), cols 4-9: full Green.
        // State labels: "X" at col 0, "Y" at col 4 (after transition cell).
        let datasets = vec![StateDataset::new("s")
            .data(vec![
                StateDataPoint {
                    x: 0.0,
                    state: "X".into(),
                },
                StateDataPoint {
                    x: 3.5,
                    state: "Y".into(),
                },
            ])
            .color_map(
                StateColorMap::new()
                    .map("X", Color::Red)
                    .map("Y", Color::Green),
            )];
        let mut state = StateGraphWidgetState::new(datasets);
        let area = Rect::new(0, 0, 14, 1);
        let mut buf = Buffer::empty(area);

        StateGraphWidget::new()
            .x_bounds([0.0, 10.0])
            .label_width(3)
            .render(area, &mut buf, &mut state);

        // col 0: label "X" (White on Red)
        // cols 1-2: full Red
        // col 3: partial block transition (fg=Red, bg=Green)
        // col 4: label "Y" (White on Green)
        // cols 5-9: full Green
        let cell_3 = &buf[(7, 0)]; // timeline_x=4, col=3 → x=7
        assert_eq!(
            cell_3.fg,
            Color::Red,
            "Transition cell fg should be old state"
        );
        assert_eq!(
            cell_3.bg,
            Color::Green,
            "Transition cell bg should be new state"
        );
        assert!(
            LEFT_BLOCKS.contains(&cell_3.symbol().chars().next().unwrap()),
            "Transition cell should use a partial block character"
        );
    }

    #[test]
    fn test_render_shade_for_many_states() {
        // Many rapid transitions in a narrow area → cells with >2 states get shade.
        // timeline_w = 4, x_bounds = [0,4]. 5 transitions → at least some cols have >2 states.
        let datasets = vec![StateDataset::new("x")
            .data(vec![
                StateDataPoint {
                    x: 0.0,
                    state: "A".into(),
                },
                StateDataPoint {
                    x: 0.3,
                    state: "B".into(),
                },
                StateDataPoint {
                    x: 0.6,
                    state: "C".into(),
                },
            ])
            .color_map(
                StateColorMap::new()
                    .map("A", Color::Red)
                    .map("B", Color::Green)
                    .map("C", Color::Blue),
            )];
        let mut state = StateGraphWidgetState::new(datasets);
        let area = Rect::new(0, 0, 8, 1);
        let mut buf = Buffer::empty(area);

        StateGraphWidget::new()
            .x_bounds([0.0, 4.0])
            .label_width(3)
            .render(area, &mut buf, &mut state);

        // Col 0 (timeline_x=4): x range [0, 1) has states A, B, C → 3 distinct → shade
        let cell = &buf[(4, 0)];
        assert_eq!(
            cell.symbol().chars().next().unwrap(),
            '░',
            "Cell with >2 states should use shade character"
        );
        assert_eq!(cell.fg, Color::DarkGray);
    }

    #[test]
    fn test_state_label_cropped_to_section() {
        // State "longstate" in a section that only spans 4 columns → cropped to "long".
        let datasets = vec![StateDataset::new("s")
            .data(vec![
                StateDataPoint {
                    x: 0.0,
                    state: "longstate".into(),
                },
                StateDataPoint {
                    x: 5.0,
                    state: "Y".into(),
                },
            ])
            .color_map(
                StateColorMap::new()
                    .map("longstate", Color::Red)
                    .map("Y", Color::Green),
            )];
        let mut state = StateGraphWidgetState::new(datasets);
        // timeline_w = 10, transition at col 5.0.
        let area = Rect::new(0, 0, 14, 1);
        let mut buf = Buffer::empty(area);

        StateGraphWidget::new()
            .x_bounds([0.0, 10.0])
            .label_width(3)
            .render(area, &mut buf, &mut state);

        // timeline_x = 4. Segment "longstate" spans cols 0-4.
        // Label should be "longs" (5 chars fit in 5 columns).
        let mut label = String::new();
        for c in 0..5u16 {
            let cell = &buf[(4 + c, 0)];
            if cell.bg == Color::Red {
                label.push(cell.symbol().chars().next().unwrap());
            }
        }
        assert_eq!(label, "longs", "Label should be cropped to available space");
    }

    // ── Unit tests ───────────────────────────────────────────────────

    #[test]
    fn test_color_map_auto_assign() {
        let mut cm = StateColorMap::new();
        let c1 = cm.color_for("state_a");
        let c2 = cm.color_for("state_b");
        let c1_again = cm.color_for("state_a");

        assert_eq!(c1, c1_again, "Same state should return same color");
        assert_ne!(c1, c2, "Different states should get different colors");
    }

    #[test]
    fn test_color_map_explicit() {
        let mut cm = StateColorMap::new()
            .map("on", Color::Green)
            .map("off", Color::Red);
        assert_eq!(cm.color_for("on"), Color::Green);
        assert_eq!(cm.color_for("off"), Color::Red);
    }

    #[test]
    fn test_needs_redraw() {
        let mut state = StateGraphWidgetState::default();
        assert!(state.needs_redraw(), "First call should return true");
        assert!(!state.needs_redraw(), "Second call should return false");
        state.update(vec![]);
        assert!(state.needs_redraw(), "After update should return true");
    }

    #[test]
    fn test_contrasting_text_color() {
        // Dark colors should get white text
        assert_eq!(contrasting_text_color(Color::Red), Color::White);
        assert_eq!(contrasting_text_color(Color::Blue), Color::White);
        assert_eq!(contrasting_text_color(Color::Black), Color::White);
        // Light colors should get black text
        assert_eq!(contrasting_text_color(Color::LightGreen), Color::Black);
        assert_eq!(contrasting_text_color(Color::LightYellow), Color::Black);
        assert_eq!(contrasting_text_color(Color::White), Color::Black);
    }

    #[test]
    fn test_left_block_char() {
        assert_eq!(left_block_char(0.0), ' '); // Zero → space (no visible block)
        assert_eq!(left_block_char(1.0), '█'); // Full → full block
        assert_eq!(left_block_char(0.5), '▌'); // Half
        assert_eq!(left_block_char(0.125), '▏'); // 1/8 → thinnest block
        assert_eq!(left_block_char(0.05), ' '); // < 1/16 → space
    }

    #[test]
    fn test_render_smooth_leading_edge() {
        // First state starts at x=3.5 in [0,10] with timeline_w=10.
        // cols 0-2: void cells (empty, terminal background shows through).
        // col 3: leading-edge partial block (fg=Green + REVERSED).
        // cols 4-9: full Green blocks.
        let datasets = vec![StateDataset::new("s")
            .data(vec![StateDataPoint {
                x: 3.5,
                state: "X".into(),
            }])
            .color_map(StateColorMap::new().map("X", Color::Green))];
        let mut state = StateGraphWidgetState::new(datasets);
        let area = Rect::new(0, 0, 14, 1);
        let mut buf = Buffer::empty(area);

        StateGraphWidget::new()
            .x_bounds([0.0, 10.0])
            .label_width(3)
            .render(area, &mut buf, &mut state);

        // timeline starts at x=4. col 3 maps to x [3,4) which is the leading edge.
        let cell_3 = &buf[(4 + 3, 0)];
        assert!(
            LEFT_BLOCKS.contains(&cell_3.symbol().chars().next().unwrap()),
            "Leading-edge cell should use a partial block character, got '{}'",
            cell_3.symbol()
        );
        assert_eq!(
            cell_3.fg,
            Color::Green,
            "Leading-edge cell fg should be state color"
        );
        assert!(
            cell_3.modifier.contains(Modifier::REVERSED),
            "Leading-edge cell should have REVERSED modifier"
        );

        // col 4 should be a full block
        let cell_4 = &buf[(4 + 4, 0)];
        assert_eq!(cell_4.symbol(), "█");
        assert_eq!(cell_4.fg, Color::Green);

        // cols 0-2 should be void (empty, terminal bg shows through)
        let cell_0 = &buf[(4, 0)];
        assert_eq!(cell_0.symbol(), " ", "Void cells should be empty spaces");
    }

    // ── Spacing tests ────────────────────────────────────────────────

    #[test]
    fn test_spacing_dense() {
        // 2 datasets in 3 rows with Dense → rows 0 and 1, row 2 empty.
        let datasets = vec![
            StateDataset::new("a")
                .data(vec![StateDataPoint {
                    x: 0.0,
                    state: "X".into(),
                }])
                .color_map(StateColorMap::new().map("X", Color::Red)),
            StateDataset::new("b")
                .data(vec![StateDataPoint {
                    x: 0.0,
                    state: "Y".into(),
                }])
                .color_map(StateColorMap::new().map("Y", Color::Green)),
        ];
        let mut state = StateGraphWidgetState::new(datasets);
        let area = Rect::new(0, 0, 10, 3);
        let mut buf = Buffer::empty(area);

        StateGraphWidget::new()
            .x_bounds([0.0, 10.0])
            .label_width(3)
            .spacing(Spacing::Dense)
            .render(area, &mut buf, &mut state);

        // timeline_x=4, state label at col 4, check col 5 for pure block color
        let cell_r0 = &buf[(5, 0)];
        assert_eq!(cell_r0.fg, Color::Red, "Row 0 should be dataset a (Red)");
        let cell_r1 = &buf[(5, 1)];
        assert_eq!(
            cell_r1.fg,
            Color::Green,
            "Row 1 should be dataset b (Green)"
        );
        let cell_r2 = &buf[(5, 2)];
        assert_eq!(cell_r2.symbol(), " ", "Row 2 should be empty");
    }

    #[test]
    fn test_spacing_spaced() {
        // 2 datasets in 4 rows with Spaced → rows 0 and 2, rows 1 and 3 empty.
        let datasets = vec![
            StateDataset::new("a")
                .data(vec![StateDataPoint {
                    x: 0.0,
                    state: "X".into(),
                }])
                .color_map(StateColorMap::new().map("X", Color::Red)),
            StateDataset::new("b")
                .data(vec![StateDataPoint {
                    x: 0.0,
                    state: "Y".into(),
                }])
                .color_map(StateColorMap::new().map("Y", Color::Green)),
        ];
        let mut state = StateGraphWidgetState::new(datasets);
        let area = Rect::new(0, 0, 10, 4);
        let mut buf = Buffer::empty(area);

        StateGraphWidget::new()
            .x_bounds([0.0, 10.0])
            .label_width(3)
            .spacing(Spacing::Spaced)
            .render(area, &mut buf, &mut state);

        // Row 0: dataset "a"
        let cell_r0 = &buf[(5, 0)];
        assert_eq!(cell_r0.fg, Color::Red, "Row 0 should be dataset a (Red)");
        // Row 1: blank
        let cell_r1 = &buf[(5, 1)];
        assert_eq!(cell_r1.symbol(), " ", "Row 1 should be blank spacer");
        // Row 2: dataset "b"
        let cell_r2 = &buf[(5, 2)];
        assert_eq!(
            cell_r2.fg,
            Color::Green,
            "Row 2 should be dataset b (Green)"
        );
        // Row 3: empty
        let cell_r3 = &buf[(5, 3)];
        assert_eq!(cell_r3.symbol(), " ", "Row 3 should be empty");
    }

    #[test]
    fn test_spacing_auto_uses_spaced_when_room() {
        // 2 datasets in 3 rows → 2*2-1=3 fits → Spaced: rows 0 and 2.
        let datasets = vec![
            StateDataset::new("a")
                .data(vec![StateDataPoint {
                    x: 0.0,
                    state: "X".into(),
                }])
                .color_map(StateColorMap::new().map("X", Color::Red)),
            StateDataset::new("b")
                .data(vec![StateDataPoint {
                    x: 0.0,
                    state: "Y".into(),
                }])
                .color_map(StateColorMap::new().map("Y", Color::Green)),
        ];
        let mut state = StateGraphWidgetState::new(datasets);
        let area = Rect::new(0, 0, 10, 3);
        let mut buf = Buffer::empty(area);

        StateGraphWidget::new()
            .x_bounds([0.0, 10.0])
            .label_width(3)
            .spacing(Spacing::Auto)
            .render(area, &mut buf, &mut state);

        // Row 0: dataset "a"
        let cell_r0 = &buf[(5, 0)];
        assert_eq!(cell_r0.fg, Color::Red, "Row 0 should be dataset a");
        // Row 1: blank spacer
        let cell_r1 = &buf[(5, 1)];
        assert_eq!(cell_r1.symbol(), " ", "Row 1 should be blank spacer");
        // Row 2: dataset "b"
        let cell_r2 = &buf[(5, 2)];
        assert_eq!(cell_r2.fg, Color::Green, "Row 2 should be dataset b");
    }

    #[test]
    fn test_spacing_auto_dense_stable_when_scrolled() {
        // 3 datasets in 3 rows → Dense (2*3-1=5 > 3). After scrolling to offset=2
        // only 1 dataset remains visible, but spacing must still be Dense (not
        // switch to Spaced just because fewer rows are needed).
        let datasets = vec![
            StateDataset::new("a")
                .data(vec![StateDataPoint {
                    x: 0.0,
                    state: "X".into(),
                }])
                .color_map(StateColorMap::new().map("X", Color::Red)),
            StateDataset::new("b")
                .data(vec![StateDataPoint {
                    x: 0.0,
                    state: "Y".into(),
                }])
                .color_map(StateColorMap::new().map("Y", Color::Green)),
            StateDataset::new("c")
                .data(vec![StateDataPoint {
                    x: 0.0,
                    state: "Z".into(),
                }])
                .color_map(StateColorMap::new().map("Z", Color::Blue)),
        ];
        let mut state = StateGraphWidgetState::new(datasets);
        state.scroll_offset = 2; // only dataset "c" remains
        let area = Rect::new(0, 0, 10, 3);
        let mut buf = Buffer::empty(area);

        StateGraphWidget::new()
            .x_bounds([0.0, 10.0])
            .label_width(3)
            .spacing(Spacing::Auto)
            .render(area, &mut buf, &mut state);

        // With Dense stride=1, dataset "c" renders at row 0.
        // With Spaced stride=2, dataset "c" would also render at row 0 — but
        // row 1 would be blank and row 2 empty. We verify that rows 1 and 2 are
        // both blank (no spurious second render that would only appear if the
        // layout incorrectly set visible_count > 1 in spaced mode).
        let cell_r0 = &buf[(5, 0)];
        assert_eq!(cell_r0.fg, Color::Blue, "Row 0 should show dataset c");
        // Row 1 must be empty regardless of Dense vs Spaced since there is no
        // 4th dataset to render there.
        let cell_r1 = &buf[(5, 1)];
        assert_eq!(cell_r1.symbol(), " ", "Row 1 should be empty");
    }

    #[test]
    fn test_spacing_auto_consistent_across_scroll_offsets() {
        // 4 datasets in 4 rows (2*4-1=7 > 4 → Dense).
        // At scroll_offset=0 and scroll_offset=2 the row_stride must both be 1
        // (Dense), verified by checking that adjacent rows both have data.
        fn make_datasets() -> Vec<StateDataset> {
            [
                ("a", Color::Red),
                ("b", Color::Green),
                ("c", Color::Blue),
                ("d", Color::Yellow),
            ]
            .iter()
            .map(|(name, color)| {
                StateDataset::new(*name)
                    .data(vec![StateDataPoint {
                        x: 0.0,
                        state: name.to_string(),
                    }])
                    .color_map(StateColorMap::new().map(*name, *color))
            })
            .collect()
        }

        // offset=0
        let mut state0 = StateGraphWidgetState::new(make_datasets());
        let area = Rect::new(0, 0, 10, 4);
        let mut buf0 = Buffer::empty(area);
        StateGraphWidget::new()
            .x_bounds([0.0, 10.0])
            .label_width(3)
            .spacing(Spacing::Auto)
            .render(area, &mut buf0, &mut state0);

        // offset=2
        let mut state2 = StateGraphWidgetState::new(make_datasets());
        state2.scroll_offset = 2;
        let mut buf2 = Buffer::empty(area);
        StateGraphWidget::new()
            .x_bounds([0.0, 10.0])
            .label_width(3)
            .spacing(Spacing::Auto)
            .render(area, &mut buf2, &mut state2);

        // offset=0: rows 0 and 1 should both have data (Dense stride=1).
        let r0 = &buf0[(5, 0)];
        let r1 = &buf0[(5, 1)];
        assert_ne!(r0.symbol(), " ", "offset=0 row 0 should have data");
        assert_ne!(
            r1.symbol(),
            " ",
            "offset=0 row 1 should have data (Dense, not blank spacer)"
        );

        // offset=2: rows 0 and 1 should both have data (Dense stride=1, datasets c and d).
        let r0s = &buf2[(5, 0)];
        let r1s = &buf2[(5, 1)];
        assert_ne!(r0s.symbol(), " ", "offset=2 row 0 should have data");
        assert_ne!(
            r1s.symbol(),
            " ",
            "offset=2 row 1 should have data (Dense, not blank spacer)"
        );
    }

    #[test]
    fn test_spacing_auto_falls_back_to_dense() {
        // 3 datasets in 3 rows → 2*3-1=5 > 3 → Dense: rows 0, 1, 2.
        let datasets = vec![
            StateDataset::new("a")
                .data(vec![StateDataPoint {
                    x: 0.0,
                    state: "X".into(),
                }])
                .color_map(StateColorMap::new().map("X", Color::Red)),
            StateDataset::new("b")
                .data(vec![StateDataPoint {
                    x: 0.0,
                    state: "Y".into(),
                }])
                .color_map(StateColorMap::new().map("Y", Color::Green)),
            StateDataset::new("c")
                .data(vec![StateDataPoint {
                    x: 0.0,
                    state: "Z".into(),
                }])
                .color_map(StateColorMap::new().map("Z", Color::Blue)),
        ];
        let mut state = StateGraphWidgetState::new(datasets);
        let area = Rect::new(0, 0, 10, 3);
        let mut buf = Buffer::empty(area);

        StateGraphWidget::new()
            .x_bounds([0.0, 10.0])
            .label_width(3)
            .spacing(Spacing::Auto)
            .render(area, &mut buf, &mut state);

        // All three datasets packed in consecutive rows
        let cell_r0 = &buf[(5, 0)];
        assert_eq!(cell_r0.fg, Color::Red, "Row 0 should be dataset a");
        let cell_r1 = &buf[(5, 1)];
        assert_eq!(cell_r1.fg, Color::Green, "Row 1 should be dataset b");
        let cell_r2 = &buf[(5, 2)];
        assert_eq!(cell_r2.fg, Color::Blue, "Row 2 should be dataset c");
    }

    // ── X-axis tests ─────────────────────────────────────────────────

    #[test]
    fn test_x_axis_reserves_two_rows() {
        // Area 20×4. With X axis (2 rows), graph_area.height = 2.
        // One dataset should render at row 0, axis line at row 2, labels at row 3.
        let datasets = vec![StateDataset::new("led")
            .data(vec![StateDataPoint {
                x: 0.0,
                state: "on".into(),
            }])
            .color_map(StateColorMap::new().map("on", Color::Green))];
        let mut state = StateGraphWidgetState::new(datasets);
        let area = Rect::new(0, 0, 20, 4);
        let mut buf = Buffer::empty(area);

        StateGraphWidget::new()
            .x_bounds([0.0, 10.0])
            .label_width(5)
            .x_axis(
                Axis::new()
                    .labels(vec![Span::raw("0"), Span::raw("10")])
                    .style(Style::default().fg(Color::DarkGray)),
            )
            .render(area, &mut buf, &mut state);

        // Row 0: dataset (label + timeline)
        let cell_data = &buf[(8, 0)]; // past the "on" label overlay
        assert_eq!(cell_data.fg, Color::Green, "Row 0 should have the dataset");

        // Row 1: empty (graph has 2 rows of height, only 1 dataset)
        let cell_empty = &buf[(8, 1)];
        assert_eq!(
            cell_empty.symbol(),
            " ",
            "Row 1 should be empty graph space"
        );

        // Row 2: axis line (─ characters in the timeline area)
        let cell_axis = &buf[(8, 2)];
        assert_eq!(cell_axis.symbol(), "─", "Row 2 should have axis line");
        assert_eq!(cell_axis.fg, Color::DarkGray);

        // Row 3: labels — "0" at left, "10" at right
        let cell_label_left = &buf[(6, 3)]; // timeline_x = 6
        assert_eq!(cell_label_left.symbol(), "0", "Left label should be '0'");

        // Right label "10" right-aligned at end of timeline
        let cell_label_right_1 = &buf[(18, 3)]; // timeline_x + timeline_w - 2
        let cell_label_right_2 = &buf[(19, 3)]; // timeline_x + timeline_w - 1
        let right_label = format!(
            "{}{}",
            cell_label_right_1.symbol(),
            cell_label_right_2.symbol()
        );
        assert_eq!(right_label, "10", "Right label should be '10'");
    }

    #[test]
    fn test_x_axis_tick_marks() {
        // 3 labels → 3 tick marks (┴) at positions 0, mid, end.
        let datasets = vec![StateDataset::new("s")
            .data(vec![StateDataPoint {
                x: 0.0,
                state: "A".into(),
            }])
            .color_map(StateColorMap::new().map("A", Color::Red))];
        let mut state = StateGraphWidgetState::new(datasets);
        let area = Rect::new(0, 0, 16, 3);
        let mut buf = Buffer::empty(area);

        StateGraphWidget::new()
            .x_bounds([0.0, 10.0])
            .label_width(3)
            .x_axis(
                Axis::new()
                    .labels(vec![Span::raw("0"), Span::raw("5"), Span::raw("10")])
                    .style(Style::default().fg(Color::DarkGray)),
            )
            .render(area, &mut buf, &mut state);

        // timeline_x = 4, timeline_w = 12, axis_line at row 1
        // Ticks at: col 0 (abs 4), col 5 or 6 (mid), col 11 (abs 15)
        let tick_left = &buf[(4, 1)];
        assert_eq!(tick_left.symbol(), "┴", "Left tick mark");

        let tick_right = &buf[(15, 1)];
        assert_eq!(tick_right.symbol(), "┴", "Right tick mark");

        // Middle tick at col (11/2) = 5 or 6, rounded
        let mid_col = (0.5_f64 * 11.0).round() as u16; // = 6
        let tick_mid = &buf[(4 + mid_col, 1)];
        assert_eq!(tick_mid.symbol(), "┴", "Middle tick mark");

        // Non-tick columns should have ─
        let non_tick = &buf[(5, 1)]; // col 1
        assert_eq!(non_tick.symbol(), "─", "Non-tick axis cell should be ─");
    }

    #[test]
    fn test_x_axis_no_labels_no_axis() {
        // Without x_axis set, no rows are reserved. Same as before.
        let datasets = vec![StateDataset::new("led")
            .data(vec![StateDataPoint {
                x: 0.0,
                state: "on".into(),
            }])
            .color_map(StateColorMap::new().map("on", Color::Green))];
        let mut state = StateGraphWidgetState::new(datasets);
        let area = Rect::new(0, 0, 20, 2);
        let mut buf = Buffer::empty(area);

        StateGraphWidget::new()
            .x_bounds([0.0, 10.0])
            .label_width(5)
            .render(area, &mut buf, &mut state);

        // Row 0: dataset timeline
        let cell_data = &buf[(8, 0)];
        assert_eq!(cell_data.fg, Color::Green, "Row 0 should have the dataset");

        // Row 1: empty (no axis)
        let cell_r1 = &buf[(8, 1)];
        assert_eq!(cell_r1.symbol(), " ", "Row 1 should be empty (no axis)");
    }

    #[test]
    fn test_x_axis_with_border() {
        // Bordered 22×6 → inner 20×4. X axis takes 2 rows → graph_area.height = 2.
        let datasets = vec![StateDataset::new("led")
            .data(vec![StateDataPoint {
                x: 0.0,
                state: "on".into(),
            }])
            .color_map(StateColorMap::new().map("on", Color::Green))];
        let mut state = StateGraphWidgetState::new(datasets);
        let area = Rect::new(0, 0, 22, 6);
        let mut buf = Buffer::empty(area);

        StateGraphWidget::new()
            .block(Block::bordered())
            .x_bounds([0.0, 10.0])
            .label_width(5)
            .x_axis(
                Axis::new()
                    .labels(vec![Span::raw("0"), Span::raw("10")])
                    .style(Style::default().fg(Color::DarkGray)),
            )
            .render(area, &mut buf, &mut state);

        // Inner area starts at (1,1) with size 20×4.
        // graph_area: y=1, height=2. axis_line at y=3, labels at y=4.
        // Border bottom at y=5.

        // Row 1 (inner row 0): dataset
        let cell_data = &buf[(9, 1)]; // past label overlay
        assert_eq!(cell_data.fg, Color::Green, "Should have dataset in row 1");

        // Row 3 (inner row 2): axis line
        let cell_axis = &buf[(9, 3)];
        assert_eq!(cell_axis.symbol(), "─", "Should have axis line at row 3");

        // Row 4 (inner row 3): labels
        let cell_label = &buf[(7, 4)]; // timeline_x = 7 inside border
        assert_eq!(cell_label.symbol(), "0", "Should have left label at row 4");
    }

    #[test]
    fn test_x_axis_single_label_no_render() {
        // With only 1 label (<2), no axis should be rendered.
        let datasets = vec![StateDataset::new("s")
            .data(vec![StateDataPoint {
                x: 0.0,
                state: "A".into(),
            }])
            .color_map(StateColorMap::new().map("A", Color::Red))];
        let mut state = StateGraphWidgetState::new(datasets);
        let area = Rect::new(0, 0, 20, 3);
        let mut buf = Buffer::empty(area);

        StateGraphWidget::new()
            .x_bounds([0.0, 10.0])
            .label_width(5)
            .x_axis(Axis::new().labels(vec![Span::raw("0")])) // only 1 label
            .render(area, &mut buf, &mut state);

        // All 3 rows should be available for graph, no axis rendered
        // Row 0: dataset
        let cell_data = &buf[(8, 0)];
        assert_eq!(cell_data.fg, Color::Red, "Row 0 should have the dataset");

        // Row 2: should be empty (no axis taking space)
        let cell_r2 = &buf[(8, 2)];
        assert_eq!(cell_r2.symbol(), " ", "Row 2 should be empty (no axis)");
    }
}
