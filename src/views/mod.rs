use std::{cell::RefCell, fmt::Debug, rc::Rc};

use ratatui::{buffer::Buffer, layout::Rect};

use crate::{
    common::{event::Event, generic_message::InterfaceType},
    connections::{ConnectionType, NodeName},
};

pub mod hz_plot;
pub mod live_plot;
pub mod node_details;
pub mod node_list;
pub mod raw_message;
pub mod topic_list;
pub mod topic_publisher;

pub trait TuiView {
    fn handle_event(&mut self, event: Event) -> Event;

    fn name(&self) -> String;

    fn get_help_text(&self) -> String;

    // Return true if the view needs to be redrawn since the last call
    fn needs_redraw(&mut self) -> bool;

    fn render(&mut self, area: Rect, buf: &mut Buffer);

    fn as_topic_acceptor(&mut self) -> Option<&mut dyn AcceptsTopic> {
        None
    }

    fn as_field_acceptor(&mut self) -> Option<&mut dyn AcceptsField> {
        None
    }

    fn as_node_acceptor(&mut self) -> Option<&mut dyn AcceptsNode> {
        None
    }
}

impl Debug for dyn TuiView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TuiView")
    }
}

// Information about a connection
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub connection: Rc<RefCell<ConnectionType>>,
}

pub trait FromConnection: TuiView {
    fn from_connection(connection_info: ConnectionInfo) -> Self;
}

// Information about a node
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub connection: Rc<RefCell<ConnectionType>>,
    pub node_name: NodeName,
}

pub trait FromNode: TuiView {
    fn from_node(node_info: NodeInfo) -> Self;
}

// Information about a topic
#[derive(Debug, Clone)]
pub struct TopicInfo {
    pub connection: Rc<RefCell<ConnectionType>>,
    pub topic: String,
    pub type_name: InterfaceType,
}

pub trait FromTopic: TuiView {
    fn from_topic(topic_info: TopicInfo) -> Self;
}

pub trait AcceptsTopic {
    fn accepts_topic(&mut self, topic_info: TopicInfo);
}

pub trait AcceptsNode {
    fn accepts_node(&mut self, node_info: NodeInfo);
}

#[derive(Debug, Clone)]
enum FieldInfoType {
    Float,   // e.g., Float32, Float64
    Integer, // e.g., Boolean, Int8, Uint16, Int32, etc.
    String,
    Message(InterfaceType),
}

// Information about a field
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub connection: Rc<RefCell<ConnectionType>>,
    pub topic: String,
    pub type_name: InterfaceType,
    pub field: Vec<usize>,
    pub field_name: String,
    pub field_type: FieldInfoType,
}

pub trait FromField: TuiView {
    fn from_field(field_info: FieldInfo) -> Self;
}

pub trait AcceptsField {
    fn accepts_field(&mut self, field_info: FieldInfo);
}
