use ratatui::crossterm::event::Event as CrosstermEvent;
use rclrs::MessageTypeName;

#[derive(Debug, Clone)]
pub(crate) struct NewLineEvent {
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
pub(crate) struct NewHzEvent {
    pub topic: String,
    pub view: Option<usize>,
}

#[derive(Debug, Clone)]
pub(crate) struct NewPublisherEvent {
    pub topic: String,
}

#[derive(Debug, Clone)]
pub(crate) enum Event {
    None,
    Key(CrosstermEvent),
    NewMessageView(NewTopicEvent),
    NewLine(NewLineEvent),
    NewLinePlot(NewLineEvent),
    NewHz(NewHzEvent),
    NewHzPlot(NewHzEvent),
    NewPublisher(NewPublisherEvent),
}
