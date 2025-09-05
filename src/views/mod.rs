use enum_dispatch::enum_dispatch;

use hz_plot::HzPlotState;
use live_plot::LivePlotState;
use node_list::NodeListState;
use raw_message::RawMessageState;
use topic_list::TopicListState;
use topic_publisher::TopicPublisherState;

use crate::common::event::Event;

pub mod hz_plot;
pub mod live_plot;
pub mod node_list;
pub mod raw_message;
pub mod topic_list;
pub mod topic_publisher;

#[enum_dispatch]
pub enum Views {
    TopicList(TopicListState),
    RawMessage(RawMessageState),
    LivePlot(LivePlotState),
    HzPlot(HzPlotState),
    NodeList(NodeListState),
    TopicPublisher(TopicPublisherState),
}

#[enum_dispatch(Views)]
pub trait TuiView {
    fn handle_event(&mut self, event: Event) -> Event;

    fn name(&self) -> String;

    fn get_help_text(&self) -> String;
}
