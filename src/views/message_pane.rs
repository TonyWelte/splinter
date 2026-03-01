use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    common::{
        generic_message::{AnyTypeMutableRef, GenericMessage},
        generic_message_selector::GenericMessageSelector,
    },
    widgets::message_widget::MessageWidgetState,
};

/// Reusable cursor/selection state for navigating a `GenericMessage`.
///
/// Encapsulates the `selected_fields` path and `MessageWidgetState` that
/// are otherwise duplicated across raw_message, service_call, and
/// topic_publisher views.
pub struct MessagePaneState {
    pub selected_fields: Vec<usize>,
    pub widget_state: MessageWidgetState,
}

impl MessagePaneState {
    pub fn new() -> Self {
        Self {
            selected_fields: Vec::new(),
            widget_state: MessageWidgetState::new(true),
        }
    }

    /// Move selection down (next visible field).
    pub fn select_down(&mut self, message: &GenericMessage) {
        self.selected_fields = GenericMessageSelector::new(message).down(&self.selected_fields);
    }

    /// Move selection up (previous visible field).
    pub fn select_up(&mut self, message: &GenericMessage) {
        self.selected_fields = GenericMessageSelector::new(message).up(&self.selected_fields);
    }

    /// Jump to the next sibling at the current nesting level.
    pub fn select_far_down(&mut self, message: &GenericMessage) {
        if self.selected_fields.is_empty() {
            self.selected_fields.push(0);
        }
        *self.selected_fields.last_mut().unwrap() += 1;
        if message.get_field_type(&self.selected_fields).is_err() {
            *self.selected_fields.last_mut().unwrap() -= 1;
        }
    }

    /// Jump to the first sibling at the current nesting level, or pop up one level.
    pub fn select_far_up(&mut self) {
        if let Some(last) = self.selected_fields.last() {
            if *last == 0 {
                self.selected_fields.pop();
            } else {
                self.selected_fields.pop();
                self.selected_fields.push(0);
            }
        }
    }

    /// Collapse / move left in the message tree.
    pub fn select_left(&mut self, message: &GenericMessage) {
        self.selected_fields = GenericMessageSelector::new(message).left(&self.selected_fields);
    }

    /// Expand / move right in the message tree.
    pub fn select_right(&mut self, message: &GenericMessage) {
        self.selected_fields = GenericMessageSelector::new(message).right(&self.selected_fields);
    }

    /// Jump to the very last field in the message.
    pub fn select_last(&mut self, message: &GenericMessage) {
        self.selected_fields = GenericMessageSelector::new(message).last_field_path();
    }

    /// Handle common read-only navigation keys (j/k/G with Shift variants).
    /// Returns `true` if the key was consumed.
    pub fn handle_nav_key(&mut self, key_event: KeyEvent, message: &GenericMessage) -> bool {
        match key_event.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.select_far_down(message);
                } else {
                    self.select_down(message);
                }
                true
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.select_far_up();
                } else {
                    self.select_up(message);
                }
                true
            }
            KeyCode::Char('G') => {
                self.select_last(message);
                true
            }
            _ => false,
        }
    }
}

/// Parse `field_content` and write it into the field at `selected_fields`
/// inside `message`. Shared by topic_publisher and service_call views.
pub fn commit_field_edit(
    message: &mut GenericMessage,
    selected_fields: &[usize],
    field_content: &str,
) -> Result<(), String> {
    let value = message.get_mut_deep_index(selected_fields)?;
    match value {
        AnyTypeMutableRef::Float(v) => {
            *v = field_content
                .parse::<f32>()
                .map_err(|e| format!("Failed to parse float: {e}"))?;
            Ok(())
        }
        AnyTypeMutableRef::Double(v) => {
            *v = field_content
                .parse::<f64>()
                .map_err(|e| format!("Failed to parse double: {e}"))?;
            Ok(())
        }
        AnyTypeMutableRef::Boolean(v) => {
            *v = field_content
                .parse::<bool>()
                .map_err(|e| format!("Failed to parse boolean: {e}"))?;
            Ok(())
        }
        AnyTypeMutableRef::Uint8(v) => {
            *v = field_content
                .parse::<u8>()
                .map_err(|e| format!("Failed to parse uint8: {e}"))?;
            Ok(())
        }
        AnyTypeMutableRef::Int8(v) => {
            *v = field_content
                .parse::<i8>()
                .map_err(|e| format!("Failed to parse int8: {e}"))?;
            Ok(())
        }
        AnyTypeMutableRef::Uint16(v) => {
            *v = field_content
                .parse::<u16>()
                .map_err(|e| format!("Failed to parse uint16: {e}"))?;
            Ok(())
        }
        AnyTypeMutableRef::Int16(v) => {
            *v = field_content
                .parse::<i16>()
                .map_err(|e| format!("Failed to parse int16: {e}"))?;
            Ok(())
        }
        AnyTypeMutableRef::Uint32(v) => {
            *v = field_content
                .parse::<u32>()
                .map_err(|e| format!("Failed to parse uint32: {e}"))?;
            Ok(())
        }
        AnyTypeMutableRef::Int32(v) => {
            *v = field_content
                .parse::<i32>()
                .map_err(|e| format!("Failed to parse int32: {e}"))?;
            Ok(())
        }
        AnyTypeMutableRef::Uint64(v) => {
            *v = field_content
                .parse::<u64>()
                .map_err(|e| format!("Failed to parse uint64: {e}"))?;
            Ok(())
        }
        AnyTypeMutableRef::Int64(v) => {
            *v = field_content
                .parse::<i64>()
                .map_err(|e| format!("Failed to parse int64: {e}"))?;
            Ok(())
        }
        AnyTypeMutableRef::String(v) => {
            *v = field_content.to_string();
            Ok(())
        }
        AnyTypeMutableRef::Array(_)
        | AnyTypeMutableRef::Sequence(_)
        | AnyTypeMutableRef::BoundedSequence(_) => {
            Err("Cannot edit non-primitive field".to_string())
        }
    }
}
