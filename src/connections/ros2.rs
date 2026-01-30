use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::thread::{spawn, JoinHandle};
use std::time::SystemTime;

use crate::common::generic_message::{
    ArrayField, GenericField, GenericMessage, InterfaceType, MessageMetadata, SimpleField,
};
use crate::connections::{Connection, NodeName, Parameters};

use rcl_interfaces::msg::ParameterValue;
use rclrs::*;
use rosidl_runtime_rs::Sequence;

use rcl_interfaces::srv::{
    GetParameters, GetParameters_Request, GetParameters_Response, ListParameters,
    ListParameters_Request, ListParameters_Response,
};

pub struct ConnectionROS2 {
    // Fields for the ROS2 connection
    node: Node,
    subscriptions: Vec<Arc<DynamicSubscriptionState<Arc<NodeState>>>>,

    #[allow(unused)]
    thread: JoinHandle<()>,
}

impl Debug for ConnectionROS2 {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("Implement Debug for ConnectionROS2")
    }
}

impl Default for ConnectionROS2 {
    fn default() -> Self {
        Self::new()
    }
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
                    v.copy_from_slice(f);
                }
                (
                    ValueMut::Array(ArrayValueMut::FloatArray(v)),
                    GenericField::Array(ArrayField::Float(f)),
                ) => {
                    v.copy_from_slice(f);
                }
                (
                    ValueMut::Array(ArrayValueMut::DoubleArray(v)),
                    GenericField::Array(ArrayField::Double(f)),
                ) => {
                    v.copy_from_slice(f);
                }
                (
                    ValueMut::Array(ArrayValueMut::Int64Array(v)),
                    GenericField::Array(ArrayField::Int64(f)),
                ) => {
                    v.copy_from_slice(f);
                }
                (
                    ValueMut::Array(ArrayValueMut::Uint64Array(v)),
                    GenericField::Array(ArrayField::Uint64(f)),
                ) => {
                    v.copy_from_slice(f);
                }
                (
                    ValueMut::Array(ArrayValueMut::Int32Array(v)),
                    GenericField::Array(ArrayField::Int32(f)),
                ) => {
                    v.copy_from_slice(f);
                }
                (
                    ValueMut::Array(ArrayValueMut::Uint32Array(v)),
                    GenericField::Array(ArrayField::Uint32(f)),
                ) => {
                    v.copy_from_slice(f);
                }
                (
                    ValueMut::Array(ArrayValueMut::Int16Array(v)),
                    GenericField::Array(ArrayField::Int16(f)),
                ) => {
                    v.copy_from_slice(f);
                }
                (
                    ValueMut::Array(ArrayValueMut::Uint16Array(v)),
                    GenericField::Array(ArrayField::Uint16(f)),
                ) => {
                    v.copy_from_slice(f);
                }
                (
                    ValueMut::Array(ArrayValueMut::Int8Array(v)),
                    GenericField::Array(ArrayField::Int8(f)),
                ) => {
                    v.copy_from_slice(f);
                }
                (
                    ValueMut::Array(ArrayValueMut::Uint8Array(v)),
                    GenericField::Array(ArrayField::Uint8(f)),
                ) => {
                    v.copy_from_slice(f);
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
                    v.copy_from_slice(f);
                }
                _ => {
                    eprintln!("Type mismatch for field: {}", name);
                    // TODO(@TonyWelte): Handle other types
                }
            }
        }
    }
}

impl TryFrom<&GenericMessage> for DynamicMessage {
    type Error = String;

    fn try_from(val: &GenericMessage) -> Result<Self, Self::Error> {
        let message_type = MessageTypeName {
            package_name: val.type_name().package_name.clone(),
            type_name: val.type_name().type_name.clone(),
        };
        let mut dynamic_message = DynamicMessage::new(message_type).map_err(|_| {
            format!("Failed to create dynamic message: {:?}", val.type_name()).to_string()
        })?;

        populate_message(dynamic_message.view_mut(), val);

        Ok(dynamic_message)
    }
}

