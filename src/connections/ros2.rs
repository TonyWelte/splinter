use std::collections::HashMap;
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};
use std::time::SystemTime;

use crate::common::generic_message::{
    ArrayField, GenericField, GenericMessage, InterfaceType, MessageMetadata, SimpleField,
};
use crate::connections::Connection;

use rclrs::*;
use rosidl_runtime_rs::Sequence;

pub struct ConnectionROS2 {
    // Fields for the ROS2 connection
    node: Node,
    subscriptions: Vec<Arc<DynamicSubscriptionState<Arc<NodeState>>>>,

    #[allow(unused)]
    thread: JoinHandle<()>,
}

impl ConnectionROS2 {
    /// Create a new ROS2 connection.
    pub fn new() -> Self {
        let context = Context::default_from_env().unwrap();
        let mut executor = context.create_basic_executor();
        let node = executor
            .create_node("splinter".start_parameter_services(false))
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

fn populate_message(dynamic_message: DynamicMessageViewMut, generic_message: &GenericMessage) {
    for (name, value) in dynamic_message.iter_mut() {
        if let Some(field) = generic_message.get(name) {
            match (value, field) {
                (
                    ValueMut::Simple(SimpleValueMut::Boolean(v)),
                    GenericField::Simple(SimpleField::Boolean(f)),
                ) => {
                    *v = *f;
                }
                (
                    ValueMut::Simple(SimpleValueMut::Float(v)),
                    GenericField::Simple(SimpleField::Float(f)),
                ) => {
                    *v = *f;
                }
                (
                    ValueMut::Simple(SimpleValueMut::Double(v)),
                    GenericField::Simple(SimpleField::Double(f)),
                ) => {
                    *v = *f;
                }
                (
                    ValueMut::Simple(SimpleValueMut::Int64(v)),
                    GenericField::Simple(SimpleField::Int64(f)),
                ) => {
                    *v = *f;
                }
                (
                    ValueMut::Simple(SimpleValueMut::Uint64(v)),
                    GenericField::Simple(SimpleField::Uint64(f)),
                ) => {
                    *v = *f;
                }
                (
                    ValueMut::Simple(SimpleValueMut::Int32(v)),
                    GenericField::Simple(SimpleField::Int32(f)),
                ) => {
                    *v = *f;
                }
                (
                    ValueMut::Simple(SimpleValueMut::Uint32(v)),
                    GenericField::Simple(SimpleField::Uint32(f)),
                ) => {
                    *v = *f;
                }
                (
                    ValueMut::Simple(SimpleValueMut::Int16(v)),
                    GenericField::Simple(SimpleField::Int16(f)),
                ) => {
                    *v = *f;
                }
                (
                    ValueMut::Simple(SimpleValueMut::Uint16(v)),
                    GenericField::Simple(SimpleField::Uint16(f)),
                ) => {
                    *v = *f;
                }
                (
                    ValueMut::Simple(SimpleValueMut::Int8(v)),
                    GenericField::Simple(SimpleField::Int8(f)),
                ) => {
                    *v = *f;
                }
                (
                    ValueMut::Simple(SimpleValueMut::Uint8(v)),
                    GenericField::Simple(SimpleField::Uint8(f)),
                ) => {
                    *v = *f;
                }
                (
                    ValueMut::Simple(SimpleValueMut::String(v)),
                    GenericField::Simple(SimpleField::String(f)),
                ) => {
                    *v = rosidl_runtime_rs::String::from(f.clone());
                }
                (
                    ValueMut::Simple(SimpleValueMut::Message(v)),
                    GenericField::Simple(SimpleField::Message(f)),
                ) => {
                    populate_message(v, f);
                }
                // Arrays
                (
                    ValueMut::Array(ArrayValueMut::BooleanArray(v)),
                    GenericField::Array(ArrayField::Boolean(f)),
                ) => {
                    v.copy_from_slice(&f);
                }
                (
                    ValueMut::Array(ArrayValueMut::FloatArray(v)),
                    GenericField::Array(ArrayField::Float(f)),
                ) => {
                    v.copy_from_slice(&f);
                }
                (
                    ValueMut::Array(ArrayValueMut::DoubleArray(v)),
                    GenericField::Array(ArrayField::Double(f)),
                ) => {
                    v.copy_from_slice(&f);
                }
                (
                    ValueMut::Array(ArrayValueMut::Int64Array(v)),
                    GenericField::Array(ArrayField::Int64(f)),
                ) => {
                    v.copy_from_slice(&f);
                }
                (
                    ValueMut::Array(ArrayValueMut::Uint64Array(v)),
                    GenericField::Array(ArrayField::Uint64(f)),
                ) => {
                    v.copy_from_slice(&f);
                }
                (
                    ValueMut::Array(ArrayValueMut::Int32Array(v)),
                    GenericField::Array(ArrayField::Int32(f)),
                ) => {
                    v.copy_from_slice(&f);
                }
                (
                    ValueMut::Array(ArrayValueMut::Uint32Array(v)),
                    GenericField::Array(ArrayField::Uint32(f)),
                ) => {
                    v.copy_from_slice(&f);
                }
                (
                    ValueMut::Array(ArrayValueMut::Int16Array(v)),
                    GenericField::Array(ArrayField::Int16(f)),
                ) => {
                    v.copy_from_slice(&f);
                }
                (
                    ValueMut::Array(ArrayValueMut::Uint16Array(v)),
                    GenericField::Array(ArrayField::Uint16(f)),
                ) => {
                    v.copy_from_slice(&f);
                }
                (
                    ValueMut::Array(ArrayValueMut::Int8Array(v)),
                    GenericField::Array(ArrayField::Int8(f)),
                ) => {
                    v.copy_from_slice(&f);
                }
                (
                    ValueMut::Array(ArrayValueMut::Uint8Array(v)),
                    GenericField::Array(ArrayField::Uint8(f)),
                ) => {
                    v.copy_from_slice(&f);
                }
                (
                    ValueMut::Array(ArrayValueMut::StringArray(v)),
                    GenericField::Array(ArrayField::String(f)),
                ) => {
                    for (i, item) in f.iter().enumerate() {
                        if let Some(elem) = v.get_mut(i) {
                            *elem = rosidl_runtime_rs::String::from(item.clone());
                        }
                    }
                }
                // (
                //     ValueMut::Array(ArrayValueMut::MessageArray(v)),
                //     GenericField::Array(ArrayField::Message(f)),
                // ) => {
                //     for (i, item) in f.iter().enumerate() {
                //         if let Some(elem) = v.get(i) {
                //             populate_message(elem, item);
                //         }
                //     }
                // }
                (
                    ValueMut::Sequence(SequenceValueMut::BooleanSequence(v)),
                    GenericField::Array(ArrayField::Boolean(f)),
                ) => {
                    *v = Sequence::new(f.len());
                    v.copy_from_slice(&f);
                }
                _ => {
                    eprintln!("Type mismatch for field: {}", name);
                    // TODO(@TonyWelte): Handle other types
                }
            }
        }
    }
}

impl Into<DynamicMessage> for &GenericMessage {
    fn into(self) -> DynamicMessage {
        let message_type = MessageTypeName {
            package_name: self.type_name().package_name.clone(),
            type_name: self.type_name().type_name.clone(),
        };
        let mut dynamic_message = DynamicMessage::new(message_type).unwrap();

        populate_message(dynamic_message.view_mut(), &self);

        dynamic_message
    }
}

impl Connection for ConnectionROS2 {
    /// Get the name of the connection.
    fn name(&self) -> &str {
        "ROS2 Connection"
    }

    /// Get the type of the connection.
    fn list_topics(&self) -> Vec<(String, InterfaceType)> {
        self.node
            .get_topic_names_and_types()
            .unwrap()
            .into_iter()
            .map(|(name, types)| {
                let first_type = types
                    .first()
                    .map(|type_name| InterfaceType::new(&type_name))
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
    ) -> Result<Box<dyn Fn(&GenericMessage)>, String> {
        let publisher = self
            .node
            .create_dynamic_publisher(message_type.clone(), topic.reliable().transient_local())
            .map_err(|e| format!("Failed to create publisher: {}", e))?;

        let publish_fn = move |msg: &GenericMessage| {
            publisher.publish(msg.into()).unwrap();
        };
        Ok(Box::new(publish_fn))
    }
}
