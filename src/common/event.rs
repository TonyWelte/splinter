use std::{cell::RefCell, rc::Rc};

use ratatui::crossterm::event::Event as CrosstermEvent;

use crate::views::{ConnectionInfo, FieldInfo, NodeInfo, TopicInfo, TuiView};

#[derive(Debug, Clone)]
pub enum Event {
    None,
    Key(CrosstermEvent),
    NewConnection(ConnectionInfo),
    NewNode(NodeInfo),
    NewTopic(TopicInfo),
    NewField(FieldInfo),
    Error(String),
    ClosePopup,
    NewView(Rc<RefCell<dyn TuiView>>),
}