impl From<&rcl_interfaces::msg::ParameterValue> for Parameters {
    fn from(value: &rcl_interfaces::msg::ParameterValue) -> Self {
        match value.type_ {
            rcl_interfaces::msg::ParameterType::PARAMETER_BOOL => {
                Parameters::Bool(value.bool_value)
            }
            rcl_interfaces::msg::ParameterType::PARAMETER_INTEGER => {
                Parameters::Integer(value.integer_value)
            }
            rcl_interfaces::msg::ParameterType::PARAMETER_DOUBLE => {
                Parameters::Double(value.double_value)
            }
            rcl_interfaces::msg::ParameterType::PARAMETER_STRING => {
                Parameters::String(value.string_value.clone())
            }
            rcl_interfaces::msg::ParameterType::PARAMETER_BYTE_ARRAY => {
                Parameters::ByteArray(value.byte_array_value.clone())
            }
            rcl_interfaces::msg::ParameterType::PARAMETER_BOOL_ARRAY => {
                Parameters::BoolArray(value.bool_array_value.clone())
            }
            rcl_interfaces::msg::ParameterType::PARAMETER_INTEGER_ARRAY => {
                Parameters::IntegerArray(value.integer_array_value.clone())
            }
            rcl_interfaces::msg::ParameterType::PARAMETER_DOUBLE_ARRAY => {
                Parameters::DoubleArray(value.double_array_value.clone())
            }
            rcl_interfaces::msg::ParameterType::PARAMETER_STRING_ARRAY => Parameters::StringArray(
                value
                    .string_array_value
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            ),
            _ => panic!("Unknown parameter type"),
        }
    }
}

impl From<Parameters> for rcl_interfaces::msg::ParameterValue {
    fn from(param: Parameters) -> Self {
        match param {
            Parameters::Bool(v) => ParameterValue {
                type_: rcl_interfaces::msg::ParameterType::PARAMETER_BOOL,
                bool_value: v,
                ..Default::default()
            },
            Parameters::Integer(v) => ParameterValue {
                type_: rcl_interfaces::msg::ParameterType::PARAMETER_INTEGER,
                integer_value: v,
                ..Default::default()
            },
            Parameters::Double(v) => ParameterValue {
                type_: rcl_interfaces::msg::ParameterType::PARAMETER_DOUBLE,
                double_value: v,
                ..Default::default()
            },
            Parameters::String(v) => ParameterValue {
                type_: rcl_interfaces::msg::ParameterType::PARAMETER_STRING,
                string_value: v,
                ..Default::default()
            },
            Parameters::ByteArray(v) => ParameterValue {
                type_: rcl_interfaces::msg::ParameterType::PARAMETER_BYTE_ARRAY,
                byte_array_value: v,
                ..Default::default()
            },
            Parameters::BoolArray(v) => ParameterValue {
                type_: rcl_interfaces::msg::ParameterType::PARAMETER_BOOL_ARRAY,
                bool_array_value: v,
                ..Default::default()
            },
            Parameters::IntegerArray(v) => ParameterValue {
                type_: rcl_interfaces::msg::ParameterType::PARAMETER_INTEGER_ARRAY,
                integer_array_value: v,
                ..Default::default()
            },
            Parameters::DoubleArray(v) => ParameterValue {
                type_: rcl_interfaces::msg::ParameterType::PARAMETER_DOUBLE_ARRAY,
                double_array_value: v,
                ..Default::default()
            },
            Parameters::StringArray(v) => ParameterValue {
                type_: rcl_interfaces::msg::ParameterType::PARAMETER_STRING_ARRAY,
                string_array_value: v.to_vec(),
                ..Default::default()
            },
        }
    }
}

impl Connection for ConnectionROS2 {
    /// Get the name of the connection.
    fn name(&self) -> &str {
        "ROS2 Connection"
    }

    /// Get the type of the connection.
    fn list_topics(&self) -> Result<Vec<(String, InterfaceType)>, String> {
        // Get topic names and types
        let topics = self
            .node
            .get_topic_names_and_types()
            .map_err(|_| "Failed to get topic names and types".to_string())?;

        // Convert to InterfaceType
        let topics: Vec<(String, InterfaceType)> = topics
            .into_iter()
            .filter_map(|(name, types)| {
                types.first().map(|type_name| {
                    let interface_type = InterfaceType::new(type_name);
                    (name, interface_type)
                })
            })
            .collect();

        Ok(topics)
    }

    fn list_nodes(&self) -> Vec<NodeName> {
        self.node
            .get_node_names()
            .expect("Failed to get node names")
            .iter()
            .map(|node_name_info| NodeName {
                name: node_name_info.name.clone(),
                namespace: node_name_info.namespace.clone(),
            })
            .collect()
    }

