use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};
use std::time::{Duration, SystemTime};

use crate::common::generic_message::{
    ArrayField, BoundedSequenceField, GenericField, GenericMessage, InterfaceType, MessageMetadata,
    SequenceField, SimpleField,
};
use crate::connections::{Connection, NamedInterface, NodeName, Parameters};

use rcl_interfaces::msg::ParameterValue;
use rclrs::*;
use rosidl_runtime_rs::{Sequence, SequenceAlloc};

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

// Generates all match arms for a set of primitive numeric types, expanding four variants each:
// scalar copy, fixed-array copy, unbounded-sequence copy, and bounded-sequence copy.
// Special-cased types (String, WString, Message) are handled as explicit arms below.
//
// Arguments:
//   $value  – the ValueMut expression
//   $field  – the GenericField expression
//   $name   – &str used in error/bounded-sequence calls
//
// Per-type tuple: (SimpleVariant, ArrayVariant, SequenceVariant, BoundedSequenceVariant)
macro_rules! dispatch_primitive_field {
    (
        $value:expr, $field:expr, $name:expr, $errors:expr;
        $(($prim:ident, $arr:ident, $seq:ident, $bseq:ident)),* $(,)?
    ) => {
        match ($value, $field) {
            // ---- Primitive scalars ----
            $(
                (ValueMut::Simple(SimpleValueMut::$prim(v)), GenericField::Simple(SimpleField::$prim(f))) => {
                    *v = *f;
                }
                (ValueMut::Array(ArrayValueMut::$arr(v)), GenericField::Array(ArrayField::$prim(f))) => {
                    v.copy_from_slice(f);
                }
                (ValueMut::Sequence(SequenceValueMut::$seq(v)), GenericField::Sequence(SequenceField::$prim(f))) => {
                    *v = Sequence::new(f.len());
                    v.copy_from_slice(f);
                }
                (ValueMut::BoundedSequence(BoundedSequenceValueMut::$bseq(v)), GenericField::BoundedSequence(BoundedSequenceField::$prim(f, _))) => {
                    if let Some(e) = populate_bounded_sequence(v, f, $name) {
                        $errors.push(e);
                    }
                }
            )*

            // ---- String (scalar) ----
            (ValueMut::Simple(SimpleValueMut::String(v)), GenericField::Simple(SimpleField::String(f))) => {
                *v = rosidl_runtime_rs::String::from(f.clone());
            }
            // ---- Nested message (scalar) ----
            (ValueMut::Simple(SimpleValueMut::Message(mut v)), GenericField::Simple(SimpleField::Message(f))) => {
                $errors.extend(populate_message(&mut v, f));
            }

            // ---- String / WString arrays ----
            (ValueMut::Array(ArrayValueMut::StringArray(v)), GenericField::Array(ArrayField::String(f))) => {
                for (i, item) in f.iter().enumerate() {
                    if let Some(elem) = v.get_mut(i) {
                        *elem = rosidl_runtime_rs::String::from(item.as_ref());
                    }
                }
            }
            (ValueMut::Array(ArrayValueMut::WStringArray(v)), GenericField::Array(ArrayField::String(f))) => {
                for (i, item) in f.iter().enumerate() {
                    if let Some(elem) = v.get_mut(i) {
                        *elem = rosidl_runtime_rs::WString::from(item.as_ref());
                    }
                }
            }
            // ---- Message array ----
            (ValueMut::Array(ArrayValueMut::MessageArray(mut v)), GenericField::Array(ArrayField::Message(f))) => {
                for (i, item) in f.iter().enumerate() {
                    if let Some(elem) = v.get_mut(i) {
                        $errors.extend(populate_message(elem, item));
                    }
                }
            }

            // ---- String / WString sequences ----
            (ValueMut::Sequence(SequenceValueMut::StringSequence(v)), GenericField::Sequence(SequenceField::String(f))) => {
                *v = Sequence::new(f.len());
                for (i, item) in f.iter().enumerate() {
                    if let Some(elem) = v.get_mut(i) {
                        *elem = rosidl_runtime_rs::String::from(item.as_ref());
                    }
                }
            }
            (ValueMut::Sequence(SequenceValueMut::WStringSequence(v)), GenericField::Sequence(SequenceField::String(f))) => {
                *v = Sequence::new(f.len());
                for (i, item) in f.iter().enumerate() {
                    if let Some(elem) = v.get_mut(i) {
                        *elem = rosidl_runtime_rs::WString::from(item.as_ref());
                    }
                }
            }
            // ---- Message sequence ----
            (ValueMut::Sequence(SequenceValueMut::MessageSequence(mut v)), GenericField::Sequence(SequenceField::Message(f))) => {
                v.reset(f.len());
                for (i, item) in f.iter().enumerate() {
                    if let Some(elem) = v.get_mut(i) {
                        $errors.extend(populate_message(elem, item));
                    }
                }
            }

            // ---- Message bounded sequence ----
            (ValueMut::BoundedSequence(BoundedSequenceValueMut::MessageBoundedSequence(mut v)), GenericField::BoundedSequence(BoundedSequenceField::Message(f, _))) => {
                if let Err(e) = v.try_reset(f.len()) {
                    $errors.push(format!("Failed to resize bounded message sequence for field '{}': {}", $name, e));
                } else {
                    for (i, item) in f.iter().enumerate() {
                        if let Some(elem) = v.get_mut(i) {
                            $errors.extend(populate_message(elem, item));
                        }
                    }
                }
            }

            // ---- String / WString bounded sequences ----
            (ValueMut::BoundedSequence(BoundedSequenceValueMut::StringBoundedSequence(mut v)), GenericField::BoundedSequence(BoundedSequenceField::String(f, _))) => {
                if let Err(e) = v.try_reset(f.len()) {
                    $errors.push(format!("Failed to resize bounded string sequence for field '{}': {}", $name, e));
                } else {
                    for (i, item) in f.iter().enumerate() {
                        if let Some(elem) = v.get_mut(i) {
                            *elem = rosidl_runtime_rs::String::from(item.as_ref());
                        }
                    }
                }
            }
            (ValueMut::BoundedSequence(BoundedSequenceValueMut::WStringBoundedSequence(mut v)), GenericField::BoundedSequence(BoundedSequenceField::WString(f, _))) => {
                if let Err(e) = v.try_reset(f.len()) {
                    $errors.push(format!("Failed to resize bounded wstring sequence for field '{}': {}", $name, e));
                } else {
                    for (i, item) in f.iter().enumerate() {
                        if let Some(elem) = v.get_mut(i) {
                            *elem = rosidl_runtime_rs::WString::from(item.as_ref());
                        }
                    }
                }
            }
            _ => {
                $errors.push(format!("Type mismatch for field: {}", $name));
            }
        }
    };
}

