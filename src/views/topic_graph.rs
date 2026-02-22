use std::{cell::RefCell, rc::Rc, time::Instant};

use crossterm::event::{Event as CrosstermEvent, KeyCode};
use ratatui::{
    layout::Spacing,
    prelude::{Buffer, Constraint, Layout, Rect},
    symbols::{border, line},
    text::Line,
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};

use crate::{
    common::{event::Event, style::SELECTED_STYLE},
    connections::{Connection, ConnectionType, NamedInterface, NodeName},
    views::{AcceptsNode, AcceptsTopic, FromNode, FromTopic, NodeInfo, TopicInfo, TuiView},
    widgets::list_widget::{ListWidget, ListWidgetState},
};

// ─── Border sets for collapsed panel borders ─────────────────────────────────

/// Left panel: rounded outer corners, T-junctions where it meets the centre.
const LEFT_BORDER_SET: border::Set = border::Set {
    top_left: line::ROUNDED_TOP_LEFT,
    top_right: line::HORIZONTAL_DOWN,
    bottom_left: line::ROUNDED_BOTTOM_LEFT,
    bottom_right: line::HORIZONTAL_UP,
    vertical_left: line::VERTICAL,
    vertical_right: line::VERTICAL,
    horizontal_top: line::HORIZONTAL,
    horizontal_bottom: line::HORIZONTAL,
};

/// Centre panel: T-junctions on every corner (shared with both neighbours).
const CENTER_BORDER_SET: border::Set = border::Set {
    top_left: line::HORIZONTAL_DOWN,
    top_right: line::HORIZONTAL_DOWN,
    bottom_left: line::HORIZONTAL_UP,
    bottom_right: line::HORIZONTAL_UP,
    vertical_left: line::VERTICAL,
    vertical_right: line::VERTICAL,
    horizontal_top: line::HORIZONTAL,
    horizontal_bottom: line::HORIZONTAL,
};

/// Right panel: T-junctions where it meets the centre, rounded outer corners.
const RIGHT_BORDER_SET: border::Set = border::Set {
    top_left: line::HORIZONTAL_DOWN,
    top_right: line::ROUNDED_TOP_RIGHT,
    bottom_left: line::HORIZONTAL_UP,
    bottom_right: line::ROUNDED_BOTTOM_RIGHT,
    vertical_left: line::VERTICAL,
    vertical_right: line::VERTICAL,
    horizontal_top: line::HORIZONTAL,
    horizontal_bottom: line::HORIZONTAL,
};

// ─── GraphFocus / GraphMode ───────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum GraphFocus {
    Left,
    Centre,
    Right,
}

#[derive(Clone)]
enum GraphMode {
    /// Showing a topic in the centre with its publishers (left) and subscribers (right).
    Topic(TopicInfo),
    /// Showing a node in the centre with its subscriptions (left) and publications (right).
    Node(NodeInfo),
}

// ─── TopicGraphState ─────────────────────────────────────────────────────────

pub struct TopicGraphState {
    connection: Rc<RefCell<ConnectionType>>,
    mode: GraphMode,
    focus: GraphFocus,

    // Topic mode panels
    publisher_nodes: ListWidgetState<NodeName>,
    subscriber_nodes: ListWidgetState<NodeName>,

    // Node mode panels
    subscribed_topics: ListWidgetState<NamedInterface>,
    published_topics: ListWidgetState<NamedInterface>,

    last_update: Instant,
    needs_redraw: bool,
}

impl TopicGraphState {
    fn new_for_topic(topic_info: TopicInfo) -> Self {
        let connection = topic_info.connection.clone();
        let (publishers, subscribers) = Self::fetch_topic_endpoints(&connection, &topic_info.topic);
        Self {
            connection,
            mode: GraphMode::Topic(topic_info),
            focus: GraphFocus::Centre,
            publisher_nodes: ListWidgetState::new(publishers, None),
            subscriber_nodes: ListWidgetState::new(subscribers, None),
            subscribed_topics: ListWidgetState::new(vec![], None),
            published_topics: ListWidgetState::new(vec![], None),
            last_update: Instant::now(),
            needs_redraw: true,
        }
    }

    fn new_for_node(node_info: NodeInfo) -> Self {
        let connection = node_info.connection.clone();
        let (subscriptions, publications) =
            Self::fetch_node_topics(&connection, &node_info.node_name);
        Self {
            connection,
            mode: GraphMode::Node(node_info),
            focus: GraphFocus::Centre,
            publisher_nodes: ListWidgetState::new(vec![], None),
            subscriber_nodes: ListWidgetState::new(vec![], None),
            subscribed_topics: ListWidgetState::new(subscriptions, None),
            published_topics: ListWidgetState::new(publications, None),
            last_update: Instant::now(),
            needs_redraw: true,
        }
    }

