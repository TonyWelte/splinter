use std::usize;

use rclrs::{
    ArrayValue, BoundedSequenceValue, DynamicMessage, DynamicMessageView, MessageTypeName,
    SequenceValue, SimpleValue, Value,
};

use crate::common::generic_message::{
    ArrayField, BoundedSequenceField, GenericField, GenericMessage, SequenceField, SimpleField,
};

struct GenericMessageSelection {
    message: GenericMessage,
    current_field_path: Vec<usize>,
}

impl GenericMessageSelection {
    pub fn new(message: GenericMessage) -> Self {
        Self {
            message: message,
            current_field_path: vec![],
        }
    }

    pub fn get_selected_field_path(&self) -> Vec<usize> {
        self.current_field_path.clone()
    }

    pub fn next(&mut self) {
        self.current_field_path =
            next_field(&self.message, &self.current_field_path).unwrap_or_default();
    }

    pub fn previous(&mut self) {
        // self.current_field_path =
        //     prev_field(&self.message, &self.current_field_path).unwrap_or_default();
    }
}

fn skip_length(current_field_path: &[usize]) -> usize {
    match current_field_path.len() {
        0 => 0,
        1 => current_field_path[0] + 1,
        _ => current_field_path[0],
    }
}

fn skip_length_backward(field_count: usize, current_field_path: &[usize]) -> usize {
    match current_field_path.len() {
        0 => 0,
        1 => field_count - current_field_path[0],
        _ => field_count - current_field_path[0] - 1,
    }
}

// macro to handle array next field selection
macro_rules! next_messages {
    ($array:expr, $current_field_path:expr) => {{
        let mut result = None;
        let skip_fields = skip_length($current_field_path);
        for (index, item) in $array.iter().enumerate().skip(skip_fields) {
            if let Some(inner_path) = next_field(&item, &$current_field_path[1..]) {
                let mut path = vec![index];
                path.extend(inner_path);
                result = Some(path);
                break;
            }
        }
        result
    }};
}

macro_rules! prev_messages {
    ($array:expr, $current_field_path:expr) => {{
        let mut result = None;
        let skip_fields = skip_length_backward($array.len(), $current_field_path);
        for (index, item) in $array.iter().enumerate().rev().skip(skip_fields) {
            if let Some(inner_path) = prev_field(&item, &$current_field_path[1..]) {
                let mut path = vec![index];
                path.extend(inner_path);
                result = Some(path);
                break;
            }
        }
        result
    }};
}

macro_rules! next_values {
    ($index:expr, $array:expr, $current_field_path:expr) => {{
        let skip_fields = skip_length($current_field_path);
        for i in (0..$array.len()).skip(skip_fields) {
            assert!(
                $current_field_path.len() < 2,
                "Nested list/array selection not supported"
            );
            return Some(vec![$index, i]);
        }
    }};
}

macro_rules! prev_values {
    ($index:expr, $array:expr, $current_field_path:expr) => {{
        let skip_fields = skip_length_backward($array.len(), $current_field_path);
        for i in (0..$array.len()).rev().skip(skip_fields) {
            assert!(
                $current_field_path.len() < 2,
                "Nested list/array selection not supported"
            );
            return Some(vec![$index, i]);
        }
    }};
}