    /// Get the type of a specific topic.
    fn get_topic_type(&self, topic: &str) -> Option<InterfaceType> {
        let topics = self.node.get_topic_names_and_types().ok()?;

        topics
            .into_iter()
            .find(|(name, _types)| name == topic)
            .and_then(|(_name, types)| {
                types.first().map(|type_name| {
                    let parts: Vec<&str> = type_name.split('/').collect();
                    InterfaceType {
                        package_name: parts.first().unwrap_or(&"").to_string(),
                        category: parts.get(1).unwrap_or(&"").to_string(),
                        type_name: parts.get(2).unwrap_or(&"").to_string(),
                    }
                })
            })
    }

    fn get_publisher_names_and_types_by_node(
        &self,
        node_name: &NodeName,
    ) -> Result<HashMap<String, Vec<String>>, String> {
        self.node
            .get_publisher_names_and_types_by_node(&node_name.name, &node_name.namespace)
            .map_err(|_| {
                format!("Failed to get publishers for node: {}", node_name.name).to_string()
            })
    }

    fn get_subscription_names_and_types_by_node(
        &self,
        node_name: &NodeName,
    ) -> Result<HashMap<String, Vec<String>>, String> {
        self.node
            .get_subscription_names_and_types_by_node(&node_name.name, &node_name.namespace)
            .map_err(|_| {
                format!("Failed to get subscriptions for node: {}", node_name.name).to_string()
            })
    }

    fn get_client_names_and_types_by_node(
        &self,
        node_name: &NodeName,
    ) -> Result<HashMap<String, Vec<String>>, String> {
        self.node
            .get_client_names_and_types_by_node(&node_name.name, &node_name.namespace)
            .map_err(|_| format!("Failed to get clients for node: {}", node_name.name).to_string())
    }

    fn get_service_names_and_types_by_node(
        &self,
        node_name: &NodeName,
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
        // Get topic type
        let topic_type = self
            .get_topic_type(topic)
            .ok_or(format!("Failed to get topic type for topic: {}", topic))?;
        let topic_type2 = topic_type.clone();
        // Create subscription
        let subscription = self
            .node
            .create_dynamic_subscription(
                MessageTypeName {
                    package_name: topic_type.package_name,
                    type_name: topic_type.type_name,
                },
                topic,
                move |msg: DynamicMessage, msg_info: MessageInfo| {
                    let metadata = MessageMetadata {
                        received_time: msg_info.received_timestamp.unwrap_or(SystemTime::now()),
                    };
                    let generic_message = GenericMessage::from(msg.view());
                    callback(generic_message, metadata);
                },
            )
            .map_err(|e| {
                format!(
                    "Failed to create subscription: {} (topic: {}, type {:?})",
                    e, topic, topic_type2
                )
            })?;
        self.subscriptions.push(subscription);

        Ok(())
    }

    fn create_publisher(
        &mut self,
        topic: &str,
        message_type: &InterfaceType,
    ) -> Result<Box<dyn Fn(&GenericMessage) -> Result<(), String>>, String> {
        let publisher = self
            .node
            .create_dynamic_publisher(
                MessageTypeName {
                    package_name: message_type.package_name.clone(),
                    type_name: message_type.type_name.clone(),
                },
                topic.reliable().transient_local(),
            )
            .map_err(|e| format!("Failed to create publisher: {}", e))?;

        let publish_fn = move |msg: &GenericMessage| {
            let dynamic_message = msg
                .try_into()
                .map_err(|_| "Failed to convert GenericMessage to DynamicMessage")?;

            publisher
                .publish(dynamic_message)
                .map_err(|e| format!("Failed to publish message: {}", e))?;

            Ok(())
        };
        Ok(Box::new(publish_fn))
    }