/// Populates `dynamic_message` from `generic_message`, returning a list of field-level errors.
/// The message is always (partially) populated even when errors are returned.
fn populate_message(
    dynamic_message: &mut DynamicMessageViewMut,
    generic_message: &GenericMessage,
) -> Vec<String> {
    fn populate_bounded_sequence<'msg, T>(
        mut sequence: DynamicBoundedSequenceMut<'msg, T>,
        data: &[T],
        field_name: &str,
    ) -> Option<String>
    where
        T: Copy,
        T: Debug,
        T: PartialEq,
        T: SequenceAlloc,
        T: 'static,
    {
        if let Err(e) = sequence.try_reset(data.len()) {
            return Some(format!(
                "Failed to resize bounded sequence for field '{}': {}",
                field_name, e
            ));
        }
        sequence.copy_from_slice(data);
        None
    }

    let mut errors: Vec<String> = Vec::new();

    let field_names: Vec<String> = dynamic_message
        .structure()
        .fields
        .iter()
        .map(|f| f.name.clone())
        .collect();

    for name in &field_names {
        if let Some(field) = generic_message.get(name) {
            if let Some(value) = dynamic_message.get_mut(name) {
                dispatch_primitive_field!(value, field, name, &mut errors;
                    (Boolean, BooleanArray, BooleanSequence, BooleanBoundedSequence),
                    (Float,   FloatArray,   FloatSequence,   FloatBoundedSequence),
                    (Double,  DoubleArray,  DoubleSequence,  DoubleBoundedSequence),
                    (Int64,   Int64Array,   Int64Sequence,   Int64BoundedSequence),
                    (Uint64,  Uint64Array,  Uint64Sequence,  Uint64BoundedSequence),
                    (Int32,   Int32Array,   Int32Sequence,   Int32BoundedSequence),
                    (Uint32,  Uint32Array,  Uint32Sequence,  Uint32BoundedSequence),
                    (Int16,   Int16Array,   Int16Sequence,   Int16BoundedSequence),
                    (Uint16,  Uint16Array,  Uint16Sequence,  Uint16BoundedSequence),
                    (Int8,    Int8Array,    Int8Sequence,    Int8BoundedSequence),
                    (Uint8,   Uint8Array,   Uint8Sequence,   Uint8BoundedSequence),
                    (Octet,   OctetArray,   OctetSequence,   OctetBoundedSequence),
                );
            }
        }
    }

    errors
}