    // ─── Data fetching ───────────────────────────────────────────────────────

    fn fetch_topic_endpoints(
        connection: &Rc<RefCell<ConnectionType>>,
        topic: &str,
    ) -> (Vec<NodeName>, Vec<NodeName>) {
        let conn = connection.borrow();
        let publishers = conn.get_publishers_info_by_topic(topic).unwrap_or_default();
        let subscribers = conn
            .get_subscriptions_info_by_topic(topic)
            .unwrap_or_default();
        (publishers, subscribers)
    }

    fn fetch_node_topics(
        connection: &Rc<RefCell<ConnectionType>>,
        node_name: &NodeName,
    ) -> (Vec<NamedInterface>, Vec<NamedInterface>) {
        let conn = connection.borrow();

        let mut subscriptions = conn
            .get_subscription_names_and_types_by_node(node_name)
            .unwrap_or_default();
        subscriptions.sort_by(|a, b| a.name.cmp(&b.name));

        let mut publications = conn
            .get_publisher_names_and_types_by_node(node_name)
            .unwrap_or_default();
        publications.sort_by(|a, b| a.name.cmp(&b.name));

        (subscriptions, publications)
    }

    // ─── State transitions ───────────────────────────────────────────────────

    fn apply_topic(&mut self, topic_info: TopicInfo) {
        let (publishers, subscribers) =
            Self::fetch_topic_endpoints(&topic_info.connection, &topic_info.topic);
        self.connection = topic_info.connection.clone();
        self.publisher_nodes = ListWidgetState::new(publishers, None);
        self.subscriber_nodes = ListWidgetState::new(subscribers, None);
        self.mode = GraphMode::Topic(topic_info);
        self.focus = GraphFocus::Centre;
        self.last_update = Instant::now();
        self.needs_redraw = true;
    }

    fn apply_node(&mut self, node_info: NodeInfo) {
        let (subscriptions, publications) =
            Self::fetch_node_topics(&node_info.connection, &node_info.node_name);
        self.connection = node_info.connection.clone();
        self.subscribed_topics = ListWidgetState::new(subscriptions, None);
        self.published_topics = ListWidgetState::new(publications, None);
        self.mode = GraphMode::Node(node_info);
        self.focus = GraphFocus::Centre;
        self.last_update = Instant::now();
        self.needs_redraw = true;
    }

    fn update(&mut self) {
        const UPDATE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(1000);
        if self.last_update.elapsed() < UPDATE_INTERVAL {
            return;
        }
        self.last_update = Instant::now();
        match self.mode.clone() {
            GraphMode::Topic(topic_info) => {
                let (publishers, subscribers) =
                    Self::fetch_topic_endpoints(&self.connection, &topic_info.topic);
                self.publisher_nodes.update(publishers);
                self.subscriber_nodes.update(subscribers);
            }
            GraphMode::Node(node_info) => {
                let (subscriptions, publications) =
                    Self::fetch_node_topics(&self.connection, &node_info.node_name);
                self.subscribed_topics.update(subscriptions);
                self.published_topics.update(publications);
            }
        }
        self.needs_redraw = true;
    }

    // ─── Event helpers ───────────────────────────────────────────────────────

    /// Returns `true` if the panel is non-empty (an item was selected).
    fn try_select_first_left(&mut self) -> bool {
        match &self.mode {
            GraphMode::Topic(_) => {
                self.publisher_nodes.unselect(); // should already be unselected, but just in case
                self.publisher_nodes.next_item();
                self.publisher_nodes.get_selected().is_some()
            }
            GraphMode::Node(_) => {
                self.subscribed_topics.unselect(); // should already be unselected, but just in case
                self.subscribed_topics.next_item();
                self.subscribed_topics.get_selected().is_some()
            }
        }
    }

    /// Returns `true` if the panel is non-empty (an item was selected).
    fn try_select_first_right(&mut self) -> bool {
        match &self.mode {
            GraphMode::Topic(_) => {
                self.subscriber_nodes.unselect(); // should already be unselected, but just in case
                self.subscriber_nodes.next_item();
                self.subscriber_nodes.get_selected().is_some()
            }
            GraphMode::Node(_) => {
                self.published_topics.unselect(); // should already be unselected, but just in case
                self.published_topics.next_item();
                self.published_topics.get_selected().is_some()
            }
        }
    }

