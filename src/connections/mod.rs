use std::collections::HashMap;

use enum_dispatch::enum_dispatch;

use rclrs::NodeNameInfo;
use ros2::ConnectionROS2;

use crate::common::generic_message::{GenericMessage, InterfaceType, MessageMetadata};

use rclrs::MessageTypeName;

type PublisherFunc = dyn Fn(&GenericMessage);

// Connection trait
#[enum_dispatch(ConnectionType)]
pub trait Connection {
    /// Get the name of the connection.
    fn name(&self) -> &str;

    /// Get the list of topics in the connection.
    fn list_topics(&self) -> Vec<(String, InterfaceType)>;

    /// Get the list of nodes in the connection.
    fn list_nodes(&self) -> Vec<NodeNameInfo>;

    /// Get the type of a specific topic.
    fn get_topic_type(&self, topic: &str) -> Option<MessageTypeName>;

    fn subscribe(
        &mut self,
        topic: &str,
        callback: impl Fn(GenericMessage, MessageMetadata) + Send + Sync + 'static,
    ) -> Result<(), String>;

    fn create_publisher(
        &mut self,
        topic: &str,
        message_type: &MessageTypeName,
    ) -> Result<Box<PublisherFunc>, String>;

    fn get_publisher_names_and_types_by_node(
        &self,
        node_name: &NodeNameInfo,
    ) -> Result<HashMap<String, Vec<String>>, String>;

    fn get_subscription_names_and_types_by_node(
        &self,
        node_name: &NodeNameInfo,
    ) -> Result<HashMap<String, Vec<String>>, String>;

    fn get_client_names_and_types_by_node(
        &self,
        node_name: &NodeNameInfo,
    ) -> Result<HashMap<String, Vec<String>>, String>;

    fn get_service_names_and_types_by_node(
        &self,
        node_name: &NodeNameInfo,
    ) -> Result<HashMap<String, Vec<String>>, String>;
}

#[enum_dispatch]
pub enum ConnectionType {
    // Mcap(ConnectionMcap),
    ROS2(ConnectionROS2),
}

pub mod ros2;