/// Converts a [`GenericMessage`] into a [`DynamicMessage`], returning the (potentially
/// partially-populated) message alongside any field-level errors encountered during conversion.
///
/// Returns `Err` only when the message type itself cannot be created (e.g. unknown package/type).
pub fn dynamic_message_from_generic(
    val: &GenericMessage,
) -> Result<(DynamicMessage, Vec<String>), String> {
    let message_type = MessageTypeName {
        package_name: val.type_name().package_name.clone(),
        type_name: val.type_name().type_name.clone(),
    };
    let mut dynamic_message = DynamicMessage::new(message_type)
        .map_err(|_| format!("Failed to create dynamic message: {:?}", val.type_name()))?;
    let errors = {
        let mut view = dynamic_message.view_mut();
        populate_message(&mut view, val)
    };
    Ok((dynamic_message, errors))
}

impl TryFrom<&GenericMessage> for DynamicMessage {
    type Error = String;

    fn try_from(val: &GenericMessage) -> Result<Self, Self::Error> {
        dynamic_message_from_generic(val).map(|(msg, _warnings)| msg)
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
    ) -> Result<Box<dyn Fn(&GenericMessage) -> Result<Vec<String>, String>>, String> {
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
            let (dynamic_message, warnings) = dynamic_message_from_generic(msg)
                .map_err(|_| "Failed to convert GenericMessage to DynamicMessage".to_owned())?;

            publisher
                .publish(dynamic_message)
                .map_err(|e| format!("Failed to publish message: {}", e))?;

            Ok(warnings)
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
    ) -> Result<(GenericMessage, Vec<String>), String> {
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
        let request_warnings = {
            let mut view = dynamic_request.view_mut();
            populate_message(&mut view, request)
        };

        let (response, _info) = run_blocking(&self.node, DEFAULT_SERVICE_TIMEOUT, async move {
            client.notify_on_service_ready().await.unwrap();
            client.call(dynamic_request).unwrap().await.unwrap()
        })?;

        Ok((GenericMessage::from(response.view()), request_warnings))
    }
}

