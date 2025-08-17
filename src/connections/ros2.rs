use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::{spawn, JoinHandle};
use std::time::SystemTime;

use crate::common::generic_message::{GenericMessage, MessageMetadata};
use crate::connections::Connection;

use rclrs::MessageTypeName;
use rclrs::*;

pub struct ConnectionROS2 {
    // Fields for the ROS2 connection
    node: Node,
    subscriptions: Vec<Arc<DynamicSubscriptionState<Arc<NodeState>>>>,
    thread: JoinHandle<()>,
}

impl ConnectionROS2 {
    /// Create a new ROS2 connection.
    pub fn new() -> Self {
        let context = Context::default_from_env().unwrap();
        let mut executor = context.create_basic_executor();
        let node = executor
            .create_node("ros2tui".start_parameter_services(false))
            .unwrap();
        ConnectionROS2 {
            node,
            subscriptions: Vec::new(),
            thread: spawn(move || {
                let result = executor.spin(SpinOptions::default()).first_error();
                if let Err(e) = result {
                    eprintln!("Error in ROS2 executor: {}", e);
                }
            }),
        }
    }
}

impl Connection for ConnectionROS2 {
    /// Get the name of the connection.
    fn name(&self) -> &str {
        "ROS2 Connection"
    }

    /// Get the type of the connection.
    fn list_topics(&self) -> Vec<(String, MessageTypeName)> {
        self.node
            .get_topic_names_and_types()
            .unwrap()
            .into_iter()
            .map(|(name, types)| {
                let first_type = types
                    .first()
                    .map(|type_name| {
                        let parts: Vec<&str> = type_name.split('/').collect();
                        MessageTypeName {
                            package_name: parts.get(0).unwrap_or(&"").to_string(),
                            type_name: parts.get(2).unwrap_or(&"").to_string(),
                        }
                    })
                    .unwrap();
                (name, first_type)
            })
            .collect()
    }

    fn list_nodes(&self) -> Vec<NodeNameInfo> {
        self.node
            .get_node_names()
            .expect("Failed to get node names")
    }

    /// Get the type of a specific topic.
    fn get_topic_type(&self, topic: &str) -> Option<MessageTypeName> {
        self.node
            .get_topic_names_and_types()
            .unwrap()
            .into_iter()
            .find(|(name, _types)| name == topic)
            .and_then(|(_name, types)| {
                types.first().map(|type_name| {
                    let parts: Vec<&str> = type_name.split('/').collect();
                    MessageTypeName {
                        package_name: parts.get(0).unwrap_or(&"").to_string(),
                        type_name: parts.get(2).unwrap_or(&"").to_string(),
                    }
                })
            })
    }

    fn get_publisher_names_and_types_by_node(
        &self,
        node_name: &NodeNameInfo,
    ) -> Result<HashMap<String, Vec<String>>, String> {
        self.node
            .get_publisher_names_and_types_by_node(&node_name.name, &node_name.namespace)
            .map_err(|_| {
                format!("Failed to get publishers for node: {}", node_name.name).to_string()
            })
    }

    fn get_subscription_names_and_types_by_node(
        &self,
        node_name: &NodeNameInfo,
    ) -> Result<HashMap<String, Vec<String>>, String> {
        self.node
            .get_subscription_names_and_types_by_node(&node_name.name, &node_name.namespace)
            .map_err(|_| {
                format!("Failed to get subscriptions for node: {}", node_name.name).to_string()
            })
    }

    fn get_client_names_and_types_by_node(
        &self,
        node_name: &NodeNameInfo,
    ) -> Result<HashMap<String, Vec<String>>, String> {
        self.node
            .get_client_names_and_types_by_node(&node_name.name, &node_name.namespace)
            .map_err(|_| format!("Failed to get clients for node: {}", node_name.name).to_string())
    }

    fn get_service_names_and_types_by_node(
        &self,
        node_name: &NodeNameInfo,
    ) -> Result<HashMap<String, Vec<String>>, String> {
        self.node
            .get_service_names_and_types_by_node(&node_name.name, &node_name.namespace)
            .map_err(|_| format!("Failed to get services for node: {}", node_name.name).to_string())
    }

    fn subscribe(
        &mut self,
        topic: &str,
        callback: impl Fn(GenericMessage, MessageMetadata) + Send + Sync + 'static,
    ) -> Result<(), String> {
        let topic_type = self.get_topic_type(topic).unwrap();
        let message = Arc::new(Mutex::new(DynamicMessage::new(topic_type.clone()).unwrap()));
        let message_copy = message.clone();
        let subscription = self
            .node
            .create_dynamic_subscription(
                topic_type,
                topic,
                move |msg: DynamicMessage, msg_info: MessageInfo| {
                    let metadata = MessageMetadata {
                        received_time: msg_info.received_timestamp.unwrap_or(SystemTime::now()),
                    };
                    let generic_message = GenericMessage::from(msg.view());
                    callback(generic_message, metadata);
                },
            )
            .unwrap();
        self.subscriptions.push(subscription);

        Ok(())
    }

    fn create_publisher(
        &mut self,
        topic: &str,
        message_type: &MessageTypeName,
    ) -> Result<Arc<DynamicPublisherState>, String> {
        let publisher = self
            .node
            .create_dynamic_publisher(message_type.clone(), topic)
            .map_err(|e| format!("Failed to create publisher: {}", e))?;
        Ok(publisher)
    }
}
