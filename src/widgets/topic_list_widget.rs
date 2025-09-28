use ratatui::{
    crossterm::event::{Event as CrosstermEvent, KeyCode},
    prelude::{BlockExt, Buffer, Rect},
    style::{Color, Modifier, Style, Styled},
    text::{Line, Span},
    widgets::{Block, StatefulWidget, Widget},
};

use nucleo_matcher::{
    pattern::{AtomKind, CaseMatching, Normalization, Pattern},
    Config, Matcher, Utf32Str,
};

use crate::{
    common::{event::Event, generic_message::InterfaceType, style::SELECTED_STYLE},
    widgets::TuiWidget,
};

pub struct TopicListWidget<'a> {
    block: Option<Block<'a>>,
    overlay: Option<Line<'a>>,
    auto_scroll: bool,
}

enum TopicListWidgetMode {
    Normal,
    Search,
}

pub struct TopicListWidgetState {
    // Topics
    topics: Vec<(String, InterfaceType, u32)>, // (topic name, topic type, rank)

    // Selection
    selected_index: usize,
    scroll_offset: usize,

    // Search
    filter: Option<String>,
    hidden_topics_count: usize,
    matcher: Matcher,

    mode: TopicListWidgetMode,

    needs_redraw: bool,
}

impl TopicListWidgetState {
    pub fn new(topics: Vec<(String, InterfaceType)>, selected_index: usize) -> Self {
        Self {
            topics: topics
                .iter()
                .map(|(name, itype)| (name.clone(), itype.clone(), 1))
                .collect(),
            selected_index,
            scroll_offset: 0,
            filter: None,
            hidden_topics_count: 0,
            matcher: Matcher::new(Config::DEFAULT),
            mode: TopicListWidgetMode::Normal,
            needs_redraw: true,
        }
    }

    pub fn next_topic(&mut self) {
        if !self.topics.is_empty() {
            self.selected_index =
                (self.selected_index + 1).min(self.topics.len() - 1 - self.hidden_topics_count);
            self.needs_redraw = true;
        }
    }

    pub fn previous_topic(&mut self) {
        if !self.topics.is_empty() {
            self.selected_index = self.selected_index.saturating_sub(1);
            self.needs_redraw = true;
        }
    }