// Tests for populate_message and the DynamicMessage <-> GenericMessage conversion
#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::generic_message::InterfaceType;
    use indexmap::IndexMap;
    use rclrs::{DynamicMessage, MessageTypeName};

    fn make_message(package: &str, type_name: &str) -> DynamicMessage {
        DynamicMessage::new(MessageTypeName {
            package_name: package.to_owned(),
            type_name: type_name.to_owned(),
        })
        .unwrap()
    }

    /// Convert a DynamicMessage to GenericMessage, reconstruct a DynamicMessage from that
    /// GenericMessage, convert it again to GenericMessage, and assert both are equal.
    fn assert_round_trip(msg: &DynamicMessage) {
        let generic = GenericMessage::from(msg.view());
        let reconstructed =
            DynamicMessage::try_from(&generic).expect("TryFrom<&GenericMessage> failed");
        let generic2 = GenericMessage::from(reconstructed.view());
        assert_eq!(generic, generic2, "Round-trip GenericMessage mismatch");
    }

    // ── Scalar string ─────────────────────────────────────────────────────────

    #[test]
    fn test_string_scalar_round_trip() {
        let mut msg = make_message("std_msgs", "String");
        if let ValueMut::Simple(SimpleValueMut::String(s)) = msg.view_mut().get_mut("data").unwrap()
        {
            *s = rosidl_runtime_rs::String::from("Hello, ROS2!");
        } else {
            panic!("Expected 'data' to be a String");
        }

        let generic = GenericMessage::from(msg.view());
        assert_eq!(generic.type_name().package_name, "std_msgs");
        assert_eq!(generic.type_name().type_name, "String");
        assert_eq!(
            generic.get("data"),
            Some(&GenericField::Simple(SimpleField::String(
                "Hello, ROS2!".to_owned()
            )))
        );

        assert_round_trip(&msg);
    }

    // ── Scalar primitives ─────────────────────────────────────────────────────

    #[test]
    fn test_scalar_primitives_round_trip() {
        let mut msg = make_message("test_msgs", "BasicTypes");
        {
            let mut view = msg.view_mut();
            macro_rules! set_scalar {
                ($view:expr, $field:literal, $variant:ident, $val:expr) => {
                    if let ValueMut::Simple(SimpleValueMut::$variant(v)) =
                        $view.get_mut($field).unwrap()
                    {
                        *v = $val;
                    } else {
                        panic!("Wrong type for field '{}'", $field);
                    }
                };
            }
            set_scalar!(view, "bool_value", Boolean, true);
            set_scalar!(view, "int8_value", Int8, i8::MIN);
            set_scalar!(view, "uint8_value", Uint8, u8::MAX);
            set_scalar!(view, "int16_value", Int16, i16::MIN);
            set_scalar!(view, "uint16_value", Uint16, u16::MAX);
            set_scalar!(view, "int32_value", Int32, i32::MIN);
            set_scalar!(view, "uint32_value", Uint32, u32::MAX);
            set_scalar!(view, "int64_value", Int64, i64::MIN);
            set_scalar!(view, "uint64_value", Uint64, u64::MAX);
            set_scalar!(view, "float32_value", Float, -1.25_f32);
            set_scalar!(view, "float64_value", Double, std::f64::consts::PI);
        }

        let generic = GenericMessage::from(msg.view());
        assert_eq!(
            generic.get("bool_value"),
            Some(&GenericField::Simple(SimpleField::Boolean(true)))
        );
        assert_eq!(
            generic.get("int8_value"),
            Some(&GenericField::Simple(SimpleField::Int8(i8::MIN)))
        );
        assert_eq!(
            generic.get("uint8_value"),
            Some(&GenericField::Simple(SimpleField::Uint8(u8::MAX)))
        );
        assert_eq!(
            generic.get("int16_value"),
            Some(&GenericField::Simple(SimpleField::Int16(i16::MIN)))
        );
        assert_eq!(
            generic.get("uint16_value"),
            Some(&GenericField::Simple(SimpleField::Uint16(u16::MAX)))
        );
        assert_eq!(
            generic.get("int32_value"),
            Some(&GenericField::Simple(SimpleField::Int32(i32::MIN)))
        );
        assert_eq!(
            generic.get("uint32_value"),
            Some(&GenericField::Simple(SimpleField::Uint32(u32::MAX)))
        );
        assert_eq!(
            generic.get("int64_value"),
            Some(&GenericField::Simple(SimpleField::Int64(i64::MIN)))
        );
        assert_eq!(
            generic.get("uint64_value"),
            Some(&GenericField::Simple(SimpleField::Uint64(u64::MAX)))
        );
        assert_eq!(
            generic.get("float32_value"),
            Some(&GenericField::Simple(SimpleField::Float(-1.25_f32)))
        );
        assert_eq!(
            generic.get("float64_value"),
            Some(&GenericField::Simple(SimpleField::Double(
                std::f64::consts::PI
            )))
        );

        assert_round_trip(&msg);
    }

    // ── Fixed-size arrays ─────────────────────────────────────────────────────

    #[test]
    fn test_fixed_array_round_trip() {
        let mut msg = make_message("test_msgs", "Arrays");
        {
            let mut view = msg.view_mut();
            if let ValueMut::Array(ArrayValueMut::BooleanArray(v)) =
                view.get_mut("bool_values").unwrap()
            {
                v.copy_from_slice(&[true, false, true]);
            }
            if let ValueMut::Array(ArrayValueMut::Int32Array(v)) =
                view.get_mut("int32_values").unwrap()
            {
                v.copy_from_slice(&[-1, 0, 1]);
            }
            if let ValueMut::Array(ArrayValueMut::DoubleArray(v)) =
                view.get_mut("float64_values").unwrap()
            {
                v.copy_from_slice(&[1.1_f64, 2.2_f64, 3.3_f64]);
            }
            if let ValueMut::Array(ArrayValueMut::StringArray(v)) =
                view.get_mut("string_values").unwrap()
            {
                v[0] = rosidl_runtime_rs::String::from("alpha");
                v[1] = rosidl_runtime_rs::String::from("beta");
                v[2] = rosidl_runtime_rs::String::from("gamma");
            }
            if let ValueMut::Simple(SimpleValueMut::Int32(v)) =
                view.get_mut("alignment_check").unwrap()
            {
                *v = 99;
            }
        }

        let generic = GenericMessage::from(msg.view());
        assert_eq!(
            generic.get("bool_values"),
            Some(&GenericField::Array(ArrayField::Boolean(vec![
                true, false, true
            ])))
        );
        assert_eq!(
            generic.get("int32_values"),
            Some(&GenericField::Array(ArrayField::Int32(vec![-1, 0, 1])))
        );
        assert_eq!(
            generic.get("string_values"),
            Some(&GenericField::Array(ArrayField::String(vec![
                "alpha".to_owned(),
                "beta".to_owned(),
                "gamma".to_owned()
            ])))
        );
        assert_eq!(
            generic.get("alignment_check"),
            Some(&GenericField::Simple(SimpleField::Int32(99)))
        );

        assert_round_trip(&msg);
    }

    // ── Unbounded sequences ───────────────────────────────────────────────────

    #[test]
    fn test_unbounded_sequence_round_trip() {
        let mut msg = make_message("test_msgs", "UnboundedSequences");
        {
            let mut view = msg.view_mut();
            if let ValueMut::Sequence(SequenceValueMut::Int32Sequence(v)) =
                view.get_mut("int32_values").unwrap()
            {
                *v = rosidl_runtime_rs::Sequence::new(4);
                v.copy_from_slice(&[10, 20, 30, 40]);
            }
            if let ValueMut::Sequence(SequenceValueMut::BooleanSequence(v)) =
                view.get_mut("bool_values").unwrap()
            {
                *v = rosidl_runtime_rs::Sequence::new(3);
                v.copy_from_slice(&[true, false, true]);
            }
            if let ValueMut::Sequence(SequenceValueMut::DoubleSequence(v)) =
                view.get_mut("float64_values").unwrap()
            {
                *v = rosidl_runtime_rs::Sequence::new(2);
                v.copy_from_slice(&[3.14_f64, 2.72_f64]);
            }
            if let ValueMut::Sequence(SequenceValueMut::StringSequence(v)) =
                view.get_mut("string_values").unwrap()
            {
                *v = rosidl_runtime_rs::Sequence::new(2);
                v[0] = rosidl_runtime_rs::String::from("foo");
                v[1] = rosidl_runtime_rs::String::from("bar");
            }
            if let ValueMut::Simple(SimpleValueMut::Int32(v)) =
                view.get_mut("alignment_check").unwrap()
            {
                *v = 7;
            }
        }

        let generic = GenericMessage::from(msg.view());
        assert_eq!(
            generic.get("int32_values"),
            Some(&GenericField::Sequence(SequenceField::Int32(vec![
                10, 20, 30, 40
            ])))
        );
        assert_eq!(
            generic.get("bool_values"),
            Some(&GenericField::Sequence(SequenceField::Boolean(vec![
                true, false, true
            ])))
        );
        assert_eq!(
            generic.get("string_values"),
            Some(&GenericField::Sequence(SequenceField::String(vec![
                "foo".to_owned(),
                "bar".to_owned()
            ])))
        );
        assert_eq!(
            generic.get("alignment_check"),
            Some(&GenericField::Simple(SimpleField::Int32(7)))
        );

        assert_round_trip(&msg);
    }

    #[test]
    fn test_empty_sequences_default_round_trip() {
        // All sequence fields are empty by default; the round-trip must preserve that.
        let msg = make_message("test_msgs", "UnboundedSequences");
        let generic = GenericMessage::from(msg.view());
        assert_eq!(
            generic.get("int32_values"),
            Some(&GenericField::Sequence(SequenceField::Int32(vec![])))
        );
        assert_round_trip(&msg);
    }

    // ── Bounded sequences ─────────────────────────────────────────────────────

    #[test]
    fn test_bounded_sequence_round_trip() {
        let mut msg = make_message("test_msgs", "BoundedSequences");
        {
            let mut view = msg.view_mut();
            if let ValueMut::BoundedSequence(BoundedSequenceValueMut::Int32BoundedSequence(mut v)) =
                view.get_mut("int32_values").unwrap()
            {
                v.try_reset(2).unwrap();
                v.copy_from_slice(&[100, 200]);
            }
            if let ValueMut::BoundedSequence(BoundedSequenceValueMut::BooleanBoundedSequence(
                mut v,
            )) = view.get_mut("bool_values").unwrap()
            {
                v.try_reset(3).unwrap();
                v.copy_from_slice(&[false, true, false]);
            }
            if let ValueMut::BoundedSequence(BoundedSequenceValueMut::DoubleBoundedSequence(
                mut v,
            )) = view.get_mut("float64_values").unwrap()
            {
                v.try_reset(1).unwrap();
                v.copy_from_slice(&[9.99_f64]);
            }
        }

        let generic = GenericMessage::from(msg.view());
        // The upper bound for BoundedSequences.msg fields is 3
        assert_eq!(
            generic.get("int32_values"),
            Some(&GenericField::BoundedSequence(BoundedSequenceField::Int32(
                vec![100, 200],
                3
            )))
        );
        assert_eq!(
            generic.get("bool_values"),
            Some(&GenericField::BoundedSequence(
                BoundedSequenceField::Boolean(vec![false, true, false], 3)
            ))
        );

        assert_round_trip(&msg);
    }

    // ── Nested messages ───────────────────────────────────────────────────────

    #[test]
    fn test_nested_message_round_trip() {
        let mut msg = make_message("test_msgs", "Nested");
        {
            let mut view = msg.view_mut();
            if let ValueMut::Simple(SimpleValueMut::Message(mut inner)) =
                view.get_mut("basic_types_value").unwrap()
            {
                if let ValueMut::Simple(SimpleValueMut::Int32(v)) =
                    inner.get_mut("int32_value").unwrap()
                {
                    *v = 12345;
                }
                if let ValueMut::Simple(SimpleValueMut::Boolean(v)) =
                    inner.get_mut("bool_value").unwrap()
                {
                    *v = true;
                }
                if let ValueMut::Simple(SimpleValueMut::Double(v)) =
                    inner.get_mut("float64_value").unwrap()
                {
                    *v = -0.5_f64;
                }
            }
        }

        let generic = GenericMessage::from(msg.view());
        if let Some(GenericField::Simple(SimpleField::Message(inner))) =
            generic.get("basic_types_value")
        {
            assert_eq!(
                inner.get("int32_value"),
                Some(&GenericField::Simple(SimpleField::Int32(12345)))
            );
            assert_eq!(
                inner.get("bool_value"),
                Some(&GenericField::Simple(SimpleField::Boolean(true)))
            );
            assert_eq!(
                inner.get("float64_value"),
                Some(&GenericField::Simple(SimpleField::Double(-0.5_f64)))
            );
        } else {
            panic!("Expected 'basic_types_value' to be a nested message");
        }

        assert_round_trip(&msg);
    }

    // ── Error path ────────────────────────────────────────────────────────────

    #[test]
    fn test_try_from_unknown_type_returns_error() {
        let bad_type = InterfaceType {
            package_name: "nonexistent_pkg".to_owned(),
            category: "msg".to_owned(),
            type_name: "FakeMessage".to_owned(),
        };
        let generic = GenericMessage::new(bad_type, IndexMap::new());
        assert!(
            DynamicMessage::try_from(&generic).is_err(),
            "Expected TryFrom to fail for an unknown message type"
        );
    }
}
