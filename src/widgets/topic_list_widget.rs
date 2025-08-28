use ratatui::{
    crossterm::event::{Event as CrosstermEvent, KeyCode},
    prelude::{BlockExt, Buffer, Rect},
    style::Style,
    widgets::{Block, StatefulWidget, Widget},
};

use crate::{common::event::Event, common::style::SELECTED_STYLE, widgets::TuiWidget};

// TODO(@TonyWelte): Remove dependency on rclrs in widgets module
use rclrs::MessageTypeName;

pub struct TopicListWidget<'a> {
    block: Option<Block<'a>>,
}

pub struct TopicListWidgetState {
    pub topics: Vec<(String, MessageTypeName)>,
    pub selected_index: usize,
}

impl TopicListWidgetState {
    pub fn new(topics: Vec<(String, MessageTypeName)>, selected_index: Option<usize>) -> Self {
        Self {
            topics,
            selected_index: selected_index.unwrap_or(0),
        }
    }

    pub fn next_topic(&mut self) {
        if !self.topics.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.topics.len();
        }
    }

    pub fn previous_topic(&mut self) {
        if !self.topics.is_empty() {
            self.selected_index = (self.selected_index + self.topics.len() - 1) % self.topics.len();
        }
    }

    pub fn update(&mut self, new_topics: Vec<(String, MessageTypeName)>) {
        if self.topics.is_empty() {
            self.topics = new_topics;
            self.selected_index = 0;
        } else if new_topics != self.topics {
            let selected_topic = self.topics.get(self.selected_index).unwrap().0.clone();
            let new_index = new_topics
                .iter()
                .position(|topic| topic.0 == selected_topic)
                .unwrap_or(0);
            self.topics = new_topics;
            self.selected_index = new_index;
        }
    }
}

impl TuiWidget for TopicListWidgetState {
    fn handle_event(&mut self, event: Event) -> Event {
        match event {
            Event::Key(CrosstermEvent::Key(key)) => match key.code {
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
}

impl<'a> TopicListWidget<'a> {
    pub fn new() -> Self {
        Self { block: None }
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> StatefulWidget for TopicListWidget<'a> {
    type State = TopicListWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.block.as_ref().render(area, buf);

        let inner_area = self.block.inner_if_some(area);

        // Iterate through all elements in the `items` and stylize them.
        for (i, (topic_name, topic_type)) in state.topics.iter().enumerate() {
            let style = if i == state.selected_index {
                SELECTED_STYLE
            } else {
                Style::default()
            };
            let available_width = inner_area.width as usize;
            let remaining_space = available_width.saturating_sub(
                topic_name.len()
                    + topic_type.package_name.len()
                    + "/msg/".len()
                    + topic_type.type_name.len(),
            );

            let text = format!(
                "{}{:>remaining_space$}{}/msg/{}",
                topic_name, "", topic_type.package_name, topic_type.type_name
            );
            buf.set_string(inner_area.x, inner_area.y + i as u16, text, style);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ratatui::prelude::{Buffer, Line};

    #[track_caller]
    fn test_case_render<'a, Lines>(
        topics: &Vec<(String, MessageTypeName)>,
        selected: Option<usize>,
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
                MessageTypeName {
                    package_name: "std_msgs".to_string(),
                    type_name: "String".to_string(),
                },
            ),
            (
                "topic2".to_string(),
                MessageTypeName {
                    package_name: "sensor_msgs".to_string(),
                    type_name: "Image".to_string(),
                },
            ),
            (
                "topic3".to_string(),
                MessageTypeName {
                    package_name: "nav_msgs".to_string(),
                    type_name: "Odometry".to_string(),
                },
            ),
        ];

        test_case_render(
            &topics,
            None,
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
            Some(0),
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
            Some(1),
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
            Some(2),
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
                MessageTypeName {
                    package_name: "std_msgs".to_string(),
                    type_name: "String".to_string(),
                },
            ),
            (
                "topic2".to_string(),
                MessageTypeName {
                    package_name: "sensor_msgs".to_string(),
                    type_name: "Image".to_string(),
                },
            ),
            (
                "topic3".to_string(),
                MessageTypeName {
                    package_name: "nav_msgs".to_string(),
                    type_name: "Odometry".to_string(),
                },
            ),
        ];

        let mut state = TopicListWidgetState::new(topics.clone(), None);
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