    /// Navigate the currently selected left-panel item into the centre.
    /// Returns `true` if navigation happened.
    fn navigate_left_to_centre(&mut self) -> bool {
        match self.mode.clone() {
            GraphMode::Topic(ti) => {
                if let Some(node_name) = self.publisher_nodes.get_selected().cloned() {
                    self.apply_node(NodeInfo {
                        connection: ti.connection.clone(),
                        node_name,
                    });
                    return true;
                }
            }
            GraphMode::Node(ni) => {
                if let Some(topic) = self.subscribed_topics.get_selected().cloned() {
                    self.apply_topic(TopicInfo {
                        connection: ni.connection.clone(),
                        topic: topic.name,
                        type_name: topic.type_name,
                    });
                    return true;
                }
            }
        }
        false
    }

    /// Navigate the currently selected right-panel item into the centre.
    /// Returns `true` if navigation happened.
    fn navigate_right_to_centre(&mut self) -> bool {
        match self.mode.clone() {
            GraphMode::Topic(ti) => {
                if let Some(node_name) = self.subscriber_nodes.get_selected().cloned() {
                    self.apply_node(NodeInfo {
                        connection: ti.connection.clone(),
                        node_name,
                    });
                    return true;
                }
            }
            GraphMode::Node(ni) => {
                if let Some(topic) = self.published_topics.get_selected().cloned() {
                    self.apply_topic(TopicInfo {
                        connection: ni.connection.clone(),
                        topic: topic.name,
                        type_name: topic.type_name,
                    });
                    return true;
                }
            }
        }
        false
    }

    /// Returns `Some(Event::None)` if the event was consumed by focus/navigation, otherwise `None`.
    fn try_switch_focus(&mut self, event: &Event) -> Option<Event> {
        if let Event::Key(CrosstermEvent::Key(key_event)) = event {
            match key_event.code {
                KeyCode::Char('h') | KeyCode::Left => {
                    match self.focus {
                        GraphFocus::Left => {
                            // At the leftmost edge: navigate selected item to centre
                            if self.navigate_left_to_centre() {
                                self.focus = GraphFocus::Centre;
                                self.needs_redraw = true;
                            }
                        }
                        GraphFocus::Centre => {
                            // Enter left panel only if it has items
                            if self.try_select_first_left() {
                                self.focus = GraphFocus::Left;
                                self.needs_redraw = true;
                            }
                        }
                        GraphFocus::Right => {
                            // Move back to centre, no content change
                            self.subscriber_nodes.unselect();
                            self.published_topics.unselect();
                            self.focus = GraphFocus::Centre;
                            self.needs_redraw = true;
                        }
                    }
                    return Some(Event::None);
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    match self.focus {
                        GraphFocus::Right => {
                            // At the rightmost edge: navigate selected item to centre
                            if self.navigate_right_to_centre() {
                                self.focus = GraphFocus::Centre;
                                self.needs_redraw = true;
                            }
                        }
                        GraphFocus::Centre => {
                            // Enter right panel only if it has items
                            if self.try_select_first_right() {
                                self.focus = GraphFocus::Right;
                                self.needs_redraw = true;
                            }
                        }
                        GraphFocus::Left => {
                            // Move back to centre, no content change
                            self.publisher_nodes.unselect();
                            self.subscribed_topics.unselect();
                            self.focus = GraphFocus::Centre;
                            self.needs_redraw = true;
                        }
                    }
                    return Some(Event::None);
                }
                _ => {}
            }
        }
        None
    }

    fn process_topic_mode(&mut self, event: Event, topic_info: TopicInfo) -> Event {
        match self.focus {
            GraphFocus::Centre => {
                if let Event::Key(CrosstermEvent::Key(key)) = &event {
                    if key.code == KeyCode::Enter {
                        return Event::NewTopic(topic_info);
                    }
                }
                event
            }
            GraphFocus::Left => {
                let new_event = self.publisher_nodes.handle_event(event);
                if let Event::Key(CrosstermEvent::Key(key_event)) = &new_event {
                    if key_event.code == KeyCode::Enter {
                        if let Some(node_name) = self.publisher_nodes.get_selected().cloned() {
                            return Event::NewNode(NodeInfo {
                                connection: topic_info.connection.clone(),
                                node_name,
                            });
                        }
                    }
                }
                new_event
            }
            GraphFocus::Right => {
                let new_event = self.subscriber_nodes.handle_event(event);
                if let Event::Key(CrosstermEvent::Key(key_event)) = &new_event {
                    if key_event.code == KeyCode::Enter {
                        if let Some(node_name) = self.subscriber_nodes.get_selected().cloned() {
                            return Event::NewNode(NodeInfo {
                                connection: topic_info.connection.clone(),
                                node_name,
                            });
                        }
                    }
                }
                new_event
            }
        }
    }