    fn get_parameters_by_node(
        &self,
        node_name: &NodeName,
    ) -> Result<HashMap<String, Parameters>, String> {
        let namespace = if node_name.namespace.is_empty() {
            "/".to_string()
        } else {
            node_name.namespace.clone()
        };

        // Executor
        let mut executor = Context::default_from_env()
            .map_err(|_| "Error creating context")?
            .create_basic_executor();

        // Get parameters list
        let service_name = format!("{}{}/list_parameters", namespace, &node_name.name);
        let client = self
            .node
            .create_client::<ListParameters>(&service_name)
            .map_err(|_| format!("Failed to create client for sevice: {}", &service_name))?;

        client
            .service_is_ready()
            .map_err(|_| format!("Service not available for sevice: {}", &service_name))?;

        let response = Arc::new(Mutex::new(Option::<ListParameters_Response>::None));
        let response_clone = Arc::clone(&response);

        let promise = executor.commands().run(async move {
            client.notify_on_service_ready().await.unwrap();

            let request = ListParameters_Request {
                prefixes: vec![],
                depth: ListParameters_Request::DEPTH_RECURSIVE,
            };

            // TODO(@TonyWelte): Handle error
            let response: ListParameters_Response = client.call(&request).unwrap().await.unwrap();

            response_clone.lock().unwrap().replace(response);
        });

        executor
            .spin(
                SpinOptions::new()
                    .until_promise_resolved(promise)
                    .timeout(std::time::Duration::from_millis(100)),
            )
            .first_error()
            .map_err(|_| format!("Error when running service: {}", &service_name))?;

        let response = response.lock().unwrap().take();

        let param_names = response
            .ok_or(format!("Service call failed to service: {}", &service_name))?
            .result
            .names;

        // Get parameter values
        let service_name = format!("{}{}/get_parameters", namespace, &node_name.name);
        let client = self
            .node
            .create_client::<GetParameters>(&service_name)
            .map_err(|_| format!("Failed to create client for node: {}", node_name.name))?;

        let response = Arc::new(Mutex::new(Option::<GetParameters_Response>::None));
        let response_clone = Arc::clone(&response);

        let request = GetParameters_Request {
            names: param_names.clone(),
        };

        let promise = executor.commands().run(async move {
            client.notify_on_service_ready().await.unwrap();

            // TODO: Handle error
            let response: GetParameters_Response = client.call(&request).unwrap().await.unwrap();

            response_clone.lock().unwrap().replace(response);
        });

        executor
            .spin(
                SpinOptions::new()
                    .until_promise_resolved(promise)
                    .timeout(std::time::Duration::from_millis(100)),
            )
            .first_error()
            .map_err(|_| format!("Error when running service: {}", &service_name))?;

        let response = response.lock().unwrap().take();

        let parameters = response.ok_or("Service call failed")?;
        let mut params_map = HashMap::new();
        for (name, value) in param_names.iter().zip(parameters.values.iter()) {
            params_map.insert(name.clone(), value.into());
        }

        Ok(params_map)
    }

    fn set_parameter_by_node(
        &mut self,
        node_name: &NodeName,
        parameter_name: &str,
        parameter: Parameters,
    ) -> Result<(), String> {
        // Executor
        let mut executor = Context::default_from_env()
            .map_err(|_| "Error creating context")?
            .create_basic_executor();

        let namespace = if node_name.namespace.is_empty() {
            "/".to_string()
        } else {
            node_name.namespace.clone()
        };

        // Set parameter
        let service_name = format!("{}{}/set_parameters", namespace, &node_name.name);
        let client = self
            .node
            .create_client::<rcl_interfaces::srv::SetParameters>(&service_name)
            .map_err(|_| format!("Failed to create client for sevice: {}", &service_name))?;

        client
            .service_is_ready()
            .map_err(|_| format!("Service not available for sevice: {}", &service_name))?;

        let response = Arc::new(Mutex::new(
            Option::<rcl_interfaces::srv::SetParameters_Response>::None,
        ));
        let response_clone = Arc::clone(&response);

        let request = rcl_interfaces::srv::SetParameters_Request {
            parameters: vec![rcl_interfaces::msg::Parameter {
                name: parameter_name.to_string(),
                value: parameter.into(),
            }],
        };

        let promise = executor.commands().run(async move {
            client.notify_on_service_ready().await.unwrap();

            let response = client.call(&request).unwrap().await.unwrap();
            response_clone.lock().unwrap().replace(response);
        });

        executor
            .spin(
                SpinOptions::new()
                    .until_promise_resolved(promise)
                    .timeout(std::time::Duration::from_millis(100)),
            )
            .first_error()
            .map_err(|_| format!("Error when running service: {}", &service_name))?;

        let response = response
            .lock()
            .unwrap()
            .take()
            .ok_or("Service call failed".to_string())?;

        for result in response.results {
            if !result.successful {
                return Err(format!("Failed to set parameter: {}", result.reason));
            }
        }

        Ok(())
    }
}
