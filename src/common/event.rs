use ratatui::crossterm::event::Event as CrosstermEvent;
use rclrs::MessageTypeName;

#[derive(Debug, Clone)]
pub(crate) struct NewGraphLineEvent {
    pub topic: String,
    pub field: Vec<usize>,
    pub view: Option<usize>,
}

#[derive(Debug, Clone)]
pub(crate) struct NewTopicEvent {
    pub topic: String,
    pub message_type: MessageTypeName,
}

#[derive(Debug, Clone)]
pub(crate) enum Event {
    None,
    Key(CrosstermEvent),
    NewGraphLine(NewGraphLineEvent),
    NewTopic(NewTopicEvent),
    NewGraph(NewGraphLineEvent),
    NewHzPlot(NewTopicEvent),
}