    fn process_node_mode(&mut self, event: Event, node_info: NodeInfo) -> Event {
        match self.focus {
            GraphFocus::Centre => {
                if let Event::Key(CrosstermEvent::Key(key)) = &event {
                    if key.code == KeyCode::Enter {
                        return Event::NewNode(node_info);
                    }
                }
                event
            }
            GraphFocus::Left => {
                let new_event = self.subscribed_topics.handle_event(event);
                if let Event::Key(CrosstermEvent::Key(key_event)) = &new_event {
                    if key_event.code == KeyCode::Enter {
                        if let Some(interface) = self.subscribed_topics.get_selected().cloned() {
                            return Event::NewTopic(TopicInfo {
                                connection: node_info.connection.clone(),
                                topic: interface.name.clone(),
                                type_name: interface.type_name.clone(),
                            });
                        }
                    }
                }
                new_event
            }
            GraphFocus::Right => {
                let new_event = self.published_topics.handle_event(event);
                if let Event::Key(CrosstermEvent::Key(key_event)) = &new_event {
                    if key_event.code == KeyCode::Enter {
                        if let Some(interface) = self.published_topics.get_selected().cloned() {
                            return Event::NewTopic(TopicInfo {
                                connection: node_info.connection.clone(),
                                topic: interface.name.clone(),
                                type_name: interface.type_name.clone(),
                            });
                        }
                    }
                }
                new_event
            }
        }
    }

    // ─── Rendering ───────────────────────────────────────────────────────────

    /// Wrap `name` into lines that fit within `inner_width` columns, preferring
    /// to break on `/` boundaries. If a single segment is longer than the
    /// available width it is placed on its own line and may be truncated during
    /// rendering.
    fn wrap_name(name: &str, inner_width: u16, focused: bool) -> Vec<Line<'static>> {
        let width = inner_width as usize;
        let segments: Vec<String> = name
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| format!("/{}", s))
            .collect();

        if segments.is_empty() {
            let text = name.to_owned();
            return vec![if focused {
                Line::styled(text, SELECTED_STYLE)
            } else {
                Line::raw(text)
            }];
        }

        let mut lines: Vec<String> = Vec::new();
        let mut current = String::new();

        for seg in &segments {
            if current.is_empty() {
                current = seg.clone();
            } else if current.len() + seg.len() <= width {
                current.push_str(seg);
            } else {
                lines.push(current);
                current = seg.clone();
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }

        lines
            .into_iter()
            .map(|text| {
                if focused {
                    Line::styled(text, SELECTED_STYLE)
                } else {
                    Line::raw(text)
                }
            })
            .collect()
    }

    fn render_topic_mode(area: Rect, buf: &mut Buffer, state: &mut Self, topic_info: &TopicInfo) {
        let [left_area, center_area, right_area] = Layout::horizontal([
            Constraint::Percentage(40),
            Constraint::Percentage(20),
            Constraint::Percentage(40),
        ])
        .spacing(Spacing::Overlap(1))
        .areas(area);

        // Left: publisher nodes
        let left_block = Block::bordered()
            .title(Line::raw("Publishers").centered())
            .border_set(LEFT_BORDER_SET);
        StatefulWidget::render(
            ListWidget::<NodeName>::new()
                .block(left_block)
                .auto_scroll(true),
            left_area,
            buf,
            &mut state.publisher_nodes,
        );

        // Centre: topic name
        let center_block = Block::bordered()
            .title(Line::raw("Topic").centered())
            .border_set(CENTER_BORDER_SET);
        let focused = state.focus == GraphFocus::Centre;
        let inner_width = center_area.width.saturating_sub(2); // subtract borders
        let topic_lines = Self::wrap_name(&topic_info.topic, inner_width, focused);
        Paragraph::new(topic_lines)
            .block(center_block)
            .render(center_area, buf);

        // Right: subscriber nodes
        let right_block = Block::bordered()
            .title(Line::raw("Subscribers").centered())
            .border_set(RIGHT_BORDER_SET);
        StatefulWidget::render(
            ListWidget::<NodeName>::new()
                .block(right_block)
                .auto_scroll(true),
            right_area,
            buf,
            &mut state.subscriber_nodes,
        );
    }

    fn render_node_mode(area: Rect, buf: &mut Buffer, state: &mut Self, node_info: &NodeInfo) {
        let [left_area, center_area, right_area] = Layout::horizontal([
            Constraint::Percentage(40),
            Constraint::Percentage(20),
            Constraint::Percentage(40),
        ])
        .spacing(Spacing::Overlap(1))
        .areas(area);

        // Left: subscribed topics
        let left_block = Block::bordered()
            .title(Line::raw("Subscriptions").centered())
            .border_set(LEFT_BORDER_SET);
        StatefulWidget::render(
            ListWidget::<NamedInterface>::new()
                .block(left_block)
                .auto_scroll(true),
            left_area,
            buf,
            &mut state.subscribed_topics,
        );

        // Centre: node name
        let center_block = Block::bordered()
            .title(Line::raw("Node").centered())
            .border_set(CENTER_BORDER_SET);
        let focused = state.focus == GraphFocus::Centre;
        let inner_width = center_area.width.saturating_sub(2); // subtract borders
        let node_lines = Self::wrap_name(&node_info.node_name.full_name(), inner_width, focused);
        Paragraph::new(node_lines)
            .block(center_block)
            .render(center_area, buf);

        // Right: published topics
        let right_block = Block::bordered()
            .title(Line::raw("Publications").centered())
            .border_set(RIGHT_BORDER_SET);
        StatefulWidget::render(
            ListWidget::<NamedInterface>::new()
                .block(right_block)
                .auto_scroll(true),
            right_area,
            buf,
            &mut state.published_topics,
        );
    }
}