pub fn next_field(msg: &GenericMessage, current_field_path: &[usize]) -> Option<Vec<usize>> {
    let skip_fields = skip_length(current_field_path);
    let mut next_current_field_path =
        current_field_path[1.min(current_field_path.len())..].to_vec();
    for (index, (name, value)) in msg.iter().enumerate().skip(skip_fields) {
        match value {
            GenericField::Simple(simple_value) => {
                if let SimpleField::Message(inner_msg) = simple_value {
                    if let Some(inner_path) = next_field(&inner_msg, &next_current_field_path) {
                        let mut path = vec![index];
                        path.extend(inner_path);
                        return Some(path);
                    }
                } else {
                    return Some(vec![index]);
                }
            }
            GenericField::Array(array_value) => {
                match array_value {
                    ArrayField::Message(inner_msgs) => {
                        let innder_skip_fields = skip_length(&next_current_field_path);
                        next_current_field_path = next_current_field_path
                            [1.min(next_current_field_path.len())..]
                            .to_vec();
                        if let Some(inner_path) =
                            next_messages!(inner_msgs, &next_current_field_path)
                        {
                            let mut path = vec![index];
                            path.extend(inner_path);
                            return Some(path);
                        }
                    }
                    ArrayField::Float(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    ArrayField::Double(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    ArrayField::LongDouble(_) => {
                        todo!("LongDoubleArray is not supported yet")
                    }
                    ArrayField::Int8(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    ArrayField::Uint8(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    ArrayField::Int16(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    ArrayField::Uint16(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    ArrayField::Int32(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    ArrayField::Uint32(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    ArrayField::Int64(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    ArrayField::Uint64(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    ArrayField::Char(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    ArrayField::WChar(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    ArrayField::Octet(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    ArrayField::Boolean(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    ArrayField::String(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    ArrayField::WString(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    ArrayField::BoundedString(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    ArrayField::BoundedWString(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                }
                next_current_field_path.clear();
            }
            GenericField::Sequence(sequence_value) => {
                match sequence_value {
                    SequenceField::Message(inner_msgs) => {
                        if let Some(inner_path) =
                            next_messages!(inner_msgs, &next_current_field_path)
                        {
                            let mut path = vec![index];
                            path.extend(inner_path);
                            return Some(path);
                        }
                    }
                    SequenceField::Float(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    SequenceField::Double(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    SequenceField::LongDouble(_) => {
                        todo!("LongDoubleSequence is not supported yet")
                    }
                    SequenceField::Int8(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    SequenceField::Uint8(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    SequenceField::Int16(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    SequenceField::Uint16(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    SequenceField::Int32(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    SequenceField::Uint32(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    SequenceField::Int64(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    SequenceField::Uint64(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    SequenceField::Octet(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    SequenceField::Char(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    SequenceField::WChar(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    SequenceField::Boolean(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    SequenceField::String(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    SequenceField::WString(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    SequenceField::BoundedString(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    SequenceField::BoundedWString(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                }
                next_current_field_path.clear();
            }
            GenericField::BoundedSequence(bounded_sequence_value) => {
                match bounded_sequence_value {
                    BoundedSequenceField::Message(inner_msgs) => {
                        if let Some(inner_path) =
                            next_messages!(inner_msgs, &next_current_field_path)
                        {
                            let mut path = vec![index];
                            path.extend(inner_path);
                            return Some(path);
                        }
                    }
                    BoundedSequenceField::Float(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    BoundedSequenceField::Double(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    BoundedSequenceField::LongDouble(_) => {
                        todo!("LongDoubleSequence is not supported yet")
                    }
                    BoundedSequenceField::Int8(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    BoundedSequenceField::Uint8(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    BoundedSequenceField::Int16(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    BoundedSequenceField::Uint16(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    BoundedSequenceField::Int32(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    BoundedSequenceField::Uint32(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    BoundedSequenceField::Int64(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    BoundedSequenceField::Uint64(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    BoundedSequenceField::Octet(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    BoundedSequenceField::Char(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    BoundedSequenceField::WChar(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    BoundedSequenceField::Boolean(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    BoundedSequenceField::String(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    BoundedSequenceField::WString(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    BoundedSequenceField::BoundedString(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                    BoundedSequenceField::BoundedWString(inner_values) => {
                        next_values!(index, inner_values, &next_current_field_path)
                    }
                }
                next_current_field_path.clear();
            }
        }
        next_current_field_path.clear();
    }
    None
}

// pub fn prev_field(msg: &GenericMessage, current_field_path: &[usize]) -> Option<Vec<usize>> {
//     let skip_fields = skip_length_backward(msg.field_count(), current_field_path);
//     let mut prev_current_field_path =
//         current_field_path[1.min(current_field_path.len())..].to_vec();
//     for (index, (name, value)) in msg.iter().enumerate().rev().skip(skip_fields) {
//         match value {
//             Value::Simple(simple_value) => {
//                 if let SimpleValue::Message(inner_msg) = simple_value {
//                     if let Some(inner_path) = prev_field(&inner_msg, &prev_current_field_path) {
//                         let mut path = vec![index];
//                         path.extend(inner_path);
//                         return Some(path);
//                     }
//                 } else {
//                     return Some(vec![index]);
//                 }
//             }
//             Value::Array(array_value) => {
//                 match array_value {
//                     ArrayValue::MessageArray(inner_msgs) => {
//                         if let Some(inner_path) =
//                             prev_messages!(inner_msgs, &prev_current_field_path)
//                         {
//                             let mut path = vec![index];
//                             path.extend(inner_path);
//                             return Some(path);
//                         }
//                     }
//                     ArrayValue::FloatArray(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     ArrayValue::DoubleArray(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     ArrayValue::LongDoubleArray(_, _) => {
//                         todo!("LongDoubleArray is not supported yet")
//                     }
//                     ArrayValue::Int8Array(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     ArrayValue::Uint8Array(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     ArrayValue::Int16Array(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     ArrayValue::Uint16Array(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     ArrayValue::Int32Array(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     ArrayValue::Uint32Array(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     ArrayValue::Int64Array(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     ArrayValue::Uint64Array(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     ArrayValue::CharArray(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     ArrayValue::WCharArray(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     ArrayValue::OctetArray(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     ArrayValue::BooleanArray(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     ArrayValue::StringArray(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     ArrayValue::WStringArray(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     ArrayValue::BoundedStringArray(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     ArrayValue::BoundedWStringArray(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                 }
//                 prev_current_field_path.clear();
//             }
//             Value::Sequence(sequence_value) => {
//                 match sequence_value {
//                     SequenceValue::MessageSequence(inner_msgs) => {
//                         if let Some(inner_path) =
//                             prev_messages!(inner_msgs, &prev_current_field_path)
//                         {
//                             let mut path = vec![index];
//                             path.extend(inner_path);
//                             return Some(path);
//                         }
//                     }
//                     SequenceValue::FloatSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     SequenceValue::DoubleSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     SequenceValue::LongDoubleSequence(_) => {
//                         todo!("LongDoubleSequence is not supported yet")
//                     }
//                     SequenceValue::Int8Sequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     SequenceValue::Uint8Sequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     SequenceValue::Int16Sequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     SequenceValue::Uint16Sequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     SequenceValue::Int32Sequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     SequenceValue::Uint32Sequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     SequenceValue::Int64Sequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     SequenceValue::Uint64Sequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     SequenceValue::OctetSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     SequenceValue::CharSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     SequenceValue::WCharSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     SequenceValue::BooleanSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     SequenceValue::StringSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     SequenceValue::WStringSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     SequenceValue::BoundedStringSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     SequenceValue::BoundedWStringSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                 }
//                 prev_current_field_path.clear();
//             }
//             Value::BoundedSequence(bounded_sequence_value) => {
//                 match bounded_sequence_value {
//                     BoundedSequenceValue::MessageBoundedSequence(inner_msgs) => {
//                         if let Some(inner_path) =
//                             prev_messages!(inner_msgs, &prev_current_field_path)
//                         {
//                             let mut path = vec![index];
//                             path.extend(inner_path);
//                             return Some(path);
//                         }
//                     }
//                     BoundedSequenceValue::FloatBoundedSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     BoundedSequenceValue::DoubleBoundedSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     BoundedSequenceValue::LongDoubleBoundedSequence(_, _) => {
//                         todo!("LongDoubleSequence is not supported yet")
//                     }
//                     BoundedSequenceValue::Int8BoundedSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     BoundedSequenceValue::Uint8BoundedSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     BoundedSequenceValue::Int16BoundedSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     BoundedSequenceValue::Uint16BoundedSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     BoundedSequenceValue::Int32BoundedSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     BoundedSequenceValue::Uint32BoundedSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     BoundedSequenceValue::Int64BoundedSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     BoundedSequenceValue::Uint64BoundedSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     BoundedSequenceValue::OctetBoundedSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     BoundedSequenceValue::CharBoundedSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     BoundedSequenceValue::WCharBoundedSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     BoundedSequenceValue::BooleanBoundedSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     BoundedSequenceValue::StringBoundedSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     BoundedSequenceValue::WStringBoundedSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     BoundedSequenceValue::BoundedStringBoundedSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                     BoundedSequenceValue::BoundedWStringBoundedSequence(inner_values) => {
//                         prev_values!(index, inner_values, &prev_current_field_path)
//                     }
//                 }
//                 prev_current_field_path.clear();
//             }
//         }
//         prev_current_field_path.clear();
//     }
//     None
// }

mod test {
    use crate::common::generic_message;

    use super::*;

    #[test]
    fn test_next_field_basic_types() {
        let message_type = MessageTypeName {
            package_name: "test_msgs".to_owned(),
            type_name: "BasicTypes".to_owned(),
        };
        let mut msg = DynamicMessage::new(message_type).unwrap();
        let generic_message = GenericMessage::from(msg.view());
        let mut msg_selection = GenericMessageSelection::new(generic_message);

        assert_eq!(msg_selection.get_selected_field_path(), vec![]);
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![0]);
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![1]);
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![2]);
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![3]);
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![4]);
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![5]);
    }

    #[test]
    fn test_next_field_odometry() {
        let message_type = MessageTypeName {
            package_name: "nav_msgs".to_owned(),
            type_name: "Odometry".to_owned(),
        };
        let mut msg = DynamicMessage::new(message_type).unwrap();
        let generic_message = GenericMessage::from(msg.view());
        let mut msg_selection = GenericMessageSelection::new(generic_message);

        assert_eq!(msg_selection.get_selected_field_path(), vec![]);
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![0, 0, 0]); // header.stamp.sec
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![0, 0, 1]); // header.stamp.nanosec
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![0, 1]); // header.frame_id
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![1]); // child_frame_id
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![2, 0, 0, 0]); // pose.pose.position.x
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![2, 0, 0, 1]); // pose.pose.position.y
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![2, 0, 0, 2]); // pose.pose.position.z
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![2, 0, 1, 0]); // pose.pose.orientation.x
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![2, 0, 1, 1]); // pose.pose.orientation.y
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![2, 0, 1, 2]); // pose.pose.orientation.z
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![2, 0, 1, 3]); // pose.pose.orientation.w
        for i in 0..36 {
            msg_selection.next();
            assert_eq!(msg_selection.get_selected_field_path(), vec![2, 1, i]); // pose.covariance[i]
        }
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![3, 0, 0, 0]); // twist.twist.linear.x
    }

    #[test]
    fn test_previous_field_basic_types() {
        let message_type = MessageTypeName {
            package_name: "test_msgs".to_owned(),
            type_name: "BasicTypes".to_owned(),
        };
        let mut msg = DynamicMessage::new(message_type).unwrap();
        let generic_message = GenericMessage::from(msg.view());
        let mut msg_selection = GenericMessageSelection::new(generic_message);

        assert_eq!(msg_selection.get_selected_field_path(), vec![]);
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![0]);
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![1]);
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![2]);
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![3]);
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![4]);
        msg_selection.next();
        assert_eq!(msg_selection.get_selected_field_path(), vec![5]);

        // Test going back
        msg_selection.previous();
        assert_eq!(msg_selection.get_selected_field_path(), vec![4]);
        msg_selection.previous();
        assert_eq!(msg_selection.get_selected_field_path(), vec![3]);
        msg_selection.previous();
        assert_eq!(msg_selection.get_selected_field_path(), vec![2]);
        msg_selection.previous();
        assert_eq!(msg_selection.get_selected_field_path(), vec![1]);
        msg_selection.previous();
        assert_eq!(msg_selection.get_selected_field_path(), vec![0]);
    }

    #[test]
    fn test_previous_field_odometry() {
        let message_type = MessageTypeName {
            package_name: "nav_msgs".to_owned(),
            type_name: "Odometry".to_owned(),
        };
        let msg = DynamicMessage::new(message_type).unwrap();
        let generic_message = GenericMessage::from(msg.view());

        let mut msg_selection = GenericMessageSelection::new(generic_message);
        assert_eq!(msg_selection.get_selected_field_path(), vec![]);

        for i in (0..36).rev() {
            msg_selection.previous();
            assert_eq!(msg_selection.get_selected_field_path(), vec![3, 1, i]); // twist.covariance[i]
        }
        msg_selection.previous();
        assert_eq!(msg_selection.get_selected_field_path(), vec![3, 0, 1, 2]); // twist.twist.angular.z
        msg_selection.previous();
        assert_eq!(msg_selection.get_selected_field_path(), vec![3, 0, 1, 1]); // twist.twist.angular.y
        msg_selection.previous();
        assert_eq!(msg_selection.get_selected_field_path(), vec![3, 0, 1, 0]); // twist.twist.angular.x
        msg_selection.previous();
        assert_eq!(msg_selection.get_selected_field_path(), vec![3, 0, 0, 2]); // twist.twist.linear.z
        msg_selection.previous();
        assert_eq!(msg_selection.get_selected_field_path(), vec![3, 0, 0, 1]); // twist.twist.linear.y
        msg_selection.previous();
        assert_eq!(msg_selection.get_selected_field_path(), vec![3, 0, 0, 0]); // twist.twist.linear.x
        msg_selection.previous();
        for i in (0..36).rev() {
            assert_eq!(msg_selection.get_selected_field_path(), vec![2, 1, i]); // pose.covariance[i]
            msg_selection.previous();
        }
        assert_eq!(msg_selection.get_selected_field_path(), vec![2, 0, 1, 3]); // pose.pose.orientation.w
        msg_selection.previous();
        assert_eq!(msg_selection.get_selected_field_path(), vec![2, 0, 1, 2]); // pose.pose.orientation.z
        msg_selection.previous();
        assert_eq!(msg_selection.get_selected_field_path(), vec![2, 0, 1, 1]); // pose.pose.orientation.y
        msg_selection.previous();
        assert_eq!(msg_selection.get_selected_field_path(), vec![2, 0, 1, 0]); // pose.pose.orientation.x
        msg_selection.previous();
        assert_eq!(msg_selection.get_selected_field_path(), vec![2, 0, 0, 2]); // pose.pose.position.z
        msg_selection.previous();
        assert_eq!(msg_selection.get_selected_field_path(), vec![2, 0, 0, 1]); // pose.pose.position.y
        msg_selection.previous();
        assert_eq!(msg_selection.get_selected_field_path(), vec![2, 0, 0, 0]); // pose.pose.position.x
        msg_selection.previous();
        assert_eq!(msg_selection.get_selected_field_path(), vec![1]); // child_frame_id
        msg_selection.previous();
        assert_eq!(msg_selection.get_selected_field_path(), vec![0, 1]); // header.frame_id
        msg_selection.previous();
        assert_eq!(msg_selection.get_selected_field_path(), vec![0, 0, 1]); // header.stamp.nanosec
        msg_selection.previous();
        assert_eq!(msg_selection.get_selected_field_path(), vec![0, 0, 0]); // header.stamp.sec
    }

    // #[test]
    // fn test_next_field_sequence() {
    //     let mut msg_selection = DynamicMessageSelection::new(msg);
    //     assert_eq!(msg_selection.get_selected_field_path(), vec![]);

    //     msg_selection.next();
    //     assert_eq!(msg_selection.get_selected_field_path(), vec![0, 0]); //
    // }
}
