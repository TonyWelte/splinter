use std::f32::consts::E;

use rclrs::{ArrayValue, BoundedSequenceValue, SequenceValue, SimpleValue, Value, ValueMut};

use crate::common::generic_message::{AnyTypeRef, FieldType, SimpleField};

enum GenericValueInternal<'a> {
    Rcl(Value<'a>),
}

enum GenericValueMutInternal<'a> {
    Rcl(ValueMut<'a>),
}

pub struct GenericValue<'a> {
    internal: GenericValueInternal<'a>,
}

pub struct GenericValueMut<'a> {
    internal: GenericValueMutInternal<'a>,
}

impl<'a> From<Value<'a>> for GenericValue<'a> {
    fn from(value: Value<'a>) -> Self {
        // TODO
        GenericValue {
            internal: GenericValueInternal::Rcl(value),
        }
    }
}

fn get_deep_index<'a, 'b>(
    value: &'b Value<'a>,
    field_index_path: &[usize],
) -> Result<AnyTypeRef<'a>, String> {
    match value {
        Value::Simple(SimpleValue::Message(value)) => {
            let next_index = field_index_path
                .first()
                .ok_or("Field index path too short".to_string())?;

            let next_field_name = &value
                .fields
                .get(*next_index)
                .ok_or(format!(
                    "Index {} out of bounds for message with {} fields",
                    next_index,
                    value.fields.len()
                ))?
                .name;

            let next_value = value.get(next_field_name).ok_or(format!(
                "Field '{}' does not exist in message '{}'",
                next_field_name, value.type_name
            ))?;

            get_deep_index(&next_value, &field_index_path[1..])
        }
        Value::Simple(value) => {
            if !field_index_path.is_empty() {
                return Err("Field index path to long".to_string());
            }
            match value {
                SimpleValue::Float(f) => Ok(AnyTypeRef::Float(f)),
                SimpleValue::Double(f) => Ok(AnyTypeRef::Double(f)),
                SimpleValue::LongDouble(f) => Err("LongDouble not supported".to_string()),
                SimpleValue::Char(v) => Ok(AnyTypeRef::Uint8(v)),
                SimpleValue::WChar(v) => Ok(AnyTypeRef::Uint16(v)),
                SimpleValue::Boolean(v) => Ok(AnyTypeRef::Boolean(v)),
                SimpleValue::Octet(v) => Ok(AnyTypeRef::Uint8(v)),
                SimpleValue::Uint8(v) => Ok(AnyTypeRef::Uint8(v)),
                SimpleValue::Int8(v) => Ok(AnyTypeRef::Int8(v)),
                SimpleValue::Uint16(v) => Ok(AnyTypeRef::Uint16(v)),
                SimpleValue::Int16(v) => Ok(AnyTypeRef::Int16(v)),
                SimpleValue::Uint32(v) => Ok(AnyTypeRef::Uint32(v)),
                SimpleValue::Int32(v) => Ok(AnyTypeRef::Int32(v)),
                SimpleValue::Uint64(v) => Ok(AnyTypeRef::Uint64(v)),
                SimpleValue::Int64(v) => Ok(AnyTypeRef::Int64(v)),
                SimpleValue::String(_v) => Err("String not supported".to_string()),
                SimpleValue::BoundedString(_v) => Err("String not supported".to_string()),
                SimpleValue::WString(_v) => Err("String not supported".to_string()),
                SimpleValue::BoundedWString(_v) => Err("String not supported".to_string()),
                SimpleValue::Message(_) => unreachable!(), // Handled above
            }
        }
        Value::Array(ArrayValue::MessageArray(values)) => {
            todo!("handle array of messages")
        }
        Value::Array(value) => {
            if field_index_path.len() != 1 {
                return Err("Field index path to short/long".to_string());
            }
            let index = field_index_path[0];
            let error = format!("Index {} out of bounds for array", index).to_string();
            match value {
                ArrayValue::FloatArray(v) => Ok(AnyTypeRef::Float(v.get(index).ok_or(error)?)),
                ArrayValue::DoubleArray(v) => Ok(AnyTypeRef::Double(v.get(index).ok_or(error)?)),
                ArrayValue::LongDoubleArray(_, _) => Err("LongDouble not supported".to_string()),
                ArrayValue::CharArray(v) => Ok(AnyTypeRef::Uint8(v.get(index).ok_or(error)?)),
                ArrayValue::WCharArray(v) => Ok(AnyTypeRef::Uint16(v.get(index).ok_or(error)?)),
                ArrayValue::BooleanArray(v) => Ok(AnyTypeRef::Boolean(v.get(index).ok_or(error)?)),
                ArrayValue::OctetArray(v) => Ok(AnyTypeRef::Uint8(v.get(index).ok_or(error)?)),
                ArrayValue::Uint8Array(v) => Ok(AnyTypeRef::Uint8(v.get(index).ok_or(error)?)),
                ArrayValue::Int8Array(v) => Ok(AnyTypeRef::Int8(v.get(index).ok_or(error)?)),
                ArrayValue::Uint16Array(v) => Ok(AnyTypeRef::Uint16(v.get(index).ok_or(error)?)),
                ArrayValue::Int16Array(v) => Ok(AnyTypeRef::Int16(v.get(index).ok_or(error)?)),
                ArrayValue::Uint32Array(v) => Ok(AnyTypeRef::Uint32(v.get(index).ok_or(error)?)),
                ArrayValue::Int32Array(v) => Ok(AnyTypeRef::Int32(v.get(index).ok_or(error)?)),
                ArrayValue::Uint64Array(v) => Ok(AnyTypeRef::Uint64(v.get(index).ok_or(error)?)),
                ArrayValue::Int64Array(v) => Ok(AnyTypeRef::Int64(v.get(index).ok_or(error)?)),
                ArrayValue::StringArray(_v) => Err("String not supported".to_string()),
                ArrayValue::BoundedStringArray(_v) => Err("String not supported".to_string()),
                ArrayValue::WStringArray(_v) => Err("String not supported".to_string()),
                ArrayValue::BoundedWStringArray(_v) => Err("String not supported".to_string()),
                ArrayValue::MessageArray(_) => unreachable!(), // Handled above
            }
        }
        Value::Sequence(SequenceValue::MessageSequence(values)) => {
            todo!("handle array of messages")
        }
        Value::Sequence(value) => {
            if field_index_path.len() != 1 {
                return Err("Field index path to short/long".to_string());
            }
            let index = field_index_path[0];
            let error = format!("Index {} out of bounds for array", index).to_string();
            match value {
                SequenceValue::FloatSequence(v) => {
                    Ok(AnyTypeRef::Float(v.get(index).ok_or(error)?))
                }
                SequenceValue::DoubleSequence(v) => {
                    Ok(AnyTypeRef::Double(v.get(index).ok_or(error)?))
                }
                SequenceValue::LongDoubleSequence(_) => Err("LongDouble not supported".to_string()),
                SequenceValue::CharSequence(v) => Ok(AnyTypeRef::Uint8(v.get(index).ok_or(error)?)),
                SequenceValue::WCharSequence(v) => {
                    Ok(AnyTypeRef::Uint16(v.get(index).ok_or(error)?))
                }
                SequenceValue::BooleanSequence(v) => {
                    Ok(AnyTypeRef::Boolean(v.get(index).ok_or(error)?))
                }
                SequenceValue::OctetSequence(v) => {
                    Ok(AnyTypeRef::Uint8(v.get(index).ok_or(error)?))
                }
                SequenceValue::Uint8Sequence(v) => {
                    Ok(AnyTypeRef::Uint8(v.get(index).ok_or(error)?))
                }
                SequenceValue::Int8Sequence(v) => Ok(AnyTypeRef::Int8(v.get(index).ok_or(error)?)),
                SequenceValue::Uint16Sequence(v) => {
                    Ok(AnyTypeRef::Uint16(v.get(index).ok_or(error)?))
                }
                SequenceValue::Int16Sequence(v) => {
                    Ok(AnyTypeRef::Int16(v.get(index).ok_or(error)?))
                }
                SequenceValue::Uint32Sequence(v) => {
                    Ok(AnyTypeRef::Uint32(v.get(index).ok_or(error)?))
                }
                SequenceValue::Int32Sequence(v) => {
                    Ok(AnyTypeRef::Int32(v.get(index).ok_or(error)?))
                }
                SequenceValue::Uint64Sequence(v) => {
                    Ok(AnyTypeRef::Uint64(v.get(index).ok_or(error)?))
                }
                SequenceValue::Int64Sequence(v) => {
                    Ok(AnyTypeRef::Int64(v.get(index).ok_or(error)?))
                }
                SequenceValue::StringSequence(_v) => Err("String not supported".to_string()),
                SequenceValue::BoundedStringSequence(_v) => Err("String not supported".to_string()),
                SequenceValue::WStringSequence(_v) => Err("String not supported".to_string()),
                SequenceValue::BoundedWStringSequence(_v) => {
                    Err("String not supported".to_string())
                }
                SequenceValue::MessageSequence(_) => unreachable!(), // Handled above
            }
        }
        Value::BoundedSequence(BoundedSequenceValue::MessageBoundedSequence(values)) => {
            todo!("handle array of messages")
        }
        Value::BoundedSequence(value) => {
            if field_index_path.len() != 1 {
                return Err("Field index path to short/long".to_string());
            }
            todo!("handle bounded sequences")
            // let index = field_index_path[0];
            // let error = format!("Index {} out of bounds for array", index).to_string();
            // match value {
            //     BoundedSequenceValue::FloatBoundedSequence(v) => {
            //         Ok(AnyTypeRef::Float(v.get(index).ok_or(error)?))
            //     }
            //     BoundedSequenceValue::DoubleBoundedSequence(v) => {
            //         Ok(AnyTypeRef::Double(v.get(index).ok_or(error)?))
            //     }
            //     BoundedSequenceValue::LongDoubleBoundedSequence(_, _) => {
            //         Err("LongDouble not supported".to_string())
            //     }
            //     BoundedSequenceValue::CharBoundedSequence(v) => {
            //         Ok(AnyTypeRef::Uint8(v.get(index).ok_or(error)?))
            //     }
            //     BoundedSequenceValue::WCharBoundedSequence(v) => {
            //         Ok(AnyTypeRef::Uint16(v.get(index).ok_or(error)?))
            //     }
            //     BoundedSequenceValue::BooleanBoundedSequence(v) => {
            //         Ok(AnyTypeRef::Boolean(v.get(index).ok_or(error)?))
            //     }
            //     BoundedSequenceValue::OctetBoundedSequence(v) => {
            //         Ok(AnyTypeRef::Uint8(v.get(index).ok_or(error)?))
            //     }
            //     BoundedSequenceValue::Uint8BoundedSequence(v) => {
            //         Ok(AnyTypeRef::Uint8(v.get(index).ok_or(error)?))
            //     }
            //     BoundedSequenceValue::Int8BoundedSequence(v) => {
            //         Ok(AnyTypeRef::Int8(v.get(index).ok_or(error)?))
            //     }
            //     BoundedSequenceValue::Uint16BoundedSequence(v) => {
            //         Ok(AnyTypeRef::Uint16(v.get(index).ok_or(error)?))
            //     }
            //     BoundedSequenceValue::Int16BoundedSequence(v) => {
            //         Ok(AnyTypeRef::Int16(v.get(index).ok_or(error)?))
            //     }
            //     BoundedSequenceValue::Uint32BoundedSequence(v) => {
            //         Ok(AnyTypeRef::Uint32(v.get(index).ok_or(error)?))
            //     }
            //     BoundedSequenceValue::Int32BoundedSequence(v) => {
            //         Ok(AnyTypeRef::Int32(v.get(index).ok_or(error)?))
            //     }
            //     BoundedSequenceValue::Uint64BoundedSequence(v) => {
            //         Ok(AnyTypeRef::Uint64(v.get(index).ok_or(error)?))
            //     }
            //     BoundedSequenceValue::Int64BoundedSequence(v) => {
            //         Ok(AnyTypeRef::Int64(v.get(index).ok_or(error)?))
            //     }
            //     BoundedSequenceValue::StringBoundedSequence(_v) => {
            //         Err("String not supported".to_string())
            //     }
            //     BoundedSequenceValue::BoundedStringBoundedSequence(_v) => {
            //         Err("String not supported".to_string())
            //     }
            //     BoundedSequenceValue::WStringBoundedSequence(_v) => {
            //         Err("String not supported".to_string())
            //     }
            //     BoundedSequenceValue::BoundedWStringBoundedSequence(_v) => {
            //         Err("String not supported".to_string())
            //     }
            //     BoundedSequenceValue::MessageBoundedSequence(_) => unreachable!(), // Handled above
            // }
        }
        _ => todo!("handle complex types"),
    }
}

