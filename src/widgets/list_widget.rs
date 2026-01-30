use ratatui::{
    crossterm::event::{Event as CrosstermEvent, KeyCode},
    prelude::{BlockExt, Buffer, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, StatefulWidget, Widget},
};

use nucleo_matcher::{
    pattern::{AtomKind, CaseMatching, Normalization, Pattern},
    Config, Matcher, Utf32Str,
};

use crate::{common::event::Event, widgets::TuiWidget};

pub trait ListItemTrait {
    fn search_text(&self) -> String;

    fn to_line(&self, width: usize, selected: bool, indices: Vec<u32>) -> Line<'_>;
}

pub struct ListWidget<'a, ItemType> {
    block: Option<Block<'a>>,

    show_mode: bool,
    enable_search: bool,

    auto_scroll: bool,

    _phantom: std::marker::PhantomData<ItemType>,
}

#[derive(Default)]
enum ListWidgetMode {
    #[default]
    Normal,
    Search,
}


#[derive(Default)]
pub struct ListWidgetState<ItemType>
where
    ItemType: ListItemTrait,
{
    // Topics
    items: Vec<(ItemType, u32)>, // (Item, rank)

    // Selection
    selected_index: Option<usize>,
    scroll_offset: usize,

    // Search
    filter: String,
    hidden_nodes_count: usize,
    matcher: Matcher,

    mode: ListWidgetMode,

    needs_redraw: bool,
}

