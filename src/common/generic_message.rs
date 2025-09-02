use indexmap::IndexMap;
use rclrs::{
    ArrayValue, BoundedSequenceValue, DynamicMessageView, SequenceValue, SimpleValue, Value,
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
    Float(Vec<f32>, usize),
    Double(Vec<f64>, usize),
    LongDouble(Vec<[u8; 16]>, usize),
    Char(Vec<u8>, usize),
    WChar(Vec<u16>, usize),
    Boolean(Vec<bool>, usize),
    Octet(Vec<u8>, usize),
    Uint8(Vec<u8>, usize),
    Int8(Vec<i8>, usize),
    Uint16(Vec<u16>, usize),
    Int16(Vec<i16>, usize),
    Uint32(Vec<u32>, usize),
    Int32(Vec<i32>, usize),
    Uint64(Vec<u64>, usize),
    Int64(Vec<i64>, usize),
    String(Vec<String>, usize),
    BoundedString(Vec<String>, usize),
    WString(Vec<String>, usize),
    BoundedWString(Vec<String>, usize),
    Message(Vec<GenericMessage>, usize),
}

pub enum AnyTypeMutableRef<'a> {
    Float(&'a mut f32),
    Double(&'a mut f64),
    Boolean(&'a mut bool),
    Uint8(&'a mut u8),
    Int8(&'a mut i8),
    Uint16(&'a mut u16),
    Int16(&'a mut i16),
    Uint32(&'a mut u32),
    Int32(&'a mut i32),
    Uint64(&'a mut u64),
    Int64(&'a mut i64),
    String(&'a mut String),
    Array(&'a mut ArrayField),
    Sequence(&'a mut SequenceField),
    BoundedSequence(&'a mut BoundedSequenceField),
}

pub enum AnyTypeRef<'a> {
    Float(&'a f32),
    Double(&'a f64),
    Boolean(&'a bool),
    Uint8(&'a u8),
    Int8(&'a i8),
    Uint16(&'a u16),
    Int16(&'a i16),
    Uint32(&'a u32),
    Int32(&'a i32),
    Uint64(&'a u64),
    Int64(&'a i64),
    String(&'a String),
    Array(&'a ArrayField),
    Sequence(&'a SequenceField),
    BoundedSequence(&'a BoundedSequenceField),
}

pub enum FieldType {
    Float,
    Double,
    Boolean,
    Uint8,
    Int8,
    Uint16,
    Int16,
    Uint32,
    Int32,
    Uint64,
    Int64,
    String,
    Message,
    Array,
    Sequence,
    BoundedSequence,
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
            BoundedSequenceField::Float(v, _) => v.len(),
            BoundedSequenceField::Double(v, _) => v.len(),
            BoundedSequenceField::LongDouble(v, _) => v.len(),
            BoundedSequenceField::Char(v, _) => v.len(),
            BoundedSequenceField::WChar(v, _) => v.len(),
            BoundedSequenceField::Boolean(v, _) => v.len(),
            BoundedSequenceField::Octet(v, _) => v.len(),
            BoundedSequenceField::Uint8(v, _) => v.len(),
            BoundedSequenceField::Int8(v, _) => v.len(),
            BoundedSequenceField::Uint16(v, _) => v.len(),
            BoundedSequenceField::Int16(v, _) => v.len(),
            BoundedSequenceField::Uint32(v, _) => v.len(),
            BoundedSequenceField::Int32(v, _) => v.len(),
            BoundedSequenceField::Uint64(v, _) => v.len(),
            BoundedSequenceField::Int64(v, _) => v.len(),
            BoundedSequenceField::String(v, _) => v.len(),
            BoundedSequenceField::BoundedString(v, _) => v.len(),
            BoundedSequenceField::WString(v, _) => v.len(),
            BoundedSequenceField::BoundedWString(v, _) => v.len(),
            BoundedSequenceField::Message(v, _) => v.len(),
        }
    }
}

impl SequenceField {
    pub fn resize(&mut self, new_size: usize) {
        match self {
            SequenceField::Float(v) => v.resize(new_size, 0.0),
            SequenceField::Double(v) => v.resize(new_size, 0.0),
            SequenceField::LongDouble(v) => v.resize(new_size, [0u8; 16]),
            SequenceField::Char(v) => v.resize(new_size, 0),
            SequenceField::WChar(v) => v.resize(new_size, 0),
            SequenceField::Boolean(v) => v.resize(new_size, false),
            SequenceField::Octet(v) => v.resize(new_size, 0),
            SequenceField::Uint8(v) => v.resize(new_size, 0),
            SequenceField::Int8(v) => v.resize(new_size, 0),
            SequenceField::Uint16(v) => v.resize(new_size, 0),
            SequenceField::Int16(v) => v.resize(new_size, 0),
            SequenceField::Uint32(v) => v.resize(new_size, 0),
            SequenceField::Int32(v) => v.resize(new_size, 0),
            SequenceField::Uint64(v) => v.resize(new_size, 0),
            SequenceField::Int64(v) => v.resize(new_size, 0),
            SequenceField::String(v) => v.resize(new_size, "".to_string()),
            SequenceField::BoundedString(v) => v.resize(new_size, "".to_string()),
            SequenceField::WString(v) => v.resize(new_size, "".to_string()),
            SequenceField::BoundedWString(v) => v.resize(new_size, "".to_string()),
            SequenceField::Message(_) => todo!("Implement resize for Message sequence"),
        }
    }
}

impl BoundedSequenceField {
    pub fn resize(&mut self, new_size: usize) {
        match self {
            BoundedSequenceField::Float(v, max_size) => {
                if new_size <= *max_size {
                    v.resize(new_size, 0.0)
                }
            }
            BoundedSequenceField::Double(v, max_size) => {
                if new_size <= *max_size {
                    v.resize(new_size, 0.0)
                }
            }
            BoundedSequenceField::LongDouble(v, max_size) => {
                if new_size <= *max_size {
                    v.resize(new_size, [0u8; 16])
                }
            }
            BoundedSequenceField::Char(v, max_size) => {
                if new_size <= *max_size {
                    v.resize(new_size, 0)
                }
            }
            BoundedSequenceField::WChar(v, max_size) => {
                if new_size <= *max_size {
                    v.resize(new_size, 0)
                }
            }
            BoundedSequenceField::Boolean(v, max_size) => {
                if new_size <= *max_size {
                    v.resize(new_size, false)
                }
            }
            BoundedSequenceField::Octet(v, max_size) => {
                if new_size <= *max_size {
                    v.resize(new_size, 0)
                }
            }
            BoundedSequenceField::Uint8(v, max_size) => {
                if new_size <= *max_size {
                    v.resize(new_size, 0)
                }
            }
            BoundedSequenceField::Int8(v, max_size) => {
                if new_size <= *max_size {
                    v.resize(new_size, 0)
                }
            }
            BoundedSequenceField::Uint16(v, max_size) => {
                if new_size <= *max_size {
                    v.resize(new_size, 0)
                }
            }
            BoundedSequenceField::Int16(v, max_size) => {
                if new_size <= *max_size {
                    v.resize(new_size, 0)
                }
            }
            BoundedSequenceField::Uint32(v, max_size) => {
                if new_size <= *max_size {
                    v.resize(new_size, 0)
                }
            }
            BoundedSequenceField::Int32(v, max_size) => {
                if new_size <= *max_size {
                    v.resize(new_size, 0)
                }
            }
            BoundedSequenceField::Uint64(v, max_size) => {
                if new_size <= *max_size {
                    v.resize(new_size, 0)
                }
            }
            BoundedSequenceField::Int64(v, max_size) => {
                if new_size <= *max_size {
                    v.resize(new_size, 0)
                }
            }
            BoundedSequenceField::String(v, max_size) => {
                if new_size <= *max_size {
                    v.resize(new_size, "".to_string())
                }
            }
            BoundedSequenceField::BoundedString(v, max_size) => {
                if new_size <= *max_size {
                    v.resize(new_size, "".to_string())
                }
            }
            BoundedSequenceField::WString(v, max_size) => {
                if new_size <= *max_size {
                    v.resize(new_size, "".to_string())
                }
            }
            BoundedSequenceField::BoundedWString(v, max_size) => {
                if new_size <= *max_size {
                    v.resize(new_size, "".to_string())
                }
            }
            BoundedSequenceField::Message(_, _) => {
                todo!("Implement resize for Message bounded sequence")
            }
        }
    }

    pub fn max_len(&self) -> usize {
        match self {
            BoundedSequenceField::Float(_, max_size) => *max_size,
            BoundedSequenceField::Double(_, max_size) => *max_size,
            BoundedSequenceField::LongDouble(_, max_size) => *max_size,
            BoundedSequenceField::Char(_, max_size) => *max_size,
            BoundedSequenceField::WChar(_, max_size) => *max_size,
            BoundedSequenceField::Boolean(_, max_size) => *max_size,
            BoundedSequenceField::Octet(_, max_size) => *max_size,
            BoundedSequenceField::Uint8(_, max_size) => *max_size,
            BoundedSequenceField::Int8(_, max_size) => *max_size,
            BoundedSequenceField::Uint16(_, max_size) => *max_size,
            BoundedSequenceField::Int16(_, max_size) => *max_size,
            BoundedSequenceField::Uint32(_, max_size) => *max_size,
            BoundedSequenceField::Int32(_, max_size) => *max_size,
            BoundedSequenceField::Uint64(_, max_size) => *max_size,
            BoundedSequenceField::Int64(_, max_size) => *max_size,
            BoundedSequenceField::String(_, max_size) => *max_size,
            BoundedSequenceField::BoundedString(_, max_size) => *max_size,
            BoundedSequenceField::WString(_, max_size) => *max_size,
            BoundedSequenceField::BoundedWString(_, max_size) => *max_size,
            BoundedSequenceField::Message(_, max_size) => *max_size,
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
    pub fn get_deep_index(&self, field_index_path: &[usize]) -> Result<AnyTypeRef, String> {
        match self {
            GenericField::Simple(SimpleField::Message(inner_message)) => {
                if field_index_path.is_empty() {
                    return Err("Field index path is empty".to_string());
                }
                return inner_message.get_deep_index(&field_index_path);
            }
            GenericField::Simple(simple_field) => {
                if !field_index_path.is_empty() {
                    return Err("Field index path out of bounds".to_string());
                }
                match simple_field {
                    SimpleField::Float(v) => Ok(AnyTypeRef::Float(v)),
                    SimpleField::Double(v) => Ok(AnyTypeRef::Double(v)),
                    SimpleField::LongDouble(_) => Err("LongDouble not supported".to_string()),
                    SimpleField::Char(v) => Ok(AnyTypeRef::Uint8(v)),
                    SimpleField::WChar(v) => Ok(AnyTypeRef::Uint16(v)),
                    SimpleField::Boolean(v) => Ok(AnyTypeRef::Boolean(v)),
                    SimpleField::Octet(v) => Ok(AnyTypeRef::Uint8(v)),
                    SimpleField::Uint8(v) => Ok(AnyTypeRef::Uint8(v)),
                    SimpleField::Int8(v) => Ok(AnyTypeRef::Int8(v)),
                    SimpleField::Uint16(v) => Ok(AnyTypeRef::Uint16(v)),
                    SimpleField::Int16(v) => Ok(AnyTypeRef::Int16(v)),
                    SimpleField::Uint32(v) => Ok(AnyTypeRef::Uint32(v)),
                    SimpleField::Int32(v) => Ok(AnyTypeRef::Int32(v)),
                    SimpleField::Uint64(v) => Ok(AnyTypeRef::Uint64(v)),
                    SimpleField::Int64(v) => Ok(AnyTypeRef::Int64(v)),
                    SimpleField::String(v) => Ok(AnyTypeRef::String(v)),
                    SimpleField::BoundedString(v) => Ok(AnyTypeRef::String(v)),
                    SimpleField::WString(v) => Ok(AnyTypeRef::String(v)),
                    SimpleField::BoundedWString(v) => Ok(AnyTypeRef::String(v)),
                    SimpleField::Message(v) => Err("Message not supported".to_string()),
                }
            }
            GenericField::Array(array_field) => {
                let index = match field_index_path.first() {
                    Some(i) => *i,
                    None => return Ok(AnyTypeRef::Array(array_field)),
                };
                match array_field {
                    ArrayField::Float(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Float(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::Double(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Double(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::LongDouble(_) => Err("LongDouble not supported".to_string()),
                    ArrayField::Char(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Uint8(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::WChar(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Uint16(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::Boolean(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Boolean(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::Octet(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Uint8(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::Uint8(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Uint8(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::Int8(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Int8(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::Uint16(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Uint16(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::Int16(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Int16(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::Uint32(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Uint32(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::Int32(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Int32(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::Uint64(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Uint64(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::Int64(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Int64(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::String(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::String(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::BoundedString(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::String(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::WString(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::String(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::BoundedWString(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::String(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::Message(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return v[index].get_deep_index(&field_index_path[1..]);
                    }
                }
            }
            GenericField::Sequence(sequence_field) => {
                let index = match field_index_path.first() {
                    Some(i) => *i,
                    None => return Ok(AnyTypeRef::Sequence(sequence_field)),
                };
                match sequence_field {
                    SequenceField::Float(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Float(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::Double(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Double(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::LongDouble(_) => Err("LongDouble not supported".to_string()),
                    SequenceField::Char(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Uint8(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::WChar(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Uint16(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::Boolean(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Boolean(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::Octet(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Uint8(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::Uint8(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Uint8(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::Int8(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Int8(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::Uint16(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Uint16(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::Int16(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Int16(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::Uint32(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Uint32(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::Int32(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Int32(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::Uint64(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Uint64(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::Int64(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Int64(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::String(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::String(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::BoundedString(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::String(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::WString(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::String(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::BoundedWString(v) => v
                        .get(index)
                        .map(|val| AnyTypeRef::String(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::Message(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return v[index].get_deep_index(&field_index_path[1..]);
                    }
                }
            }
            GenericField::BoundedSequence(sequence_field) => {
                let index = match field_index_path.first() {
                    Some(i) => *i,
                    None => return Ok(AnyTypeRef::BoundedSequence(sequence_field)),
                };
                match sequence_field {
                    BoundedSequenceField::Float(v, _) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Float(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::Double(v, _) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Double(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::LongDouble(_, _) => {
                        Err("LongDouble not supported".to_string())
                    }
                    BoundedSequenceField::Char(v, _) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Uint8(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::WChar(v, _) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Uint16(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::Boolean(v, _) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Boolean(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::Octet(v, _) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Uint8(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::Uint8(v, _) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Uint8(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::Int8(v, _) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Int8(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::Uint16(v, _) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Uint16(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::Int16(v, _) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Int16(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::Uint32(v, _) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Uint32(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::Int32(v, _) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Int32(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::Uint64(v, _) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Uint64(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::Int64(v, _) => v
                        .get(index)
                        .map(|val| AnyTypeRef::Int64(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::String(v, _) => v
                        .get(index)
                        .map(|val| AnyTypeRef::String(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::BoundedString(v, _) => v
                        .get(index)
                        .map(|val| AnyTypeRef::String(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::WString(v, _) => v
                        .get(index)
                        .map(|val| AnyTypeRef::String(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::BoundedWString(v, _) => v
                        .get(index)
                        .map(|val| AnyTypeRef::String(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::Message(v, _) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return v[index].get_deep_index(&field_index_path[1..]);
                    }
                }
            }
        }
    }

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
                    SimpleField::Boolean(v) => Ok(AnyTypeMutableRef::Boolean(v)),
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
            GenericField::Array(array_field) => {
                let index = match field_index_path.first() {
                    Some(i) => *i,
                    None => return Ok(AnyTypeMutableRef::Array(array_field)),
                };
                match array_field {
                    ArrayField::Float(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Float(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::Double(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Double(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    ArrayField::Boolean(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Boolean(val))
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
                    ArrayField::Message(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return v[index].get_mut_deep_index(&field_index_path[1..]);
                    }
                    _ => Err("Unsupported array field type for mutable reference".to_string()),
                }
            }
            GenericField::Sequence(sequence_field) => {
                let index = match field_index_path.first() {
                    Some(i) => *i,
                    None => return Ok(AnyTypeMutableRef::Sequence(sequence_field)),
                };
                match sequence_field {
                    SequenceField::Float(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Float(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::Double(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Double(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::Boolean(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Boolean(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::Uint8(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Uint8(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::Int8(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Int8(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::Uint16(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Uint16(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::Int16(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Int16(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::Uint32(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Uint32(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::Int32(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Int32(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::Uint64(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Uint64(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::Int64(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Int64(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::String(v) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::String(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    SequenceField::Message(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return v[index].get_mut_deep_index(&field_index_path[1..]);
                    }
                    _ => Err("Unsupported array field type for mutable reference".to_string()),
                }
            }
            GenericField::BoundedSequence(sequence_field) => {
                let index = match field_index_path.first() {
                    Some(i) => *i,
                    None => return Ok(AnyTypeMutableRef::BoundedSequence(sequence_field)),
                };
                match sequence_field {
                    BoundedSequenceField::Float(v, _) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Float(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::Double(v, _) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Double(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::Boolean(v, _) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Boolean(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::Uint8(v, _) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Uint8(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::Int8(v, _) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Int8(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::Uint16(v, _) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Uint16(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::Int16(v, _) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Int16(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::Uint32(v, _) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Uint32(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::Int32(v, _) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Int32(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::Uint64(v, _) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Uint64(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::Int64(v, _) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::Int64(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::String(v, _) => v
                        .get_mut(index)
                        .map(|val| AnyTypeMutableRef::String(val))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    BoundedSequenceField::Message(v, _) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return v[index].get_mut_deep_index(&field_index_path[1..]);
                    }
                    _ => Err("Unsupported array field type for mutable reference".to_string()),
                }
            }
        }
    }

    pub fn get_field_type(&self, field_index_path: &[usize]) -> Result<FieldType, String> {
        if field_index_path.is_empty() {
            return Err("Field index path is empty".to_string());
        }
        let index = field_index_path[0];
        match self {
            GenericField::Simple(simple_field) => {
                if !field_index_path.is_empty() {
                    return Err("Field index path out of bounds".to_string());
                }
                match simple_field {
                    SimpleField::Float(_) => Ok(FieldType::Float),
                    SimpleField::Double(_) => Ok(FieldType::Double),
                    SimpleField::Boolean(_) => Ok(FieldType::Boolean),
                    SimpleField::Uint8(_) => Ok(FieldType::Uint8),
                    SimpleField::Int8(_) => Ok(FieldType::Int8),
                    SimpleField::Uint16(_) => Ok(FieldType::Uint16),
                    SimpleField::Int16(_) => Ok(FieldType::Int16),
                    SimpleField::Uint32(_) => Ok(FieldType::Uint32),
                    SimpleField::Int32(_) => Ok(FieldType::Int32),
                    SimpleField::Uint64(_) => Ok(FieldType::Uint64),
                    SimpleField::Int64(_) => Ok(FieldType::Int64),
                    SimpleField::String(_) => Ok(FieldType::String),
                    SimpleField::Message(_) => Ok(FieldType::Message),
                    _ => Err("Unsupported simple field type".to_string()),
                }
            }
            GenericField::Array(array_field) => {
                if field_index_path.len() == 1 {
                    return Ok(FieldType::Array);
                }
                match array_field {
                    ArrayField::Message(msgs) => {
                        if index >= msgs.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return msgs[index].get_field_type(&field_index_path[1..]);
                    }
                    ArrayField::Float(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Float);
                    }
                    ArrayField::Double(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Double);
                    }
                    ArrayField::Boolean(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Boolean);
                    }
                    ArrayField::Uint8(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Uint8);
                    }
                    ArrayField::Int8(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Int8);
                    }
                    ArrayField::Uint16(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Uint16);
                    }
                    ArrayField::Int16(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Int16);
                    }
                    ArrayField::Uint32(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Uint32);
                    }
                    ArrayField::Int32(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Int32);
                    }
                    ArrayField::Uint64(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Uint64);
                    }
                    ArrayField::Int64(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Int64);
                    }
                    ArrayField::String(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::String);
                    }
                    _ => Err("Unsupported array field type".to_string()),
                }
            }
            GenericField::Sequence(sequence_field) => {
                if field_index_path.len() == 1 {
                    return Ok(FieldType::Sequence);
                }
                match sequence_field {
                    SequenceField::Message(msgs) => {
                        if index >= msgs.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return msgs[index].get_field_type(&field_index_path[1..]);
                    }
                    SequenceField::Float(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Float);
                    }
                    SequenceField::Double(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Double);
                    }
                    SequenceField::Boolean(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Boolean);
                    }
                    SequenceField::Uint8(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Uint8);
                    }
                    SequenceField::Int8(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Int8);
                    }
                    SequenceField::Uint16(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Uint16);
                    }
                    SequenceField::Int16(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Int16);
                    }
                    SequenceField::Uint32(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Uint32);
                    }
                    SequenceField::Int32(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Int32);
                    }
                    SequenceField::Uint64(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Uint64);
                    }
                    SequenceField::Int64(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Int64);
                    }
                    SequenceField::String(v) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::String);
                    }
                    _ => Err("Unsupported sequence field type".to_string()),
                }
            }
            GenericField::BoundedSequence(bounded_sequence_field) => {
                if field_index_path.len() == 1 {
                    return Ok(FieldType::BoundedSequence);
                }
                match bounded_sequence_field {
                    BoundedSequenceField::Message(msgs, _) => {
                        if index >= msgs.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return msgs[index].get_field_type(&field_index_path[1..]);
                    }
                    BoundedSequenceField::Float(v, _) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Float);
                    }
                    BoundedSequenceField::Double(v, _) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Double);
                    }
                    BoundedSequenceField::Boolean(v, _) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Boolean);
                    }
                    BoundedSequenceField::Uint8(v, _) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Uint8);
                    }
                    BoundedSequenceField::Int8(v, _) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Int8);
                    }
                    BoundedSequenceField::Uint16(v, _) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Uint16);
                    }
                    BoundedSequenceField::Int16(v, _) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Int16);
                    }
                    BoundedSequenceField::Uint32(v, _) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Uint32);
                    }
                    BoundedSequenceField::Int32(v, _) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Int32);
                    }
                    BoundedSequenceField::Uint64(v, _) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Uint64);
                    }
                    BoundedSequenceField::Int64(v, _) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::Int64);
                    }
                    BoundedSequenceField::String(v, _) => {
                        if index >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        return Ok(FieldType::String);
                    }
                    _ => Err("Unsupported array field type".to_string()),
                }
            }
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
            SimpleValue::LongDouble(_) => SimpleField::LongDouble([0; 16]), // Placeholder, actual value not used),
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
            ArrayValue::LongDoubleArray(_, _) => ArrayField::LongDouble([].to_vec()), // Placeholder, actual value not used
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
            SequenceValue::LongDoubleSequence(_) => SequenceField::LongDouble([].to_vec()), // Placeholder, actual value not used
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
                BoundedSequenceField::Float(v.to_vec(), v.upper_bound())
            }
            BoundedSequenceValue::DoubleBoundedSequence(v) => {
                BoundedSequenceField::Double(v.to_vec(), v.upper_bound())
            }
            BoundedSequenceValue::LongDoubleBoundedSequence(_, _) => {
                BoundedSequenceField::LongDouble([].to_vec(), 0) // Placeholder, actual value not used
            }
            BoundedSequenceValue::CharBoundedSequence(v) => {
                BoundedSequenceField::Char(v.to_vec(), v.upper_bound())
            }
            BoundedSequenceValue::WCharBoundedSequence(v) => {
                BoundedSequenceField::WChar(v.to_vec(), v.upper_bound())
            }
            BoundedSequenceValue::BooleanBoundedSequence(v) => {
                BoundedSequenceField::Boolean(v.to_vec(), v.upper_bound())
            }
            BoundedSequenceValue::OctetBoundedSequence(v) => {
                BoundedSequenceField::Octet(v.to_vec(), v.upper_bound())
            }
            BoundedSequenceValue::Uint8BoundedSequence(v) => {
                BoundedSequenceField::Uint8(v.to_vec(), v.upper_bound())
            }
            BoundedSequenceValue::Int8BoundedSequence(v) => {
                BoundedSequenceField::Int8(v.to_vec(), v.upper_bound())
            }
            BoundedSequenceValue::Uint16BoundedSequence(v) => {
                BoundedSequenceField::Uint16(v.to_vec(), v.upper_bound())
            }
            BoundedSequenceValue::Int16BoundedSequence(v) => {
                BoundedSequenceField::Int16(v.to_vec(), v.upper_bound())
            }
            BoundedSequenceValue::Uint32BoundedSequence(v) => {
                BoundedSequenceField::Uint32(v.to_vec(), v.upper_bound())
            }
            BoundedSequenceValue::Int32BoundedSequence(v) => {
                BoundedSequenceField::Int32(v.to_vec(), v.upper_bound())
            }
            BoundedSequenceValue::Uint64BoundedSequence(v) => {
                BoundedSequenceField::Uint64(v.to_vec(), v.upper_bound())
            }
            BoundedSequenceValue::Int64BoundedSequence(v) => {
                BoundedSequenceField::Int64(v.to_vec(), v.upper_bound())
            }
            BoundedSequenceValue::StringBoundedSequence(s) => BoundedSequenceField::String(
                s.iter().map(|s| s.to_string()).collect(),
                s.upper_bound(),
            ),
            BoundedSequenceValue::BoundedStringBoundedSequence(s) => {
                BoundedSequenceField::BoundedString(
                    s.iter().map(|s| s.to_string()).collect(),
                    0, // TODO: s.upper_bound()
                )
            }
            BoundedSequenceValue::WStringBoundedSequence(s) => BoundedSequenceField::WString(
                s.iter().map(|s| s.to_string()).collect(),
                s.upper_bound(),
            ),
            BoundedSequenceValue::BoundedWStringBoundedSequence(s) => {
                BoundedSequenceField::BoundedWString(
                    s.iter().map(|s| s.to_string()).collect(),
                    0, // TODO: s.upper_bound()
                )
            }
            BoundedSequenceValue::MessageBoundedSequence(msgs) => {
                let generic_msgs = msgs.into_iter().map(GenericMessage::from).collect();
                BoundedSequenceField::Message(generic_msgs, 0) // TODO: msgs.upper_bound()
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
    pub fn new(type_name: InterfaceType, fields: IndexMap<String, GenericField>) -> Self {
        Self { fields, type_name }
    }

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

    pub fn get_deep_index(&self, field_index_path: &[usize]) -> Result<AnyTypeRef, String> {
        if field_index_path.is_empty() {
            return Err("Field index path is empty".to_string());
        }
        let index = field_index_path[0];
        let field = self
            .fields
            .get_index(index)
            .map(|(_, field)| field)
            .ok_or_else(|| "Index out of bounds".to_string())?;
        field.get_deep_index(&field_index_path[1..])
    }

    pub fn get_field_type(&self, field_index_path: &[usize]) -> Result<FieldType, String> {
        if field_index_path.is_empty() {
            return Err("Field index path is empty".to_string());
        }
        let index = field_index_path[0];
        let field = self
            .fields
            .get_index(index)
            .map(|(_, field)| field)
            .ok_or_else(|| "Index out of bounds".to_string())?;

        field.get_field_type(&field_index_path[1..])
    }

    pub fn field_count(&self) -> usize {
        self.fields.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rclrs::{dynamic_message::DynamicMessage, SimpleValueMut, ValueMut};

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