// ─── TuiView ─────────────────────────────────────────────────────────────────

impl TuiView for TopicGraphState {
    fn handle_event(&mut self, event: Event) -> Event {
        self.update();
        if let Some(e) = self.try_switch_focus(&event) {
            return e;
        }
        let mode = self.mode.clone();
        match mode {
            GraphMode::Topic(ti) => self.process_topic_mode(event, ti),
            GraphMode::Node(ni) => self.process_node_mode(event, ni),
        }
    }

    fn name(&self) -> String {
        match &self.mode {
            GraphMode::Topic(ti) => format!("Graph: {}", ti.topic),
            GraphMode::Node(ni) => format!("Graph: {}", ni.node_name.full_name()),
        }
    }

    fn get_help_text(&self) -> String {
        "Topic Graph View Help:\n\
        Navigation:\n\
        - 'h' or ←  (from centre): Move to left panel, select first item.\n\
        - 'l' or →  (from centre): Move to right panel, select first item.\n\
        - 'l' or →  (from left panel): Move back to centre (no content change).\n\
        - 'h' or ←  (from right panel): Move back to centre (no content change).\n\
        - 'h' or ←  (past left edge): Navigate selected item to centre.\n\
        - 'l' or →  (past right edge): Navigate selected item to centre.\n\
        - 'j' or ↓ / 'k' or ↑: Move up/down within the focused side panel.\n\
        - 'Enter': Open the NodeInfo/TopicInfo popup for the focused item.\n\
        In Topic mode: left = publisher nodes, right = subscriber nodes.\n\
        In Node mode: left = subscribed topics, right = published topics."
            .to_string()
    }

    fn needs_redraw(&mut self) -> bool {
        self.publisher_nodes.needs_redraw()
            | self.subscriber_nodes.needs_redraw()
            | self.subscribed_topics.needs_redraw()
            | self.published_topics.needs_redraw()
            | std::mem::replace(&mut self.needs_redraw, false)
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let mode = self.mode.clone();
        match &mode {
            GraphMode::Topic(ti) => {
                Self::render_topic_mode(area, buf, self, ti);
            }
            GraphMode::Node(ni) => {
                Self::render_node_mode(area, buf, self, ni);
            }
        }
    }

    fn as_topic_acceptor(&mut self) -> Option<&mut dyn AcceptsTopic> {
        Some(self)
    }

    fn as_node_acceptor(&mut self) -> Option<&mut dyn AcceptsNode> {
        Some(self)
    }
}

// ─── Trait impls ─────────────────────────────────────────────────────────────

impl AcceptsTopic for TopicGraphState {
    fn accepts_topic(&mut self, topic_info: TopicInfo) {
        self.apply_topic(topic_info);
    }
}

impl AcceptsNode for TopicGraphState {
    fn accepts_node(&mut self, node_info: NodeInfo) {
        self.apply_node(node_info);
    }
}

impl FromTopic for TopicGraphState {
    fn from_topic(topic_info: TopicInfo) -> Self {
        Self::new_for_topic(topic_info)
    }
}

impl FromNode for TopicGraphState {
    fn from_node(node_info: NodeInfo) -> Self {
        Self::new_for_node(node_info)
    }
}
