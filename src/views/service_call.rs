use std::{cell::RefCell, rc::Rc};

use ratatui::{
    layout::{Constraint, Layout, Spacing},
    prelude::{Buffer, Rect},
    symbols::{border, line},
    text::Line,
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEventKind};

use crate::{
    common::{
        event::Event,
        generic_message::{
            AnyTypeMutableRef, BoundedSequenceField, GenericMessage, InterfaceType, Length,
            SequenceField,
        },
        generic_message_selector::{get_field_category, FieldCategory},
        style::HEADER_STYLE,
    },
    connections::{Connection, ConnectionType},
    views::{
        message_pane::{commit_field_edit, MessagePaneState},
        TuiView,
    },
    widgets::message_widget::MessageWidget,
};

/// Which half of the split view has focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusPane {
    Request,
    Response,
}

/// Left panel: rounded outer corners, T-junctions where it meets the right panel.
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

/// Right panel: T-junctions where it meets the left panel, rounded outer corners.
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

pub struct ServiceCallState {
    service_name: String,
    service_type: InterfaceType,
    connection: Rc<RefCell<ConnectionType>>,

    // ── Request side (editable, like TopicPublisher) ──────────────
    request: GenericMessage,
    request_pane: MessagePaneState,
    is_editing: bool,
    field_content: String,

    // ── Response side (read-only, like RawMessage) ────────────────
    response: Option<GenericMessage>,
    response_pane: MessagePaneState,
    response_error: Option<String>,

    focus: FocusPane,
    needs_redraw: bool,
}

impl ServiceCallState {
    pub fn new(
        service_name: String,
        service_type: InterfaceType,
        connection: Rc<RefCell<ConnectionType>>,
    ) -> Self {
        let request = connection
            .borrow()
            .get_service_request_template(&service_type)
            .expect("Failed to get service request template");

        Self {
            service_name,
            service_type,
            connection,
            request,
            request_pane: MessagePaneState::new(),
            is_editing: false,
            field_content: String::new(),
            response: None,
            response_pane: MessagePaneState::new(),
            response_error: None,
            focus: FocusPane::Request,
            needs_redraw: true,
        }
    }

    fn commit_edit(&mut self) -> Result<(), String> {
        self.needs_redraw = true;
        commit_field_edit(
            &mut self.request,
            &self.request_pane.selected_fields,
            &self.field_content,
        )
    }

    fn call_service(&mut self) {
        self.needs_redraw = true;
        match self.connection.borrow().call_service(
            &self.service_name,
            &self.service_type,
            &self.request,
        ) {
            Ok(response) => {
                self.response = Some(response);
                self.response_error = None;
                self.response_pane.selected_fields.clear();
            }
            Err(e) => {
                self.response = None;
                self.response_error = Some(e);
            }
        }
    }

    // ── Event handling ───────────────────────────────────────────

    fn handle_request_event(&mut self, key_event: crossterm::event::KeyEvent) -> Event {
        match key_event.code {
            KeyCode::Char('c') if !self.is_editing => {
                self.call_service();
                Event::None
            }
            KeyCode::Tab if !self.is_editing => {
                self.focus = FocusPane::Response;
                self.needs_redraw = true;
                Event::None
            }
            KeyCode::Char('j')
            | KeyCode::Down
            | KeyCode::Char('k')
            | KeyCode::Up
            | KeyCode::Char('G')
                if !self.is_editing =>
            {
                if self.request_pane.handle_nav_key(key_event, &self.request) {
                    self.needs_redraw = true;
                }
                Event::None
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.needs_redraw = true;
                if self.is_editing {
                    self.field_content.push('h');
                    Event::None
                } else {
                    if let Ok(field) = self
                        .request
                        .get_mut_deep_index(&self.request_pane.selected_fields)
                    {
                        match field {
                            AnyTypeMutableRef::Sequence(SequenceField::Message(_))
                            | AnyTypeMutableRef::BoundedSequence(BoundedSequenceField::Message(
                                _,
                                _,
                            )) => {
                                return Event::Error(
                                    "Cannot resize sequence of messages".to_string(),
                                );
                            }
                            AnyTypeMutableRef::Sequence(seq) => {
                                seq.resize(seq.len().saturating_sub(1));
                            }
                            AnyTypeMutableRef::BoundedSequence(seq) => {
                                seq.resize(seq.len().saturating_sub(1));
                            }
                            _ => {}
                        }
                    }
                    Event::None
                }
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.needs_redraw = true;
                if self.is_editing {
                    self.field_content.push('l');
                    Event::None
                } else {
                    if let Ok(field) = self
                        .request
                        .get_mut_deep_index(&self.request_pane.selected_fields)
                    {
                        match field {
                            AnyTypeMutableRef::Sequence(SequenceField::Message(_))
                            | AnyTypeMutableRef::BoundedSequence(BoundedSequenceField::Message(
                                _,
                                _,
                            )) => {
                                return Event::Error(
                                    "Cannot resize sequence of messages".to_string(),
                                );
                            }
                            AnyTypeMutableRef::Sequence(seq) => {
                                seq.resize(seq.len() + 1);
                            }
                            AnyTypeMutableRef::BoundedSequence(seq) => {
                                seq.resize(seq.len() + 1);
                            }
                            _ => {}
                        }
                    }
                    Event::None
                }
            }
            KeyCode::Backspace => {
                if self.is_editing {
                    self.needs_redraw = true;
                    self.field_content.pop();
                    Event::None
                } else {
                    Event::Key(CrosstermEvent::Key(key_event))
                }
            }
            KeyCode::Enter => {
                if self.is_editing {
                    self.is_editing = false;
                    self.commit_edit().unwrap_or_else(|e| {
                        eprintln!("Failed to commit edit: {e}");
                    });
                    self.field_content.clear();
                    self.needs_redraw = true;
                    Event::None
                } else if get_field_category(&self.request, &self.request_pane.selected_fields)
                    == Some(FieldCategory::Base)
                {
                    self.is_editing = true;
                    self.field_content.clear();
                    self.needs_redraw = true;
                    Event::None
                } else {
                    Event::Key(CrosstermEvent::Key(key_event))
                }
            }
            KeyCode::Char(c) => {
                if self.is_editing {
                    self.field_content.push(c);
                    self.needs_redraw = true;
                    Event::None
                } else {
                    Event::Key(CrosstermEvent::Key(key_event))
                }
            }
            _ => Event::Key(CrosstermEvent::Key(key_event)),
        }
    }

