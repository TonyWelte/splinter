use ratatui::{
    crossterm::event::{Event as CrosstermEvent, KeyCode},
    prelude::{BlockExt, Buffer, Rect},
    style::{Color, Style, Styled},
    text::{Line, Span},
    widgets::{Block, StatefulWidget, Widget},
};

use crate::{
    common::{event::Event, generic_message::InterfaceType, style::SELECTED_STYLE},
    widgets::TuiWidget,
};

pub struct TopicListWidget<'a> {
    block: Option<Block<'a>>,
    overlay: Option<Line<'a>>,
}

enum TopicListWidgetMode {
    Normal,
    Search,
}

pub struct TopicListWidgetState {
    pub filtered_topics: Vec<(String, InterfaceType)>,
    pub selected_index: usize,
    scroll_offset: usize,

    pub filter: Option<String>,
    all_topics: Vec<(String, InterfaceType)>,

    mode: TopicListWidgetMode,
}

impl TopicListWidgetState {
    pub fn new(topics: Vec<(String, InterfaceType)>, selected_index: usize) -> Self {
        Self {
            filtered_topics: topics.clone(),
            selected_index,
            scroll_offset: 0,
            filter: None,
            all_topics: topics,
            mode: TopicListWidgetMode::Normal,
        }
    }

    pub fn next_topic(&mut self) {
        if !self.filtered_topics.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_topics.len();
        }
    }

    pub fn previous_topic(&mut self) {
        if !self.filtered_topics.is_empty() {
            self.selected_index =
                (self.selected_index + self.filtered_topics.len() - 1) % self.filtered_topics.len();
        }
    }

    pub fn update(&mut self, new_topics: Vec<(String, InterfaceType)>) {
        let new_filtered_topics = if let Some(filter) = &self.filter {
            new_topics
                .iter()
                .filter(|(topic, _)| topic.contains(filter))
                .cloned()
                .collect()
        } else {
            new_topics.clone()
        };

        if self.all_topics.is_empty() {
            self.all_topics = new_topics;
            self.filtered_topics = new_filtered_topics;
            self.selected_index = 0;
        } else if new_topics != self.all_topics {
            let selected_topic = self
                .filtered_topics
                .get(self.selected_index)
                .unwrap()
                .0
                .clone();
            let new_index = &new_filtered_topics
                .iter()
                .position(|topic| topic.0 == selected_topic)
                .unwrap_or(0);
            self.filtered_topics = new_filtered_topics;
            self.selected_index = *new_index;
        }
    }

    fn update_filtered(&mut self) {
        if let Some(filter) = &self.filter {
            let selected_topic = self
                .filtered_topics
                .get(self.selected_index)
                .map(|(topic, _)| topic.clone());

            self.filtered_topics = self
                .all_topics
                .iter()
                .filter(|(topic, _)| topic.contains(filter))
                .cloned()
                .collect();

            if let Some(selected_topic) = selected_topic {
                self.selected_index = self
                    .filtered_topics
                    .iter()
                    .position(|(topic, _)| *topic == selected_topic)
                    .unwrap_or(0);
            } else {
                self.selected_index = 0;
            }
        } else {
            self.filtered_topics = self.all_topics.clone();
        }
    }

    pub fn handle_event_in_normal(&mut self, event: Event) -> Event {
        match event {
            Event::Key(CrosstermEvent::Key(key)) => match key.code {
                KeyCode::Char('/') => {
                    self.mode = TopicListWidgetMode::Search;
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
                KeyCode::Esc => {
                    self.mode = TopicListWidgetMode::Normal;
                    Event::None
                }
                KeyCode::Char(c) => {
                    if let Some(filter) = &mut self.filter {
                        filter.push(c);
                    } else {
                        self.filter = Some(c.to_string());
                    }
                    self.update_filtered();
                    Event::None
                }
                KeyCode::Backspace => {
                    if let Some(filter) = &mut self.filter {
                        filter.pop();
                        if filter.is_empty() {
                            self.filter = None;
                        }
                    }
                    self.update_filtered();
                    Event::None
                }
                _ => event,
            },
            _ => event,
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

impl<'a> TopicListWidget<'a> {
    pub fn new() -> Self {
        Self {
            block: None,
            overlay: None,
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
}

impl<'a> StatefulWidget for TopicListWidget<'a> {
    type State = TopicListWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.block.as_ref().render(area, buf);

        let inner_area = self.block.inner_if_some(area);

        // Update scroll offset to ensure the selected item is visible
        if state.selected_index.saturating_sub(state.scroll_offset) >= inner_area.height as usize {
            state.scroll_offset = state.selected_index + 1 - inner_area.height as usize;
        } else if state.selected_index < state.scroll_offset {
            state.scroll_offset = state.selected_index;
        }

        // Iterate through all elements in the `items` and stylize them.
        for (i, (topic_name, topic_type)) in state
            .filtered_topics
            .iter()
            .enumerate()
            .skip(state.scroll_offset)
        {
            if inner_area.height as usize <= i - state.scroll_offset {
                break;
            }

            let available_width = inner_area.width as usize;
            let remaining_space = available_width.saturating_sub(
                topic_name.len()
                    + topic_type.package_name.len()
                    + "/msg/".len()
                    + topic_type.type_name.len(),
            );

            let item_area = Rect {
                x: inner_area.x,
                y: inner_area.y + (i.saturating_sub(state.scroll_offset)) as u16,
                width: inner_area.width,
                height: 1,
            };

            let style = if i == state.selected_index {
                SELECTED_STYLE
            } else {
                Style::default()
            };

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

            if i != state.selected_index {
                continue;
            }

            if let Some(overlay) = &self.overlay {
                overlay.render(item_area, buf);
            }
        }

        // Render filter at the bottom if in search mode
        if let TopicListWidgetMode::Search = state.mode {
            let overlay = Line::from_iter([
                Span::raw("/"),
                Span::raw(state.filter.as_deref().unwrap_or("")),
                Span::raw(" ").style(Style::default().bg(Color::White)),
                Span::raw(
                    " ".repeat(
                        inner_area
                            .width
                            .saturating_sub(
                                (2 + state.filter.clone().map_or_else(|| 0, |f| f.len()))
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
