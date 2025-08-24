use indexmap::IndexMap;
use rclrs::{
    ArrayValue, BoundedSequenceValue, DynamicMessage, DynamicMessageView, MessageTypeName,
    SequenceValue, SimpleValue, Value,
};
use std::{ops::Index, time::SystemTime};

#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceType {
    pub package_name: String,
    pub catergory: String,
    pub type_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SimpleField {
    Float(f32),
    Double(f64),
    LongDouble([u8; 16]),
    Char(u8),
    WChar(u16),
    Boolean(bool),
    Octet(u8),
    Uint8(u8),
    Int8(i8),
    Uint16(u16),
    Int16(i16),
    Uint32(u32),
    Int32(i32),
    Uint64(u64),
    Int64(i64),
    String(String),
    BoundedString(String),
    WString(String),
    BoundedWString(String),
    Message(GenericMessage),
}

// TODO: Replace Vec with a more appropriate type
#[derive(Debug, Clone, PartialEq)]
pub enum ArrayField {
    Float(Vec<f32>),
    Double(Vec<f64>),
    LongDouble(Vec<[u8; 16]>),
    Char(Vec<u8>),
    WChar(Vec<u16>),
    Boolean(Vec<bool>),
    Octet(Vec<u8>),
    Uint8(Vec<u8>),
    Int8(Vec<i8>),
    Uint16(Vec<u16>),
    Int16(Vec<i16>),
    Uint32(Vec<u32>),
    Int32(Vec<i32>),
    Uint64(Vec<u64>),
    Int64(Vec<i64>),
    String(Vec<String>),
    BoundedString(Vec<String>),
    WString(Vec<String>),
    BoundedWString(Vec<String>),
    Message(Vec<GenericMessage>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SequenceField {
    Float(Vec<f32>),
    Double(Vec<f64>),
    LongDouble(Vec<[u8; 16]>),
    Char(Vec<u8>),
    WChar(Vec<u16>),
    Boolean(Vec<bool>),
    Octet(Vec<u8>),
    Uint8(Vec<u8>),
    Int8(Vec<i8>),
    Uint16(Vec<u16>),
    Int16(Vec<i16>),
    Uint32(Vec<u32>),
    Int32(Vec<i32>),
    Uint64(Vec<u64>),
    Int64(Vec<i64>),
    String(Vec<String>),
    BoundedString(Vec<String>),
    WString(Vec<String>),
    BoundedWString(Vec<String>),
    Message(Vec<GenericMessage>),
}

// TODO: Replace Vec with a more appropriate type
#[derive(Debug, Clone, PartialEq)]
pub enum BoundedSequenceField {
    Float(Vec<f32>),
    Double(Vec<f64>),
    LongDouble(Vec<[u8; 16]>),
    Char(Vec<u8>),
    WChar(Vec<u16>),
    Boolean(Vec<bool>),
    Octet(Vec<u8>),
    Uint8(Vec<u8>),
    Int8(Vec<i8>),
    Uint16(Vec<u16>),
    Int16(Vec<i16>),
    Uint32(Vec<u32>),
    Int32(Vec<i32>),
    Uint64(Vec<u64>),
    Int64(Vec<i64>),
    String(Vec<String>),
    BoundedString(Vec<String>),
    WString(Vec<String>),
    BoundedWString(Vec<String>),
    Message(Vec<GenericMessage>),
}

pub enum AnyTypeMutableRef<'a> {
    Float(&'a mut f32),
    Double(&'a mut f64),
    Uint8(&'a mut u8),
    Int8(&'a mut i8),
    Uint16(&'a mut u16),
    Int16(&'a mut i16),
    Uint32(&'a mut u32),
    Int32(&'a mut i32),
    Uint64(&'a mut u64),
    Int64(&'a mut i64),
    String(&'a mut String),
}

pub trait Length {
    fn len(&self) -> usize;
}

impl Length for ArrayField {
    fn len(&self) -> usize {
        match self {
            ArrayField::Float(v) => v.len(),
            ArrayField::Double(v) => v.len(),
            ArrayField::LongDouble(v) => v.len(),
            ArrayField::Char(v) => v.len(),
            ArrayField::WChar(v) => v.len(),
            ArrayField::Boolean(v) => v.len(),
            ArrayField::Octet(v) => v.len(),
            ArrayField::Uint8(v) => v.len(),
            ArrayField::Int8(v) => v.len(),
            ArrayField::Uint16(v) => v.len(),
            ArrayField::Int16(v) => v.len(),
            ArrayField::Uint32(v) => v.len(),
            ArrayField::Int32(v) => v.len(),
            ArrayField::Uint64(v) => v.len(),
            ArrayField::Int64(v) => v.len(),
            ArrayField::String(v) => v.len(),
            ArrayField::BoundedString(v) => v.len(),
            ArrayField::WString(v) => v.len(),
            ArrayField::BoundedWString(v) => v.len(),
            ArrayField::Message(v) => v.len(),
        }
    }
}

impl Length for SequenceField {
    fn len(&self) -> usize {
        match self {
            SequenceField::Float(v) => v.len(),
            SequenceField::Double(v) => v.len(),
            SequenceField::LongDouble(v) => v.len(),
            SequenceField::Char(v) => v.len(),
            SequenceField::WChar(v) => v.len(),
            SequenceField::Boolean(v) => v.len(),
            SequenceField::Octet(v) => v.len(),
            SequenceField::Uint8(v) => v.len(),
            SequenceField::Int8(v) => v.len(),
            SequenceField::Uint16(v) => v.len(),
            SequenceField::Int16(v) => v.len(),
            SequenceField::Uint32(v) => v.len(),
            SequenceField::Int32(v) => v.len(),
            SequenceField::Uint64(v) => v.len(),
            SequenceField::Int64(v) => v.len(),
            SequenceField::String(v) => v.len(),
            SequenceField::BoundedString(v) => v.len(),
            SequenceField::WString(v) => v.len(),
            SequenceField::BoundedWString(v) => v.len(),
            SequenceField::Message(v) => v.len(),
        }
    }
}

impl Length for BoundedSequenceField {
    fn len(&self) -> usize {
        match self {
            BoundedSequenceField::Float(v) => v.len(),
            BoundedSequenceField::Double(v) => v.len(),
            BoundedSequenceField::LongDouble(v) => v.len(),
            BoundedSequenceField::Char(v) => v.len(),
            BoundedSequenceField::WChar(v) => v.len(),
            BoundedSequenceField::Boolean(v) => v.len(),
            BoundedSequenceField::Octet(v) => v.len(),
            BoundedSequenceField::Uint8(v) => v.len(),
            BoundedSequenceField::Int8(v) => v.len(),
            BoundedSequenceField::Uint16(v) => v.len(),
            BoundedSequenceField::Int16(v) => v.len(),
            BoundedSequenceField::Uint32(v) => v.len(),
            BoundedSequenceField::Int32(v) => v.len(),
            BoundedSequenceField::Uint64(v) => v.len(),
            BoundedSequenceField::Int64(v) => v.len(),
            BoundedSequenceField::String(v) => v.len(),
            BoundedSequenceField::BoundedString(v) => v.len(),
            BoundedSequenceField::WString(v) => v.len(),
            BoundedSequenceField::BoundedWString(v) => v.len(),
            BoundedSequenceField::Message(v) => v.len(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GenericField {
    Simple(SimpleField),
    Array(ArrayField),
    Sequence(SequenceField),
    BoundedSequence(BoundedSequenceField),
}

impl GenericField {
    pub fn get_mut_deep_index(
        &mut self,
        field_index_path: &[usize],
    ) -> Result<AnyTypeMutableRef, String> {
        match self {
            GenericField::Simple(SimpleField::Message(inner_message)) => {
                if field_index_path.is_empty() {
                    return Err("Field index path is empty".to_string());
                }
                return inner_message.get_mut_deep_index(&field_index_path);
            }
            GenericField::Simple(simple_field) => {
                if !field_index_path.is_empty() {
                    return Err("Field index path out of bounds".to_string());
                }
                match simple_field {
                    SimpleField::Float(v) => Ok(AnyTypeMutableRef::Float(v)),
                    SimpleField::Double(v) => Ok(AnyTypeMutableRef::Double(v)),
                    SimpleField::Uint8(v) => Ok(AnyTypeMutableRef::Uint8(v)),
                    SimpleField::Int8(v) => Ok(AnyTypeMutableRef::Int8(v)),
                    SimpleField::Uint16(v) => Ok(AnyTypeMutableRef::Uint16(v)),
                    SimpleField::Int16(v) => Ok(AnyTypeMutableRef::Int16(v)),
                    SimpleField::Uint32(v) => Ok(AnyTypeMutableRef::Uint32(v)),
                    SimpleField::Int32(v) => Ok(AnyTypeMutableRef::Int32(v)),
                    SimpleField::Uint64(v) => Ok(AnyTypeMutableRef::Uint64(v)),
                    SimpleField::Int64(v) => Ok(AnyTypeMutableRef::Int64(v)),
                    SimpleField::String(v) => Ok(AnyTypeMutableRef::String(v)),
                    _ => Err("Unsupported simple field type for mutable reference".to_string()),
                }
            }
            GenericField::Array(ArrayField::Message(msgs)) => {
                if field_index_path.is_empty() {
                    return Err("Field index path is empty".to_string());
                }
                let index = field_index_path[0];
                if index >= msgs.len() {
                    return Err("Index out of bounds".to_string());
                }
                return msgs[index].get_mut_deep_index(&field_index_path[1..]);
            }
            GenericField::Array(array_field) => {
                if field_index_path.is_empty() {
                    return Err("Field index path is empty".to_string());
                }
                let index = field_index_path[0];
                match array_field {
                    ArrayField::Float(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Float(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::Double(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Double(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::Uint8(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Uint8(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::Int8(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Int8(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::Uint16(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Uint16(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::Int16(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Int16(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::Uint32(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Uint32(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::Int32(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Int32(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::Uint64(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Uint64(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::Int64(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Int64(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::String(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::String(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    _ => Err("Unsupported array field type for mutable reference".to_string()),
                }
            }
            _ => Err("Unsupported field type for mutable reference".to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericMessage {
    fields: IndexMap<String, GenericField>,
    type_name: InterfaceType,
}

pub struct MessageMetadata {
    pub received_time: SystemTime,
}

impl From<SimpleValue<'_>> for SimpleField {
    fn from(value: SimpleValue) -> Self {
        match value {
            SimpleValue::Float(v) => SimpleField::Float(*v),
            SimpleValue::Double(v) => SimpleField::Double(*v),
            SimpleValue::LongDouble(v) => SimpleField::LongDouble([0; 16]), // Placeholder, actual value not used),
            SimpleValue::Char(v) => SimpleField::Char(*v),
            SimpleValue::WChar(v) => SimpleField::WChar(*v),
            SimpleValue::Boolean(v) => SimpleField::Boolean(*v),
            SimpleValue::Octet(v) => SimpleField::Octet(*v),
            SimpleValue::Uint8(v) => SimpleField::Uint8(*v),
            SimpleValue::Int8(v) => SimpleField::Int8(*v),
            SimpleValue::Uint16(v) => SimpleField::Uint16(*v),
            SimpleValue::Int16(v) => SimpleField::Int16(*v),
            SimpleValue::Uint32(v) => SimpleField::Uint32(*v),
            SimpleValue::Int32(v) => SimpleField::Int32(*v),
            SimpleValue::Uint64(v) => SimpleField::Uint64(*v),
            SimpleValue::Int64(v) => SimpleField::Int64(*v),
            SimpleValue::String(s) => SimpleField::String(s.to_string()),
            SimpleValue::BoundedString(s) => SimpleField::BoundedString(s.to_string()),
            SimpleValue::WString(s) => SimpleField::WString(s.to_string()),
            SimpleValue::BoundedWString(s) => SimpleField::BoundedWString(s.to_string()),
            SimpleValue::Message(msg) => SimpleField::Message(GenericMessage::from(&msg)),
        }
    }
}

impl From<ArrayValue<'_>> for ArrayField {
    fn from(value: ArrayValue) -> Self {
        match value {
            ArrayValue::FloatArray(v) => ArrayField::Float(v.to_vec()),
            ArrayValue::DoubleArray(v) => ArrayField::Double(v.to_vec()),
            ArrayValue::LongDoubleArray(v, size) => ArrayField::LongDouble([].to_vec()), // Placeholder, actual value not used
            ArrayValue::CharArray(v) => ArrayField::Char(v.to_vec()),
            ArrayValue::WCharArray(v) => ArrayField::WChar(v.to_vec()),
            ArrayValue::BooleanArray(v) => ArrayField::Boolean(v.to_vec()),
            ArrayValue::OctetArray(v) => ArrayField::Octet(v.to_vec()),
            ArrayValue::Uint8Array(v) => ArrayField::Uint8(v.to_vec()),
            ArrayValue::Int8Array(v) => ArrayField::Int8(v.to_vec()),
            ArrayValue::Uint16Array(v) => ArrayField::Uint16(v.to_vec()),
            ArrayValue::Int16Array(v) => ArrayField::Int16(v.to_vec()),
            ArrayValue::Uint32Array(v) => ArrayField::Uint32(v.to_vec()),
            ArrayValue::Int32Array(v) => ArrayField::Int32(v.to_vec()),
            ArrayValue::Uint64Array(v) => ArrayField::Uint64(v.to_vec()),
            ArrayValue::Int64Array(v) => ArrayField::Int64(v.to_vec()),
            ArrayValue::StringArray(s) => {
                ArrayField::String(s.iter().map(|s| s.to_string()).collect())
            }
            ArrayValue::BoundedStringArray(s) => {
                ArrayField::BoundedString(s.iter().map(|s| s.to_string()).collect())
            }
            ArrayValue::WStringArray(s) => {
                ArrayField::WString(s.iter().map(|s| s.to_string()).collect())
            }
            ArrayValue::BoundedWStringArray(s) => {
                ArrayField::BoundedWString(s.iter().map(|s| s.to_string()).collect())
            }
            ArrayValue::MessageArray(msgs) => {
                let generic_msgs = msgs.iter().map(GenericMessage::from).collect();
                ArrayField::Message(generic_msgs)
            }
        }
    }
}

impl From<SequenceValue<'_>> for SequenceField {
    fn from(value: SequenceValue) -> Self {
        match value {
            SequenceValue::FloatSequence(v) => SequenceField::Float(v.to_vec()),
            SequenceValue::DoubleSequence(v) => SequenceField::Double(v.to_vec()),
            SequenceValue::LongDoubleSequence(v) => SequenceField::LongDouble([].to_vec()), // Placeholder, actual value not used
            SequenceValue::CharSequence(v) => SequenceField::Char(v.to_vec()),
            SequenceValue::WCharSequence(v) => SequenceField::WChar(v.to_vec()),
            SequenceValue::BooleanSequence(v) => SequenceField::Boolean(v.to_vec()),
            SequenceValue::OctetSequence(v) => SequenceField::Octet(v.to_vec()),
            SequenceValue::Uint8Sequence(v) => SequenceField::Uint8(v.to_vec()),
            SequenceValue::Int8Sequence(v) => SequenceField::Int8(v.to_vec()),
            SequenceValue::Uint16Sequence(v) => SequenceField::Uint16(v.to_vec()),
            SequenceValue::Int16Sequence(v) => SequenceField::Int16(v.to_vec()),
            SequenceValue::Uint32Sequence(v) => SequenceField::Uint32(v.to_vec()),
            SequenceValue::Int32Sequence(v) => SequenceField::Int32(v.to_vec()),
            SequenceValue::Uint64Sequence(v) => SequenceField::Uint64(v.to_vec()),
            SequenceValue::Int64Sequence(v) => SequenceField::Int64(v.to_vec()),
            SequenceValue::StringSequence(s) => {
                SequenceField::String(s.iter().map(|s| s.to_string()).collect())
            }
            SequenceValue::BoundedStringSequence(s) => {
                SequenceField::BoundedString(s.iter().map(|s| s.to_string()).collect())
            }
            SequenceValue::WStringSequence(s) => {
                SequenceField::WString(s.iter().map(|s| s.to_string()).collect())
            }
            SequenceValue::BoundedWStringSequence(s) => {
                SequenceField::BoundedWString(s.iter().map(|s| s.to_string()).collect())
            }
            SequenceValue::MessageSequence(msgs) => {
                let generic_msgs = msgs.into_iter().map(GenericMessage::from).collect();
                SequenceField::Message(generic_msgs)
            }
        }
    }
}

impl From<BoundedSequenceValue<'_>> for BoundedSequenceField {
    fn from(value: BoundedSequenceValue) -> Self {
        match value {
            BoundedSequenceValue::FloatBoundedSequence(v) => {
                BoundedSequenceField::Float(v.to_vec())
            }
            BoundedSequenceValue::DoubleBoundedSequence(v) => {
                BoundedSequenceField::Double(v.to_vec())
            }
            BoundedSequenceValue::LongDoubleBoundedSequence(v, size) => {
                BoundedSequenceField::LongDouble([].to_vec()) // Placeholder, actual value not used
            }
            BoundedSequenceValue::CharBoundedSequence(v) => BoundedSequenceField::Char(v.to_vec()),
            BoundedSequenceValue::WCharBoundedSequence(v) => {
                BoundedSequenceField::WChar(v.to_vec())
            }
            BoundedSequenceValue::BooleanBoundedSequence(v) => {
                BoundedSequenceField::Boolean(v.to_vec())
            }
            BoundedSequenceValue::OctetBoundedSequence(v) => {
                BoundedSequenceField::Octet(v.to_vec())
            }
            BoundedSequenceValue::Uint8BoundedSequence(v) => {
                BoundedSequenceField::Uint8(v.to_vec())
            }
            BoundedSequenceValue::Int8BoundedSequence(v) => BoundedSequenceField::Int8(v.to_vec()),
            BoundedSequenceValue::Uint16BoundedSequence(v) => {
                BoundedSequenceField::Uint16(v.to_vec())
            }
            BoundedSequenceValue::Int16BoundedSequence(v) => {
                BoundedSequenceField::Int16(v.to_vec())
            }
            BoundedSequenceValue::Uint32BoundedSequence(v) => {
                BoundedSequenceField::Uint32(v.to_vec())
            }
            BoundedSequenceValue::Int32BoundedSequence(v) => {
                BoundedSequenceField::Int32(v.to_vec())
            }
            BoundedSequenceValue::Uint64BoundedSequence(v) => {
                BoundedSequenceField::Uint64(v.to_vec())
            }
            BoundedSequenceValue::Int64BoundedSequence(v) => {
                BoundedSequenceField::Int64(v.to_vec())
            }
            BoundedSequenceValue::StringBoundedSequence(s) => {
                BoundedSequenceField::String(s.iter().map(|s| s.to_string()).collect())
            }
            BoundedSequenceValue::BoundedStringBoundedSequence(s) => {
                BoundedSequenceField::BoundedString(s.iter().map(|s| s.to_string()).collect())
            }
            BoundedSequenceValue::WStringBoundedSequence(s) => {
                BoundedSequenceField::WString(s.iter().map(|s| s.to_string()).collect())
            }
            BoundedSequenceValue::BoundedWStringBoundedSequence(s) => {
                BoundedSequenceField::BoundedWString(s.iter().map(|s| s.to_string()).collect())
            }
            BoundedSequenceValue::MessageBoundedSequence(msgs) => {
                let generic_msgs = msgs.into_iter().map(GenericMessage::from).collect();
                BoundedSequenceField::Message(generic_msgs)
            }
        }
    }
}

impl From<Value<'_>> for GenericField {
    fn from(value: Value) -> Self {
        match value {
            Value::Simple(simple_value) => GenericField::Simple(simple_value.into()),
            Value::Array(array_value) => GenericField::Array(array_value.into()),
            Value::Sequence(sequence_value) => GenericField::Sequence(sequence_value.into()),
            Value::BoundedSequence(bounded_sequence_value) => {
                GenericField::BoundedSequence(bounded_sequence_value.into())
            }
        }
    }
}

impl From<&DynamicMessageView<'_>> for GenericMessage {
    fn from(message: &DynamicMessageView) -> Self {
        let mut fields = IndexMap::new();
        for field in &message.fields {
            let value = message.get(&field.name).unwrap();
            let generic_field = match value {
                Value::Simple(simple_value) => GenericField::Simple(simple_value.into()),
                Value::Array(array_value) => GenericField::Array(array_value.into()),
                Value::Sequence(sequence_value) => GenericField::Sequence(sequence_value.into()),
                Value::BoundedSequence(bounded_sequence_value) => {
                    GenericField::BoundedSequence(bounded_sequence_value.into())
                }
            };
            fields.insert(field.name.to_string(), generic_field);
        }
        Self {
            fields,
            type_name: InterfaceType {
                package_name: message
                    .namespace
                    .split("__")
                    .next()
                    .unwrap_or("")
                    .to_string(),
                catergory: message
                    .namespace
                    .split("__")
                    .nth(1)
                    .unwrap_or("")
                    .to_string(),
                type_name: message.type_name.clone(),
            },
        }
    }
}

impl From<DynamicMessageView<'_>> for GenericMessage {
    fn from(message: DynamicMessageView) -> Self {
        GenericMessage::from(&message)
    }
}

impl Index<&str> for GenericMessage {
    type Output = GenericField;

    fn index(&self, index: &str) -> &GenericField {
        self.fields.get(index).expect("Field not found")
    }
}

impl Index<usize> for GenericMessage {
    type Output = GenericField;

    fn index(&self, index: usize) -> &GenericField {
        self.fields
            .get_index(index)
            .map(|(_, field)| field)
            .expect("Index out of bounds")
    }
}

impl GenericMessage {
    pub fn type_name(&self) -> &InterfaceType {
        &self.type_name
    }

    pub fn fields(&self) -> &IndexMap<String, GenericField> {
        &self.fields
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &GenericField)> {
        self.fields.iter()
    }

    pub fn get(&self, field_name: &str) -> Option<&GenericField> {
        self.fields.get(field_name)
    }

    pub fn get_index(&self, index: usize) -> Option<&GenericField> {
        self.fields.get_index(index).map(|(_, field)| field)
    }

    pub fn get_mut_deep_index(
        &mut self,
        field_index_path: &[usize],
    ) -> Result<AnyTypeMutableRef, String> {
        if field_index_path.is_empty() {
            return Err("Field index path is empty".to_string());
        }
        let index = field_index_path[0];
        let field = self
            .fields
            .get_index_mut(index)
            .map(|(_, field)| field)
            .ok_or_else(|| "Index out of bounds".to_string())?;
        field.get_mut_deep_index(&field_index_path[1..])
    }

    pub fn field_count(&self) -> usize {
        self.fields.len()
    }
}

#[cfg(test)]
mod test {
    use rclrs::{SimpleValueMut, ValueMut};

    use super::*;

    #[test]
    fn test_generic_message_from_dynamic_message() {
        let mut message = DynamicMessage::new("std_msgs/msg/String".try_into().unwrap()).unwrap();
        if let ValueMut::Simple(SimpleValueMut::String(data)) = message.get_mut("data").unwrap() {
            *data = rosidl_runtime_rs::String::from("Hello, ROS2!");
        }
        let generic_message = GenericMessage::from(message.view());
        assert_eq!(
            generic_message.type_name,
            InterfaceType {
                package_name: "std_msgs".to_string(),
                catergory: "msg".to_string(),
                type_name: "String".to_string(),
            }
        );
        assert_eq!(generic_message.fields.len(), 1);
        assert_eq!(
            generic_message["data"],
            GenericField::Simple(SimpleField::String("Hello, ROS2!".to_string()))
        );
    }
}