impl<ItemType> ListWidgetState<ItemType>
where
    ItemType: ListItemTrait + Clone,
{
    pub fn new(items: Vec<ItemType>, selected_index: Option<usize>) -> Self {
        Self {
            items: items.iter().map(|item| (item.clone(), 1)).collect(),
            selected_index,
            scroll_offset: 0,
            filter: "".to_string(),
            hidden_nodes_count: 0,
            matcher: Matcher::new(Config::DEFAULT),
            mode: ListWidgetMode::Normal,
            needs_redraw: true,
        }
    }

    pub fn reset(&mut self) {
        self.items.clear();
        self.selected_index = None;
        self.scroll_offset = 0;
        self.filter.clear();
        self.hidden_nodes_count = 0;
        self.mode = ListWidgetMode::Normal;
        self.needs_redraw = true;
    }

    pub fn next_item(&mut self) {
        if !self.items.is_empty() {
            if let Some(selected_index) = &mut self.selected_index {
                *selected_index =
                    (*selected_index + 1).min(self.items.len() - 1 - self.hidden_nodes_count);
            } else {
                self.selected_index = Some(0);
            }
            self.needs_redraw = true;
        }
    }

    pub fn previous_item(&mut self) {
        if !self.items.is_empty() {
            if let Some(selected_index) = &mut self.selected_index {
                *selected_index = selected_index.saturating_sub(1);
            } else {
                self.selected_index = Some(self.items.len() - 1 - self.hidden_nodes_count);
            }
            self.needs_redraw = true;
        }
    }

    pub fn sort_items(&mut self) {
        self.items.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then(a.0.search_text().cmp(&b.0.search_text()))
        });
        self.needs_redraw = true;
    }

    pub fn rank_topics(&mut self) {
        if self.filter.is_empty() {
            self.items.iter_mut().for_each(|(_, rank)| *rank = 1);
            self.hidden_nodes_count = 0;
        } else {
            let pattern = Pattern::new(
                &self.filter,
                CaseMatching::Ignore,
                Normalization::Smart,
                AtomKind::Fuzzy,
            );
            self.items.iter_mut().for_each(|(item, rank)| {
                *rank = pattern
                    .score(
                        Utf32Str::Ascii(item.search_text().as_bytes()),
                        &mut self.matcher,
                    )
                    .unwrap_or(0);
            });
            self.selected_index = if self.selected_index.is_some() {
                Some(0)
            } else {
                None
            };
            self.hidden_nodes_count = self.items.iter().filter(|(_, rank)| *rank == 0).count();
        }
        self.sort_items();
    }

    pub fn update(&mut self, new_items: Vec<ItemType>) {
        if new_items.len() == self.items.len() {
            return;
        }

        if self.filter.is_empty() {
            let new_items: Vec<(ItemType, u32)> =
                new_items.iter().map(|item| (item.clone(), 1)).collect();

            if self.items.is_empty() {
                self.selected_index = None;
            } else if let Some(selected_index) = &mut self.selected_index {
                let selected_item_str = self.items.get(*selected_index).unwrap().0.search_text();
                let new_index = &new_items
                    .iter()
                    .position(|(item, _)| item.search_text() == selected_item_str)
                    .unwrap_or(0);
                *selected_index = *new_index;
            } else {
                self.selected_index = None;
            }
            self.items = new_items;
        } else {
            self.items = new_items.iter().map(|item| (item.clone(), 1)).collect();
            self.rank_topics();
        }
        self.needs_redraw = true;
    }

    pub fn handle_event_in_normal(&mut self, event: Event) -> Event {
        match event {
            Event::Key(CrosstermEvent::Key(key)) => match key.code {
                KeyCode::Char('/') => {
                    self.mode = ListWidgetMode::Search;
                    self.needs_redraw = true;
                    Event::None
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.next_item();
                    Event::None
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.previous_item();
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
                    self.mode = ListWidgetMode::Normal;
                    self.needs_redraw = true;
                    Event::None
                }
                KeyCode::Char(c) => {
                    self.append_to_filter(c);
                    Event::None
                }
                KeyCode::Down => {
                    self.next_item();
                    Event::None
                }
                KeyCode::Up => {
                    self.previous_item();
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

    pub fn handle_event(&mut self, event: Event) -> Event {
        match self.mode {
            ListWidgetMode::Normal => self.handle_event_in_normal(event),
            ListWidgetMode::Search => self.handle_event_in_search(event),
        }
    }

    fn append_to_filter(&mut self, c: char) {
        self.filter.push(c);
        self.rank_topics();
        self.needs_redraw = true;
    }

    fn remove_from_filter(&mut self) {
        self.filter.pop();
        self.rank_topics();
        self.needs_redraw = true;
    }

    pub fn get_selected(&self) -> Option<&ItemType> {
        if let Some(selected_index) = self.selected_index {
            self.items.get(selected_index).map(|(item, _)| item)
        } else {
            None
        }
    }

    pub fn get_selected_index(&self) -> Option<usize> {
        self.selected_index
    }
    pub fn unselect(&mut self) {
        self.selected_index = None;
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

impl<ItemType> TuiWidget for ListWidgetState<ItemType>
where
    ItemType: ListItemTrait + Clone,
{
    fn handle_event(&mut self, event: Event) -> Event {
        match self.mode {
            ListWidgetMode::Normal => self.handle_event_in_normal(event),
            ListWidgetMode::Search => self.handle_event_in_search(event),
        }
    }
}

impl<'a, ItemType: ListItemTrait> Default for ListWidget<'a, ItemType> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, ItemType: ListItemTrait> ListWidget<'a, ItemType> {
    pub fn new() -> Self {
        Self {
            block: None,
            show_mode: true,
            enable_search: true,
            auto_scroll: false,
            _phantom: std::marker::PhantomData,
        }
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn auto_scroll(mut self, auto_scroll: bool) -> Self {
        self.auto_scroll = auto_scroll;
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn enable_search(mut self, enable_search: bool) -> Self {
        self.enable_search = enable_search;
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn show_mode(mut self, show_mode: bool) -> Self {
        self.show_mode = show_mode;
        self
    }

    pub fn height(&self, state: &ListWidgetState<ItemType>) -> usize {
        if self.auto_scroll {
            1
        } else if state.filter.is_empty() && !matches!(state.mode, ListWidgetMode::Search) {
            state.items.len()
        } else {
            state.items.len().saturating_sub(state.hidden_nodes_count) + 1 // +1 for filter
        }
    }
}

impl<'a, ItemType: ListItemTrait> StatefulWidget for ListWidget<'a, ItemType> {
    type State = ListWidgetState<ItemType>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.block.as_ref().render(area, buf);

        let inner_area = self.block.inner_if_some(area);

        let topic_list_area = Rect {
            x: inner_area.x,
            y: inner_area.y,
            width: inner_area.width,
            height: if state.filter.is_empty() {
                inner_area.height
            } else {
                inner_area.height.saturating_sub(1)
            },
        };

        // Update scroll offset to ensure the selected item is visible
        if self.auto_scroll {
            if let Some(selected_index) = state.selected_index {
                state.scroll_offset = selected_index
                    .saturating_sub((topic_list_area.height / 2).into())
                    .min(
                        (state.items.len().saturating_sub(state.hidden_nodes_count))
                            .saturating_sub(topic_list_area.height.into()),
                    );
            }
        }

        let pattern = if state.filter.is_empty() {
            None
        } else {
            Some(Pattern::new(
                &state.filter,
                CaseMatching::Ignore,
                Normalization::Smart,
                AtomKind::Fuzzy,
            ))
        };

        // Iterate through all elements in the `items` and stylize them.
        for (i, (item, rank)) in state.items.iter().enumerate().skip(state.scroll_offset) {
            if topic_list_area.height as usize <= i - state.scroll_offset {
                break;
            }

            // Topic are sorted by rank, so we can stop rendering here
            if rank == &0 {
                break;
            }

            let available_width = topic_list_area.width as usize;

            let item_area = Rect {
                x: topic_list_area.x,
                y: topic_list_area.y + (i.saturating_sub(state.scroll_offset)) as u16,
                width: topic_list_area.width,
                height: 1,
            };

            let mut indices = Vec::new();
            pattern.as_ref().and_then(|p| {
                p.indices(
                    Utf32Str::Ascii(item.search_text().as_bytes()),
                    &mut state.matcher,
                    &mut indices,
                )
            });

            // Render the item
            item.to_line(
                available_width,
                state.selected_index.is_some_and(|s| s == i),
                indices,
            )
            .render(item_area, buf);
        }

        // Render filter at the bottom if in search mode
        if !state.filter.is_empty() || matches!(state.mode, ListWidgetMode::Search) {
            let cursor_style = match state.mode {
                ListWidgetMode::Search => Style::default().bg(Color::White),
                _ => Style::default(),
            };
            let overlay = Line::from_iter([
                Span::raw("/"),
                Span::raw(&state.filter),
                Span::raw(" ").style(cursor_style),
            ]);

            // In auto_scroll mode, the overlay is displayed at the very bottom
            // In non-auto_scroll mode, the overlay is displayed right after the last item
            let overlay_area = if self.auto_scroll {
                Rect {
                    x: area.x + 1,
                    y: area.y + area.height.saturating_sub(1),
                    width: area.width,
                    height: 1,
                }
            } else {
                Rect {
                    x: topic_list_area.x,
                    y: topic_list_area.y
                        + (state
                            .items
                            .len()
                            .saturating_sub(state.hidden_nodes_count)
                            .min(topic_list_area.height as usize)) as u16,
                    width: topic_list_area.width,
                    height: 1,
                }
            };

            overlay.render(overlay_area, buf);
        }
        // Render mode at the bottom right
        if self.show_mode {
            let mode_text = match state.mode {
                ListWidgetMode::Normal => "NORMAL",
                ListWidgetMode::Search => "SEARCH",
            };
            let mode = Line::from_iter([Span::raw(mode_text).style(Style::default())]);
            let mode_area = Rect {
                x: area.x + area.width.saturating_sub(mode.width() as u16) - 1,
                y: area.y + area.height.saturating_sub(1),
                width: mode.width() as u16,
                height: 1,
            };
            mode.render(mode_area, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::common::style::SELECTED_STYLE;

    use super::*;

    use ratatui::prelude::{Buffer, Line};

    #[derive(Clone)]
    struct TestItem {
        pub name: String,
        pub interface_type: String,
    }

    impl ListItemTrait for TestItem {
        fn search_text(&self) -> String {
            self.name.clone()
        }

        fn to_line(&'_ self, width: usize, selected: bool, _indices: Vec<u32>) -> Line<'_> {
            let inter_width = width.saturating_sub(self.name.len() + self.interface_type.len());
            let style = if selected {
                SELECTED_STYLE
            } else {
                Style::default()
            };
            Line::from_iter([
                Span::raw(self.name.clone()),
                Span::raw(" ".repeat(inter_width)),
                Span::raw(self.interface_type.clone()),
            ])
            .style(style)
        }
    }

    #[track_caller]
    fn test_case_render<'a, Lines>(items: &Vec<TestItem>, selected: usize, expected: Lines)
    where
        Lines: IntoIterator,
        Lines::Item: Into<Line<'a>>,
    {
        let mut state = ListWidgetState::new(items.clone(), Some(selected));
        let area = Rect::new(0, 0, 50, 10);
        let mut buf = Buffer::empty(area);
        let widget = ListWidget::<TestItem>::new();
        widget.render(area, &mut buf, &mut state);
        assert_eq!(buf, Buffer::with_lines(expected));
    }

    #[test]
    fn test_topic_list_widget_render() {
        let topics = vec![
            TestItem {
                name: "topic1".to_string(),
                interface_type: "std_msgs/msg/String".to_string(),
            },
            TestItem {
                name: "topic2".to_string(),
                interface_type: "sensor_msgs/msg/Image".to_string(),
            },
            TestItem {
                name: "topic3".to_string(),
                interface_type: "nav_msgs/msg/Odometry".to_string(),
            },
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
                "                                           NORMAL ".into(),
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
                "                                           NORMAL ".into(),
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
                "                                           NORMAL ".into(),
            ],
        );
    }

    #[test]
    fn test_topic_list_widget_render_bordered() {
        let topics = vec![
            TestItem {
                name: "topic1".to_string(),
                interface_type: "std_msgs/msg/String".to_string(),
            },
            TestItem {
                name: "topic2".to_string(),
                interface_type: "sensor_msgs/msg/Image".to_string(),
            },
            TestItem {
                name: "topic3".to_string(),
                interface_type: "nav_msgs/msg/Odometry".to_string(),
            },
        ];

        let mut state = ListWidgetState::new(topics.clone(), Some(0));
        let area = Rect::new(0, 0, 50, 10);
        let mut buffer = Buffer::empty(area);
        let widget = ListWidget::<TestItem>::new().block(Block::bordered().title("List"));
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
            "└──────────────────────────────────────────NORMAL┘",
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
