use std::collections::HashMap;
use std::fmt::Display;

use enum_dispatch::enum_dispatch;

use ros2::ConnectionROS2;

use crate::common::generic_message::{GenericMessage, InterfaceType, MessageMetadata};

#[derive(Debug, Clone, PartialEq)]
pub enum Parameters {
    Bool(bool),
    Integer(i64),
    Double(f64),
    String(String),
    ByteArray(Vec<u8>),
    BoolArray(Vec<bool>),
    IntegerArray(Vec<i64>),
    DoubleArray(Vec<f64>),
    StringArray(Vec<String>),
}

impl Display for Parameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Parameters::Bool(v) => write!(f, "{}", v),
            Parameters::Integer(v) => write!(f, "{}", v),
            Parameters::Double(v) => write!(f, "{}", v),
            Parameters::String(v) => write!(f, "{}", v),
            Parameters::ByteArray(v) => write!(f, "{:?}", v),
            Parameters::BoolArray(v) => write!(f, "{:?}", v),
            Parameters::IntegerArray(v) => write!(f, "{:?}", v),
            Parameters::DoubleArray(v) => write!(f, "{:?}", v),
            Parameters::StringArray(v) => write!(f, "{:?}", v),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeName {
    pub namespace: String,
    pub name: String,
}

impl NodeName {
    pub fn new(namespace: &str, name: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            name: name.to_string(),
        }
    }

    pub fn full_name(&self) -> String {
        if self.namespace.ends_with('/') {
            format!("{}{}", self.namespace, self.name)
        } else {
            format!("{}/{}", self.namespace, self.name)
        }
    }
}

#[derive(Clone, Debug)]
pub struct NamedInterface {
    pub name: String,
    pub type_name: InterfaceType,
}

/// The publisher closure returns the (possibly empty) list of field-level conversion warnings on
/// success, or a fatal error string on failure. A non-empty warnings list means the message was
/// sent but some fields could not be converted and were left at their default values.
type PublisherFunc = dyn Fn(&GenericMessage) -> Result<Vec<String>, String>;

// Connection trait
#[enum_dispatch(ConnectionType)]
pub trait Connection {
    /// Get the name of the connection.
    fn name(&self) -> &str;

    /// Get the list of topics in the connection.
    fn list_topics(&self) -> Result<Vec<NamedInterface>, String>;

    /// Get the list of nodes in the connection.
    fn list_nodes(&self) -> Result<Vec<NodeName>, String>;

    /// Get the type of a specific topic.
    fn get_topic_type(&self, topic: &str) -> Option<InterfaceType>;

    fn subscribe(
        &mut self,
        topic: &str,
        callback: impl Fn(GenericMessage, MessageMetadata) + Send + Sync + 'static,
    ) -> Result<(), String>;

    fn create_publisher(
        &mut self,
        topic: &str,
        message_type: &InterfaceType,
    ) -> Result<Box<PublisherFunc>, String>;

    fn get_publisher_names_and_types_by_node(
        &self,
        node_name: &NodeName,
    ) -> Result<Vec<NamedInterface>, String>;

    fn get_subscription_names_and_types_by_node(
        &self,
        node_name: &NodeName,
    ) -> Result<Vec<NamedInterface>, String>;

    fn get_client_names_and_types_by_node(
        &self,
        node_name: &NodeName,
    ) -> Result<Vec<NamedInterface>, String>;

    fn get_service_names_and_types_by_node(
        &self,
        node_name: &NodeName,
    ) -> Result<Vec<NamedInterface>, String>;

    fn get_parameters_by_node(
        &self,
        node_name: &NodeName,
    ) -> Result<HashMap<String, Parameters>, String>;

    fn set_parameter_by_node(
        &mut self,
        node_name: &NodeName,
        parameter_name: &str,
        parameter: Parameters,
    ) -> Result<(), String>;

    /// Get the nodes that publish to the given topic.
    fn get_publishers_info_by_topic(&self, topic: &str) -> Result<Vec<NodeName>, String>;

    /// Get the nodes that subscribe to the given topic.
    fn get_subscriptions_info_by_topic(&self, topic: &str) -> Result<Vec<NodeName>, String>;

    /// List all services available in the connection.
    fn list_services(&self) -> Result<Vec<(String, InterfaceType)>, String>;

    /// Get the type of a specific service.
    fn get_service_type(&self, service_name: &str) -> Option<InterfaceType>;

    /// Return a default-initialized request GenericMessage for the given service type.
    /// The UI can use this to discover the request fields and build a form.
    fn get_service_request_template(
        &self,
        service_type: &InterfaceType,
    ) -> Result<GenericMessage, String>;

    /// Call a service. Sends `request`, blocks until the response arrives (or timeout), and
    /// returns the response together with any field-level conversion warnings that were produced
    /// while building the request. A non-empty warnings list means some request fields could not
    /// be converted and were sent as their default values.
    fn call_service(
        &self,
        service_name: &str,
        service_type: &InterfaceType,
        request: &GenericMessage,
    ) -> Result<(GenericMessage, Vec<String>), String>;
}

#[enum_dispatch]
#[derive(Debug)]
pub enum ConnectionType {
    // Mcap(ConnectionMcap),
    // Foxglove(ConnectionFoxglove),
    // Rosbridge(ConnectionRosbridge),
    ROS2(ConnectionROS2),
}

pub mod ros2;
