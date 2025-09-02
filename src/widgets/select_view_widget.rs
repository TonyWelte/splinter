use ratatui::{
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Clear, Widget},
};

use crate::common::style::SELECTED_STYLE;

pub struct SelectViewWidget<'a> {
    block: Option<Block<'a>>,
    views: &'a Vec<(usize, String)>,
    selected: Option<usize>,
    with_new_option: bool,
}

impl<'a> SelectViewWidget<'a> {
    pub fn new(views: &'a Vec<(usize, String)>) -> Self {
        Self {
            block: None,
            views,
            selected: None,
            with_new_option: false,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn with_selection(mut self, selected: usize) -> Self {
        self.selected = Some(selected);
        self
    }

    pub fn with_new_option(mut self, with_new_option: bool) -> Self {
        self.with_new_option = with_new_option;
        self
    }
}

impl<'a> Widget for SelectViewWidget<'a> {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let inner_area = if let Some(block) = self.block {
            let inner_area = block.inner(area);
            block.render(area, buf);
            inner_area
        } else {
            area
        };

        Clear.render(inner_area, buf);

        for (i, (id, name)) in self.views.iter().enumerate() {
            let y = inner_area.y + i as u16;
            if y >= inner_area.y + inner_area.height {
                break;
            }
            let style = if Some(i) == self.selected {
                SELECTED_STYLE
            } else {
                Style::default()
            };
            Line::from(format!("{}: {}", id, name)).style(style).render(
                Rect {
                    x: inner_area.x,
                    y,
                    width: inner_area.width,
                    height: 1,
                },
                buf,
            );
        }

        if self.with_new_option {
            let y = inner_area.y + self.views.len() as u16;
            if y < inner_area.y + inner_area.height {
                let style = if Some(self.views.len()) == self.selected {
                    SELECTED_STYLE
                } else {
                    Style::default()
                };
                Line::from("<New View>").style(style).render(
                    Rect {
                        x: inner_area.x,
                        y,
                        width: inner_area.width,
                        height: 1,
                    },
                    buf,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;

    #[test]
    fn test_select_view_widget() {
        let views = vec![
            (1, "View1".to_string()),
            (2, "View2".to_string()),
            (3, "View3".to_string()),
        ];
        let selected = 0 as usize;
        let widget = SelectViewWidget::new(&views).with_selection(selected);
        let area = Rect {
            x: 0,
            y: 0,
            width: 20,
            height: 5,
        };
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let expected = Buffer::with_lines(vec![
            "1: View1            ",
            "2: View2            ",
            "3: View3            ",
            "                    ",
            "                    ",
        ]);
        buf.set_style(area, Style::reset());

        assert_eq!(buf, expected);
    }

    #[test]
    fn test_select_view_widget_with_selection() {
        let views = vec![
            (1, "View1".to_string()),
            (2, "View2".to_string()),
            (3, "View3".to_string()),
        ];
        let selected = 1 as usize;
        let widget = SelectViewWidget::new(&views).with_selection(selected);
        let area = Rect {
            x: 0,
            y: 0,
            width: 20,
            height: 5,
        };
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);
        let mut expected = Buffer::with_lines(vec![
            "1: View1            ",
            "2: View2            ",
            "3: View3            ",
            "                    ",
            "                    ",
        ]);
        expected.set_style(
            Rect {
                x: 0,
                y: 1,
                width: 20,
                height: 1,
            },
            SELECTED_STYLE,
        );
        assert_eq!(buf, expected);
    }
}