impl<'a> GenericValue<'a> {
    pub fn get_deep_index(&self, field_index_path: &[usize]) -> Result<AnyTypeRef<'a>, String> {
        match &self.internal {
            GenericValueInternal::Rcl(value) => get_deep_index(value, field_index_path),
        }
    }

    pub fn get_field_type(&self, field_index_path: &[usize]) -> Result<FieldType, String> {
        if field_index_path.is_empty() {
            return match &self.internal {
                GenericValueInternal::Rcl(value) => match value {
                    Value::Simple(SimpleValue::Message(_)) => Ok(FieldType::Message),
                    Value::Simple(SimpleValue::Float(_)) => Ok(FieldType::Float),
                    Value::Simple(SimpleValue::Double(_)) => Ok(FieldType::Double),
                    Value::Simple(SimpleValue::Boolean(_)) => Ok(FieldType::Boolean),
                    Value::Simple(SimpleValue::Uint8(_)) => Ok(FieldType::Uint8),
                    Value::Simple(SimpleValue::Int8(_)) => Ok(FieldType::Int8),
                    Value::Simple(SimpleValue::Uint16(_)) => Ok(FieldType::Uint16),
                    Value::Simple(SimpleValue::Int16(_)) => Ok(FieldType::Int16),
                    Value::Simple(SimpleValue::Uint32(_)) => Ok(FieldType::Uint32),
                    Value::Simple(SimpleValue::Int32(_)) => Ok(FieldType::Int32),
                    Value::Simple(SimpleValue::Uint64(_)) => Ok(FieldType::Uint64),
                    Value::Simple(SimpleValue::Int64(_)) => Ok(FieldType::Int64),
                    Value::Simple(_) => Err("Unsupported simple type".to_string()),
                    Value::Array(_) => Ok(FieldType::Array),
                    Value::Sequence(_) => Ok(FieldType::Sequence),
                    Value::BoundedSequence(_) => Ok(FieldType::BoundedSequence),
                },
            };
        }

        let next_index = field_index_path.first().unwrap();

        match &self.internal {
            GenericValueInternal::Rcl(value) => match value {
                Value::Simple(SimpleValue::Message(msg)) => {
                    let field_info = msg
                        .fields
                        .get(*next_index)
                        .ok_or("Index out of bounds".to_string())?;
                    let field = msg.get(&field_info.name).ok_or(format!(
                        "Field '{}' does not exist in message '{}'",
                        field_info.name, msg.type_name
                    ))?;
                    let generic_value = GenericValue::from(field);
                    generic_value.get_field_type(&field_index_path[1..])
                }
                Value::Simple(_) => Err("Cannot get field type of non-message value".to_string()),
                Value::Array(ArrayValue::MessageArray(msgs)) => {
                    if msgs.is_empty() {
                        return Err("Cannot get field type of empty array".to_string());
                    }
                    if msgs.len() <= *next_index {
                        return Err("Index out of bounds".to_string());
                    }
                    let field = &msgs[*next_index];
                    let generic_value =
                        GenericValue::from(Value::Simple(SimpleValue::Message(field.clone())));
                    generic_value.get_field_type(&field_index_path[1..])
                }
                _ => Err("Cannot get field type of non-message value".to_string()),
            },
        }
    }

    pub fn get_field_name(&self, field_index_path: &[usize]) -> Result<String, String> {
        todo!("implement get_field_name")
    }
}
