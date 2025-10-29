use std::{cell::RefCell, rc::Rc};

use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Style, Styled, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, StatefulWidget},
};

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEventKind};

use crate::{
    common::{
        event::{Event, NewHzEvent, NewPublisherEvent, NewTopicEvent},
        generic_message::InterfaceType,
        style::{HEADER_STYLE, SELECTED_STYLE},
        utils::truncate_namespaces,
    },
    connections::{Connection, ConnectionType},
    views::TuiView,
    widgets::{
        list_widget::{ListItemTrait, ListWidget, ListWidgetState},
        TuiWidget,
    },
};

pub struct TopicList;

enum Action {
    Echo,
    Pub,
    FrequencyPlot,
}

impl Action {
    pub fn next(&self) -> Self {
        match self {
            Action::Echo => Action::Pub,
            Action::Pub => Action::FrequencyPlot,
            Action::FrequencyPlot => Action::FrequencyPlot,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Action::Echo => Action::Echo,
            Action::Pub => Action::Echo,
            Action::FrequencyPlot => Action::Pub,
        }
    }
}

#[derive(Clone, Debug)]
struct Topic {
    name: String,
    type_name: InterfaceType,
}

impl ListItemTrait for Topic {
    fn search_text(&self) -> String {
        self.name.clone()
    }

    fn to_line(&'_ self, width: usize, selected: bool, indices: Vec<u32>) -> Line<'_> {
        let (truncated_name, new_indices) = truncate_namespaces(&self.name, &indices, width);

        let mut spans = vec![];
        if new_indices.is_empty() {
            spans.push(Span::raw(truncated_name));
        } else {
            let first_idx = new_indices.first().unwrap();
            if *first_idx != 0 {
                spans.push(Span::raw(truncated_name[..*first_idx as usize].to_string()));
            }

            for window in new_indices.windows(2) {
                let idx = window[0] as usize;
                let next_idx = window[1] as usize;
                spans.push(Span::styled(
                    truncated_name[idx..idx + 1].to_string(),
                    Style::default().bold(),
                ));
                if next_idx > idx + 1 {
                    spans.push(Span::raw(truncated_name[idx + 1..next_idx].to_string()));
                }
            }

            let last_idx = new_indices.last().unwrap();
            let idx = *last_idx as usize;
            spans.push(Span::styled(
                truncated_name[idx..idx + 1].to_string(),
                Style::default().bold(),
            ));
            if truncated_name.len() > idx + 1 {
                spans.push(Span::raw(truncated_name[idx + 1..].to_string()));
            }
        }

        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            self.type_name.to_string(),
            Style::default().fg(Color::DarkGray),
        ));

        let mut line = Line::from(spans);

        if selected {
            line = line.set_style(SELECTED_STYLE);
        }

        line
    }
}

pub struct TopicListState {
    connection: Rc<RefCell<ConnectionType>>,
    state: ListWidgetState<Topic>,
    action: Action,

    needs_redraw: bool,
}

impl TopicListState {
    pub fn new(connection: Rc<RefCell<ConnectionType>>) -> Self {
        let topics = connection
            .borrow()
            .list_topics()
            .unwrap()
            .into_iter()
            .map(|(name, type_name)| Topic { name, type_name })
            .collect::<Vec<Topic>>();

        Self {
            connection,
            state: ListWidgetState::new(topics, Some(0)),
            action: Action::Echo,
            needs_redraw: true,
        }
    }

    pub fn update(&mut self) {
        let mut new_topics = self
            .connection
            .borrow()
            .list_topics()
            .unwrap_or_default()
            .into_iter()
            .map(|(name, type_name)| Topic { name, type_name })
            .collect::<Vec<Topic>>();
        new_topics.sort_by(|a, b| a.name.cmp(&b.name));
        self.state.update(new_topics);
    }
}

impl TuiView for TopicListState {
    fn handle_event(&mut self, event: Event) -> Event {
        self.update();

        let event = self.state.handle_event(event);
        if let Event::Key(CrosstermEvent::Key(key_event)) = event {
            if key_event.kind != KeyEventKind::Press {
                return event;
            }
            match key_event.code {
                KeyCode::Char('l') | KeyCode::Right => {
                    self.action = self.action.next();
                    self.needs_redraw = true;
                    Event::None
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    self.action = self.action.previous();
                    self.needs_redraw = true;
                    Event::None
                }
                KeyCode::Enter => {
                    if let Some(topic) = self.state.get_selected() {
                        match self.action {
                            Action::Echo => Event::NewMessageView(NewTopicEvent {
                                topic: topic.name.clone(),
                                message_type: topic.type_name.clone(),
                            }),
                            Action::Pub => Event::NewPublisher(NewPublisherEvent {
                                topic: topic.name.clone(),
                                message_type: topic.type_name.clone(),
                            }),
                            Action::FrequencyPlot => Event::NewHz(NewHzEvent {
                                topic: topic.name.clone(),
                                view: None,
                            }),
                        }
                    } else {
                        event
                    }
                }
                _ => event,
            }
        } else {
            event
        }
    }

    fn name(&self) -> String {
        "Topics".to_string()
    }

    fn get_help_text(&self) -> String {
        "Topic List View Help:\n\
        Normal Mode:\n\
        - 'j' or ↓: Move down in the topic list.\n\
        - 'k' or ↑: Move up in the topic list.\n\
        - 'l' or →: Switch to the next action (Echo, Pub, Hz).\n\
        - 'h' or ←: Switch to the previous action (Echo, Pub, Hz).\n\
        - 'Enter': Execute the selected action on the highlighted topic.\n\
        Search Mode:\n\
        - Type to filter topics.\n\
        - 'Backspace': Remove the last character from the search filter.\n\
        - 'Esc'/'Enter': Exit search mode."
            .to_string()
    }

    fn needs_redraw(&mut self) -> bool {
        if (self.state.needs_redraw()) || self.needs_redraw {
            self.needs_redraw = false;
            true
        } else {
            false
        }
    }
}

impl TopicList {
    pub fn render(area: Rect, buf: &mut Buffer, state: &mut TopicListState) {
        let action_text = Line::from_iter([
            Span::raw(" Topic List - "),
            Span::styled(
                " Echo ",
                if matches!(state.action, Action::Echo) {
                    SELECTED_STYLE
                } else {
                    Style::default()
                },
            ),
            Span::styled(
                " Pub ",
                if matches!(state.action, Action::Pub) {
                    SELECTED_STYLE
                } else {
                    Style::default()
                },
            ),
            Span::styled(
                " Hz ",
                if matches!(state.action, Action::FrequencyPlot) {
                    SELECTED_STYLE
                } else {
                    Style::default()
                },
            ),
        ])
        .centered();

        let block = Block::bordered()
            .title(action_text)
            .border_style(HEADER_STYLE)
            .border_type(BorderType::Rounded);

        let topic_list_widget = ListWidget::new().block(block).auto_scroll(true);

        topic_list_widget.render(area, buf, &mut state.state);
    }
}
