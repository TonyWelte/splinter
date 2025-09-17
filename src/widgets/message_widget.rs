use ratatui::{
    prelude::{BlockExt, Buffer, Rect},
    widgets::{Block, StatefulWidget, Widget},
};

use crate::common::generic_message::GenericMessage;

mod array_widget;
mod bounded_sequence_widget;
mod sequence_widget;
mod value_widget;

use array_widget::ArrayWidget;
use bounded_sequence_widget::BoundedSequenceWidget;
use sequence_widget::SequenceWidget;
use value_widget::ValueWidget;

pub struct MessageWidget<'a> {
    message: &'a GenericMessage,
    selection: Option<&'a [usize]>,
    edit: Option<&'a str>,

    block: Option<Block<'a>>,
}

pub struct MessageWidgetState {
    scroll_offset: u16,
    auto_scroll: bool,
}

impl Default for MessageWidgetState {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageWidgetState {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
            auto_scroll: true,
        }
    }

    pub fn auto_scroll(mut self) -> Self {
        self.auto_scroll = true;
        self
    }
}

impl<'a> MessageWidget<'a> {
    pub fn new(message: &'a GenericMessage) -> Self {
        Self {
            message,
            selection: None,
            edit: None,
            block: None,
        }
    }

    pub fn with_selection(mut self, selection: &'a [usize]) -> Self {
        self.selection = Some(selection);
        self
    }

    pub fn with_edit(mut self, edit: &'a str) -> Self {
        self.edit = Some(edit);
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn height(&self, width: u16) -> u16 {
        // TODO: Don't like having the calculation of the height of the displayed message separated from the rendering logic.
        let mut height = 0;
        for (name, value) in self.message.iter() {
            height += ValueWidget::new(name, value).height(width);
        }
        height
    }

    pub fn selection_height(&self, width: u16) -> u16 {
        if let Some(selection) = self.selection {
            if selection.is_empty() {
                return 0;
            }

            let mut height = 0;
            for (i, (name, value)) in self.message.iter().enumerate() {
                if i != selection[0] {
                    height += ValueWidget::new(name, value).height(width);
                } else {
                    height += ValueWidget::new(name, value)
                        .with_selection(&selection[1..])
                        .selection_height(width);
                    break;
                }
            }
            return height;
        }
        0
    }
}

impl<'a> Widget for MessageWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = MessageWidgetState::new();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

impl<'a> StatefulWidget for MessageWidget<'a> {
    type State = MessageWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.block.as_ref().render(area, buf);
        let area_remaining = self.block.inner_if_some(area);

        // At the end otherwise would affect stateless rendering
        if state.auto_scroll {
            let selection_height = self.selection_height(area_remaining.width);
            state.scroll_offset = selection_height
                .saturating_sub(area_remaining.height / 2)
                .min(
                    self.height(area_remaining.width)
                        .saturating_sub(area_remaining.height),
                );
        }

        let mut extended_area = Rect {
            x: 0,
            y: 0,
            width: area_remaining.width,
            height: area_remaining.height + state.scroll_offset,
        };

        let mut extended_buffer = Buffer::empty(extended_area);

        for (i, (name, value)) in self.message.iter().enumerate() {
            let mut value_widget = ValueWidget::new(name, value);
            if let Some(selection) = self.selection {
                if !selection.is_empty() && selection[0] == i {
                    value_widget = value_widget.with_selection(&selection[1..]);
                    if let Some(edit) = self.edit {
                        value_widget = value_widget.with_edit(edit);
                    }
                }
            }

            let widget_height = value_widget
                .height(extended_area.width)
                .min(extended_area.height);
            value_widget.render(extended_area, &mut extended_buffer);

            // Move the area down for the next field
            extended_area.y += widget_height;
            extended_area.height -= widget_height;
        }

        // Copy the visible part of the extended buffer to the actual buffer
        for y in 0..area_remaining.height {
            for x in 0..area_remaining.width {
                if let Some(cell) = buf.cell_mut((area_remaining.x + x, area_remaining.y + y)) {
                    *cell = extended_buffer
                        .cell((x, y + state.scroll_offset))
                        .unwrap()
                        .clone();
                }
            }
        }
    }
}

trait AsStrVec {
    fn as_str_iter(&self) -> Vec<String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use rclrs::{MessageTypeName, dynamic_message::DynamicMessage};

    #[test]
    fn test_message_widget_selection_height() {
        let message_type = MessageTypeName {
            package_name: "nav_msgs".to_owned(),
            type_name: "Odometry".to_owned(),
        };
        let msg = DynamicMessage::new(message_type).unwrap();
        let generic_message = GenericMessage::from(msg.view());

        let widget = MessageWidget::new(&generic_message);
        assert_eq!(widget.selection_height(50), 0); // No selection

        let widget = MessageWidget::new(&generic_message).with_selection(&[0]);
        assert_eq!(widget.selection_height(50), 1); // Select header field

        let widget = MessageWidget::new(&generic_message).with_selection(&[0, 0]);
        assert_eq!(widget.selection_height(50), 2); // Select header.stamp field

        let widget = MessageWidget::new(&generic_message).with_selection(&[2, 0, 1, 1]);
        assert_eq!(widget.selection_height(50), 15); // Select pose.pose.orientation.y field

        let widget = MessageWidget::new(&generic_message).with_selection(&[2, 1, 5]);
        assert_eq!(widget.selection_height(50), 27); // Select pose.covariance.5 field

        let widget = MessageWidget::new(&generic_message).with_selection(&[2, 1, 35]);
        assert_eq!(widget.selection_height(50), 27); // Select pose.covariance.35 field
    }
}