    fn handle_response_event(&mut self, key_event: crossterm::event::KeyEvent) -> Event {
        match key_event.code {
            KeyCode::Char('c') => {
                self.call_service();
                Event::None
            }
            KeyCode::BackTab => {
                self.focus = FocusPane::Request;
                self.needs_redraw = true;
                Event::None
            }
            _ => {
                if let Some(msg) = &self.response {
                    if self.response_pane.handle_nav_key(key_event, msg) {
                        self.needs_redraw = true;
                        return Event::None;
                    }
                }
                Event::Key(CrosstermEvent::Key(key_event))
            }
        }
    }
}

impl TuiView for ServiceCallState {
    fn handle_event(&mut self, event: Event) -> Event {
        match event {
            Event::Key(CrosstermEvent::Key(key_event)) => {
                if key_event.kind != KeyEventKind::Press {
                    return event;
                }
                match self.focus {
                    FocusPane::Request => self.handle_request_event(key_event),
                    FocusPane::Response => self.handle_response_event(key_event),
                }
            }
            other => other,
        }
    }

    fn name(&self) -> String {
        format!("Service Call - {}", self.service_name)
    }

    fn get_help_text(&self) -> String {
        "Service Call View Help:\n\
        - 'c': Call the service with the current request.\n\
        - 'Tab': Switch focus to the response pane.\n\
        - 'Shift+Tab': Switch focus to the request pane.\n\
        - 'j' or ↓: Move down in the message fields.\n\
        - 'k' or ↑: Move up in the message fields.\n\
        - 'G': Jump to the last field.\n\
        - 'l' or →: Increase size of sequence field (request only).\n\
        - 'h' or ←: Decrease size of sequence field (request only).\n\
        - 'Enter': Toggle edit mode for primitive fields (request only).\n\
        - 'Backspace': Remove last character when editing."
            .to_string()
    }

    fn needs_redraw(&mut self) -> bool {
        if self.needs_redraw {
            self.needs_redraw = false;
            return true;
        }
        false
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let [request_area, response_area] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .spacing(Spacing::Overlap(1))
                .areas(area);

        // ── Request pane ─────────────────────────────────────────
        let request_border_style = if self.focus == FocusPane::Request {
            HEADER_STYLE
        } else {
            ratatui::style::Style::default()
        };
        let request_block = Block::bordered()
            .title(Line::from(" Request (c to call) ").centered())
            .border_style(request_border_style)
            .border_set(LEFT_BORDER_SET);

        let mut request_widget = MessageWidget::new(&self.request).block(request_block);
        if !self.request_pane.selected_fields.is_empty() {
            request_widget = request_widget.with_selection(&self.request_pane.selected_fields);
            if self.is_editing {
                request_widget = request_widget.with_edit(&self.field_content);
            }
        }
        StatefulWidget::render(
            request_widget,
            request_area,
            buf,
            &mut self.request_pane.widget_state,
        );

        // ── Response pane ────────────────────────────────────────
        let response_border_style = if self.focus == FocusPane::Response {
            HEADER_STYLE
        } else {
            ratatui::style::Style::default()
        };
        let response_block = Block::bordered()
            .title(Line::from(" Response ").centered())
            .border_style(response_border_style)
            .border_set(RIGHT_BORDER_SET);

        if let Some(response) = &self.response {
            let response_widget = MessageWidget::new(response)
                .with_selection(&self.response_pane.selected_fields)
                .block(response_block);
            StatefulWidget::render(
                response_widget,
                response_area,
                buf,
                &mut self.response_pane.widget_state,
            );
        } else {
            let text = if let Some(err) = &self.response_error {
                format!("Error: {err}")
            } else {
                "No response yet. Press 'c' to call.".to_string()
            };
            let paragraph = Paragraph::new(text).block(response_block);
            Widget::render(paragraph, response_area, buf);
        }
    }
}