    pub fn rank_topics(&mut self) {
        if let Some(filter) = &self.filter {
            let pattern = Pattern::new(
                filter,
                CaseMatching::Ignore,
                Normalization::Smart,
                AtomKind::Fuzzy,
            );
            self.topics.iter_mut().for_each(|(name, _, rank)| {
                *rank = pattern
                    .score(Utf32Str::Ascii(name.as_bytes()), &mut self.matcher)
                    .unwrap_or(0);
            });
            self.topics
                .sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));
            self.selected_index = 0;
            self.hidden_topics_count = self.topics.iter().filter(|(_, _, rank)| *rank == 0).count();
        } else {
            self.topics.iter_mut().for_each(|(_, _, rank)| *rank = 1);
            self.topics.sort_by(|a, b| a.0.cmp(&b.0));
            self.hidden_topics_count = 0;
        }
    }

    pub fn update(&mut self, new_topics: Vec<(String, InterfaceType)>) {
        if new_topics.len() == self.topics.len() {
            return;
        }

        if self.filter.is_some() {
            self.topics = new_topics
                .iter()
                .map(|(name, itype)| (name.clone(), itype.clone(), 1))
                .collect();
            self.rank_topics();
        } else {
            let new_topics: Vec<(String, InterfaceType, u32)> = new_topics
                .iter()
                .map(|(name, itype)| (name.clone(), itype.clone(), 1))
                .collect();

            if self.topics.is_empty() {
                self.selected_index = 0;
            } else {
                let selected_topic = self.topics.get(self.selected_index).unwrap().0.clone();
                let new_index = &new_topics
                    .iter()
                    .position(|topic| topic.0 == selected_topic)
                    .unwrap_or(0);
                self.selected_index = *new_index;
            }
            self.topics = new_topics;
        }
        self.needs_redraw = true;
    }

    pub fn handle_event_in_normal(&mut self, event: Event) -> Event {
        match event {
            Event::Key(CrosstermEvent::Key(key)) => match key.code {
                KeyCode::Char('/') => {
                    self.mode = TopicListWidgetMode::Search;
                    self.needs_redraw = true;
                    Event::None
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.next_topic();
                    Event::None
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.previous_topic();
                    Event::None
                }
                _ => event,
            },
            _ => event,
        }
    }

    pub fn handle_event_in_search(&mut self, event: Event) -> Event {
        match event {
            Event::Key(CrosstermEvent::Key(key)) => match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.mode = TopicListWidgetMode::Normal;
                    self.needs_redraw = true;
                    Event::None
                }
                KeyCode::Char(c) => {
                    self.append_to_filter(c);
                    Event::None
                }
                KeyCode::Down => {
                    self.next_topic();
                    Event::None
                }
                KeyCode::Up => {
                    self.previous_topic();
                    Event::None
                }
                KeyCode::Backspace => {
                    self.remove_from_filter();
                    Event::None
                }
                _ => event,
            },
            _ => event,
        }
    }

    fn append_to_filter(&mut self, c: char) {
        if let Some(filter) = &mut self.filter {
            filter.push(c);
        } else {
            self.filter = Some(c.to_string());
        }
        self.rank_topics();
        self.needs_redraw = true;
    }

    fn remove_from_filter(&mut self) {
        if let Some(filter) = &mut self.filter {
            filter.pop();
            if filter.is_empty() {
                self.filter = None;
            }
        }
        self.rank_topics();
        self.needs_redraw = true;
    }

    pub fn get_selected(&self) -> Option<(&String, &InterfaceType)> {
        self.topics
            .get(self.selected_index)
            .map(|(name, itype, _)| (name, itype))
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

impl TuiWidget for TopicListWidgetState {
    fn handle_event(&mut self, event: Event) -> Event {
        match self.mode {
            TopicListWidgetMode::Normal => self.handle_event_in_normal(event),
            TopicListWidgetMode::Search => self.handle_event_in_search(event),
        }
    }
}

impl<'a> Default for TopicListWidget<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> TopicListWidget<'a> {
    pub fn new() -> Self {
        Self {
            block: None,
            overlay: None,
            auto_scroll: false,
        }
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn overlay(mut self, overlay: Line<'a>) -> Self {
        self.overlay = Some(overlay);
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn auto_scroll(mut self, auto_scroll: bool) -> Self {
        self.auto_scroll = auto_scroll;
        self
    }
}

impl<'a> StatefulWidget for TopicListWidget<'a> {
    type State = TopicListWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.block.as_ref().render(area, buf);

        let inner_area = self.block.inner_if_some(area);

        let topic_list_area = Rect {
            x: inner_area.x,
            y: inner_area.y,
            width: inner_area.width,
            height: if state.filter.is_some() {
                inner_area.height.saturating_sub(1)
            } else {
                inner_area.height
            },
        };

        // Update scroll offset to ensure the selected item is visible
        if self.auto_scroll {
            state.scroll_offset = state
                .selected_index
                .saturating_sub((topic_list_area.height / 2).into())
                .min(
                    (state.topics.len().saturating_sub(state.hidden_topics_count))
                        .saturating_sub(topic_list_area.height.into()),
                );
        }

        let pattern = state.filter.as_ref().map(|f| {
            Pattern::new(
                f,
                CaseMatching::Ignore,
                Normalization::Smart,
                AtomKind::Fuzzy,
            )
        });

        // Iterate through all elements in the `items` and stylize them.
        for (i, (topic_name, topic_type, rank)) in
            state.topics.iter().enumerate().skip(state.scroll_offset)
        {
            if topic_list_area.height as usize <= i - state.scroll_offset {
                break;
            }

            if rank == &0 {
                // Topic are sorted by rank, so we can stop rendering here
                break;
            }

            let available_width = topic_list_area.width as usize;
            let remaining_space = available_width.saturating_sub(
                topic_name.len()
                    + topic_type.package_name.len()
                    + "/msg/".len()
                    + topic_type.type_name.len(),
            );

            let item_area = Rect {
                x: topic_list_area.x,
                y: topic_list_area.y + (i.saturating_sub(state.scroll_offset)) as u16,
                width: topic_list_area.width,
                height: 1,
            };

            let style = if i == state.selected_index {
                SELECTED_STYLE
            } else {
                Style::default()
            };

            let mut indices = Vec::new();
            pattern.as_ref().and_then(|p| {
                p.indices(
                    Utf32Str::Ascii(topic_name.as_bytes()),
                    &mut state.matcher,
                    &mut indices,
                )
            });

            Line::from_iter([
                Span::raw(topic_name),
                Span::raw(" ".repeat(remaining_space)),
                Span::raw(format!(
                    "{}/msg/{}",
                    topic_type.package_name, topic_type.type_name
                )),
            ])
            .set_style(style)
            .render(item_area, buf);

            for i in indices.iter() {
                if *i < available_width.try_into().unwrap() {
                    buf.cell_mut((item_area.x + *i as u16, item_area.y))
                        .map(|c| {
                            c.set_style(c.style().add_modifier(Modifier::BOLD));
                            ()
                        });
                }
            }

            if i != state.selected_index {
                continue;
            }

            if let Some(overlay) = &self.overlay {
                overlay.render(item_area, buf);
            }
        }

        // Render filter at the bottom if in search mode
        if state.filter.is_some() || matches!(state.mode, TopicListWidgetMode::Search) {
            let cursor_style = match state.mode {
                TopicListWidgetMode::Search => Style::default().bg(Color::White),
                _ => Style::default(),
            };
            let overlay = Line::from_iter([
                Span::raw("/"),
                Span::raw(state.filter.as_deref().unwrap_or("")),
                Span::raw(" ").style(cursor_style),
                Span::raw(
                    " ".repeat(
                        inner_area
                            .width
                            .saturating_sub(
                                (2 + state.filter.as_deref().map_or_else(|| 0, |f| f.len()))
                                    .try_into()
                                    .unwrap(),
                            )
                            .min(inner_area.width)
                            .into(),
                    ),
                ),
            ]);
            let overlay_area = Rect {
                x: inner_area.x,
                y: inner_area.y + inner_area.height.saturating_sub(1),
                width: inner_area.width,
                height: 1,
            };
            overlay.render(overlay_area, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ratatui::prelude::{Buffer, Line};

    #[track_caller]
    fn test_case_render<'a, Lines>(
        topics: &Vec<(String, InterfaceType)>,
        selected: usize,
        expected: Lines,
    ) where
        Lines: IntoIterator,
        Lines::Item: Into<Line<'a>>,
    {
        let mut state = TopicListWidgetState::new(topics.clone(), selected);
        let area = Rect::new(0, 0, 50, 10);
        let mut buf = Buffer::empty(area);
        let widget = TopicListWidget::new();
        widget.render(area, &mut buf, &mut state);
        assert_eq!(buf, Buffer::with_lines(expected));
    }

    #[test]
    fn test_topic_list_widget_render() {
        let topics = vec![
            (
                "topic1".to_string(),
                InterfaceType::new("std_msgs/msg/String"),
            ),
            (
                "topic2".to_string(),
                InterfaceType::new("sensor_msgs/msg/Image"),
            ),
            (
                "topic3".to_string(),
                InterfaceType::new("nav_msgs/msg/Odometry"),
            ),
        ];

        test_case_render(
            &topics,
            0,
            [
                Line::from("topic1                         std_msgs/msg/String")
                    .style(SELECTED_STYLE),
                "topic2                       sensor_msgs/msg/Image".into(),
                "topic3                       nav_msgs/msg/Odometry".into(),
                "                                                  ".into(),
                "                                                  ".into(),
                "                                                  ".into(),
                "                                                  ".into(),
                "                                                  ".into(),
                "                                                  ".into(),
                "                                                  ".into(),
            ],
        );

        test_case_render(
            &topics,
            1,
            [
                "topic1                         std_msgs/msg/String".into(),
                Line::from("topic2                       sensor_msgs/msg/Image")
                    .style(SELECTED_STYLE),
                "topic3                       nav_msgs/msg/Odometry".into(),
                "                                                  ".into(),
                "                                                  ".into(),
                "                                                  ".into(),
                "                                                  ".into(),
                "                                                  ".into(),
                "                                                  ".into(),
                "                                                  ".into(),
            ],
        );

        test_case_render(
            &topics,
            2,
            [
                "topic1                         std_msgs/msg/String".into(),
                "topic2                       sensor_msgs/msg/Image".into(),
                Line::from("topic3                       nav_msgs/msg/Odometry")
                    .style(SELECTED_STYLE),
                "                                                  ".into(),
                "                                                  ".into(),
                "                                                  ".into(),
                "                                                  ".into(),
                "                                                  ".into(),
                "                                                  ".into(),
                "                                                  ".into(),
            ],
        );
    }

    #[test]
    fn test_topic_list_widget_render_bordered() {
        let topics = vec![
            (
                "topic1".to_string(),
                InterfaceType::new("std_msgs/msg/String"),
            ),
            (
                "topic2".to_string(),
                InterfaceType::new("sensor_msgs/msg/Image"),
            ),
            (
                "topic3".to_string(),
                InterfaceType::new("nav_msgs/msg/Odometry"),
            ),
        ];

        let mut state = TopicListWidgetState::new(topics.clone(), 0);
        let area = Rect::new(0, 0, 50, 10);
        let mut buffer = Buffer::empty(area);
        let widget = TopicListWidget::new().block(Block::bordered().title("List"));
        widget.render(area, &mut buffer, &mut state);

        let mut expexted = Buffer::with_lines([
            "┌List────────────────────────────────────────────┐",
            "│topic1                       std_msgs/msg/String│",
            "│topic2                     sensor_msgs/msg/Image│",
            "│topic3                     nav_msgs/msg/Odometry│",
            "│                                                │",
            "│                                                │",
            "│                                                │",
            "│                                                │",
            "│                                                │",
            "└────────────────────────────────────────────────┘",
        ]);
        expexted.set_style(
            Rect {
                x: 1,
                y: 1,
                width: 48,
                height: 1,
            },
            SELECTED_STYLE,
        );

        assert_eq!(buffer, expexted);
    }
}
