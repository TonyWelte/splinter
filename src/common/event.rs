use ratatui::crossterm::event::Event as CrosstermEvent;

use crate::common::generic_message::InterfaceType;

#[derive(Debug, Clone)]
pub struct NewLineEvent {
    pub topic: String,
    pub field: Vec<usize>,
    pub view: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct NewTopicEvent {
    pub topic: String,
    pub message_type: InterfaceType,
}

#[derive(Debug, Clone)]
pub struct NewHzEvent {
    pub topic: String,
    pub view: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct NewPublisherEvent {
    pub topic: String,
    pub message_type: InterfaceType,
}

#[derive(Debug, Clone)]
pub enum Event {
    None,
    Key(CrosstermEvent),
    NewMessageView(NewTopicEvent),
    NewLine(NewLineEvent),
    NewLinePlot(NewLineEvent),
    NewHz(NewHzEvent),
    NewHzPlot(NewHzEvent),
    NewPublisher(NewPublisherEvent),
    Error(String),
    ClosePopup,
}
