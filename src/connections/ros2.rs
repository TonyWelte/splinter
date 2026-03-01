use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};
use std::time::{Duration, SystemTime};

use crate::common::generic_message::{
    ArrayField, GenericField, GenericMessage, InterfaceType, MessageMetadata, SimpleField,
};
use crate::connections::{Connection, NamedInterface, NodeName, Parameters};

use rcl_interfaces::msg::ParameterValue;
use rclrs::*;
use rosidl_runtime_rs::Sequence;

use rcl_interfaces::srv::{
    GetParameters, GetParameters_Request, GetParameters_Response, ListParameters,
    ListParameters_Request, ListParameters_Response,
};

const DEFAULT_SERVICE_TIMEOUT: Duration = Duration::from_millis(500);

/// Submit an async task to the already-spinning background executor and block
/// the calling thread until the result arrives (or the timeout expires).
fn run_blocking<T: Send + 'static>(
    node: &Node,
    timeout: Duration,
    f: impl Future<Output = T> + Send + 'static,
) -> Result<T, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    let _promise = node.commands().run(async move {
        let result = f.await;
        let _ = tx.send(result);
    });
    rx.recv_timeout(timeout)
        .map_err(|e| format!("Service call failed (timeout: {timeout:?}): {e}"))
}

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
                types
                    .first()
                    .and_then(|type_name| InterfaceType::new(type_name).ok())
                    .map(|interface_type| (name, interface_type))
            })
            .collect();

        Ok(topics)
    }

    fn list_nodes(&self) -> Result<Vec<NodeName>, String> {
        self.node
            .get_node_names()
            .map_err(|e| format!("Failed to get node names: {}", e))
            .map(|node_names| {
                node_names
                    .iter()
                    .map(|node_name_info| NodeName {
                        name: node_name_info.name.clone(),
                        namespace: node_name_info.namespace.clone(),
                    })
                    .collect()
            })
    }

    /// Get the type of a specific topic.
    fn get_topic_type(&self, topic: &str) -> Option<InterfaceType> {
        self.list_topics()
            .ok()?
            .into_iter()
            .find(|(name, _)| name == topic)
            .map(|(_, interface_type)| interface_type)
    }

    fn get_publisher_names_and_types_by_node(
        &self,
        node_name: &NodeName,
    ) -> Result<Vec<NamedInterface>, String> {
        self.node
            .get_publisher_names_and_types_by_node(&node_name.name, &node_name.namespace)
            .map_err(|_| {
                format!("Failed to get publishers for node: {}", node_name.name).to_string()
            })
            .map(|map| {
                map.into_iter()
                    .filter_map(|(name, types)| {
                        types
                            .into_iter()
                            .next()
                            .and_then(|t| InterfaceType::new(&t).ok())
                            .map(|type_name| NamedInterface { name, type_name })
                    })
                    .collect()
            })
    }

    fn get_subscription_names_and_types_by_node(
        &self,
        node_name: &NodeName,
    ) -> Result<Vec<NamedInterface>, String> {
        self.node
            .get_subscription_names_and_types_by_node(&node_name.name, &node_name.namespace)
            .map_err(|_| {
                format!("Failed to get subscriptions for node: {}", node_name.name).to_string()
            })
            .map(|map| {
                map.into_iter()
                    .filter_map(|(name, types)| {
                        types
                            .into_iter()
                            .next()
                            .and_then(|t| InterfaceType::new(&t).ok())
                            .map(|type_name| NamedInterface { name, type_name })
                    })
                    .collect()
            })
    }

    fn get_client_names_and_types_by_node(
        &self,
        node_name: &NodeName,
    ) -> Result<Vec<NamedInterface>, String> {
        self.node
            .get_client_names_and_types_by_node(&node_name.name, &node_name.namespace)
            .map_err(|_| format!("Failed to get clients for node: {}", node_name.name).to_string())
            .map(|map| {
                map.into_iter()
                    .filter_map(|(name, types)| {
                        types
                            .into_iter()
                            .next()
                            .and_then(|t| InterfaceType::new(&t).ok())
                            .map(|type_name| NamedInterface { name, type_name })
                    })
                    .collect()
            })
    }

    fn get_service_names_and_types_by_node(
        &self,
        node_name: &NodeName,
    ) -> Result<Vec<NamedInterface>, String> {
        self.node
            .get_service_names_and_types_by_node(&node_name.name, &node_name.namespace)
            .map_err(|_| format!("Failed to get services for node: {}", node_name.name).to_string())
            .map(|map| {
                map.into_iter()
                    .filter_map(|(name, types)| {
                        types
                            .into_iter()
                            .next()
                            .and_then(|t| InterfaceType::new(&t).ok())
                            .map(|type_name| NamedInterface { name, type_name })
                    })
                    .collect()
            })
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

        // List parameters
        let service_name = format!("{}{}/list_parameters", namespace, &node_name.name);
        let client = self
            .node
            .create_client::<ListParameters>(&service_name)
            .map_err(|_| format!("Failed to create client for service: {}", &service_name))?;

        let param_names = run_blocking(&self.node, DEFAULT_SERVICE_TIMEOUT, async move {
            client.notify_on_service_ready().await.unwrap();
            let request = ListParameters_Request {
                prefixes: vec![],
                depth: ListParameters_Request::DEPTH_RECURSIVE,
            };
            client
                .call::<_, ListParameters_Response>(&request)
                .unwrap()
                .await
                .unwrap()
        })?
        .result
        .names;

        // Get parameter values
        let service_name = format!("{}{}/get_parameters", namespace, &node_name.name);
        let client = self
            .node
            .create_client::<GetParameters>(&service_name)
            .map_err(|_| format!("Failed to create client for service: {}", &service_name))?;

        let request = GetParameters_Request {
            names: param_names.clone(),
        };

        let response: GetParameters_Response =
            run_blocking(&self.node, DEFAULT_SERVICE_TIMEOUT, async move {
                client.notify_on_service_ready().await.unwrap();
                client
                    .call::<_, GetParameters_Response>(&request)
                    .unwrap()
                    .await
                    .unwrap()
            })?;

        let mut params_map: HashMap<String, Parameters> = HashMap::new();
        for (name, value) in param_names.iter().zip(response.values.iter()) {
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
        let namespace = if node_name.namespace.is_empty() {
            "/".to_string()
        } else {
            node_name.namespace.clone()
        };

        let service_name = format!("{}{}/set_parameters", namespace, &node_name.name);
        let client = self
            .node
            .create_client::<rcl_interfaces::srv::SetParameters>(&service_name)
            .map_err(|_| format!("Failed to create client for service: {}", &service_name))?;

        let request = rcl_interfaces::srv::SetParameters_Request {
            parameters: vec![rcl_interfaces::msg::Parameter {
                name: parameter_name.to_string(),
                value: parameter.into(),
            }],
        };

        let response: rcl_interfaces::srv::SetParameters_Response =
            run_blocking(&self.node, DEFAULT_SERVICE_TIMEOUT, async move {
                client.notify_on_service_ready().await.unwrap();
                client
                    .call::<_, rcl_interfaces::srv::SetParameters_Response>(&request)
                    .unwrap()
                    .await
                    .unwrap()
            })?;

        for result in response.results {
            if !result.successful {
                return Err(format!("Failed to set parameter: {}", result.reason));
            }
        }

        Ok(())
    }

    fn get_publishers_info_by_topic(&self, topic: &str) -> Result<Vec<NodeName>, String> {
        self.node
            .get_publishers_info_by_topic(topic)
            .map_err(|e| format!("Failed to get publishers info for topic '{}': {}", topic, e))
            .map(|infos| {
                infos
                    .into_iter()
                    .map(|i| NodeName {
                        name: i.node_name,
                        namespace: i.node_namespace,
                    })
                    .collect()
            })
    }

    fn get_subscriptions_info_by_topic(&self, topic: &str) -> Result<Vec<NodeName>, String> {
        self.node
            .get_subscriptions_info_by_topic(topic)
            .map_err(|e| {
                format!(
                    "Failed to get subscriptions info for topic '{}': {}",
                    topic, e
                )
            })
            .map(|infos| {
                infos
                    .into_iter()
                    .map(|i| NodeName {
                        name: i.node_name,
                        namespace: i.node_namespace,
                    })
                    .collect()
            })
    }

    fn list_services(&self) -> Result<Vec<(String, InterfaceType)>, String> {
        let services = self
            .node
            .get_service_names_and_types()
            .map_err(|_| "Failed to get service names and types".to_string())?;

        let services: Vec<(String, InterfaceType)> = services
            .into_iter()
            .filter_map(|(name, types)| {
                types
                    .first()
                    .and_then(|type_name| InterfaceType::new(type_name).ok())
                    .map(|interface_type| (name, interface_type))
            })
            .collect();

        Ok(services)
    }

    fn get_service_type(&self, service_name: &str) -> Option<InterfaceType> {
        self.list_services()
            .ok()?
            .into_iter()
            .find(|(name, _)| name == service_name)
            .map(|(_, interface_type)| interface_type)
    }

    fn get_service_request_template(
        &self,
        service_type: &InterfaceType,
    ) -> Result<GenericMessage, String> {
        let service_type_name: ServiceTypeName = format!(
            "{}/srv/{}",
            service_type.package_name, service_type.type_name
        )
        .as_str()
        .try_into()
        .map_err(|e| format!("Invalid service type: {e:?}"))?;

        let metadata = DynamicServiceMetadata::new(service_type_name)
            .map_err(|e| format!("Failed to load service metadata: {e:?}"))?;

        let request = metadata
            .request_metadata
            .create()
            .map_err(|e| format!("Failed to create request template: {e:?}"))?;

        Ok(GenericMessage::from(request.view()))
    }

    fn call_service(
        &self,
        service_name: &str,
        service_type: &InterfaceType,
        request: &GenericMessage,
    ) -> Result<GenericMessage, String> {
        let service_type_name: ServiceTypeName = format!(
            "{}/srv/{}",
            service_type.package_name, service_type.type_name
        )
        .as_str()
        .try_into()
        .map_err(|e| format!("Invalid service type: {e:?}"))?;

        let metadata = DynamicServiceMetadata::new(service_type_name)
            .map_err(|e| format!("Failed to load service metadata: {e:?}"))?;

        let request_metadata = metadata.request_metadata.clone();

        let client = self
            .node
            .create_dynamic_client(metadata, service_name)
            .map_err(|e| format!("Failed to create dynamic client: {e}"))?;

        // Build the DynamicMessage request from the GenericMessage
        let mut dynamic_request = request_metadata
            .create()
            .map_err(|e| format!("Failed to create request message: {e:?}"))?;
        populate_message(dynamic_request.view_mut(), request);

        let (response, _info) = run_blocking(&self.node, DEFAULT_SERVICE_TIMEOUT, async move {
            client.notify_on_service_ready().await.unwrap();
            client.call(dynamic_request).unwrap().await.unwrap()
        })?;

        Ok(GenericMessage::from(response.view()))
    }
}
