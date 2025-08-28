use std::usize;

use crate::common::generic_message::{
    ArrayField, BoundedSequenceField, GenericField, GenericMessage, Length, SequenceField,
    SimpleField,
};

pub struct GenericMessageSelector<'a> {
    message: &'a GenericMessage,
}

fn has_field(message: &GenericMessage, field_path: &[usize]) -> bool {
    if field_path.len() == 0 {
        return true;
    }

    if field_path.len() == 1 {
        return field_path[0] < message.len();
    }

    let field_index = field_path[0];
    if field_index >= message.len() {
        return false;
    }

    let field = message.get_index(field_index).unwrap();
    match field {
        GenericField::Simple(simple_value) => {
            if let SimpleField::Message(inner_msg) = simple_value {
                return has_field(&inner_msg, &field_path[1..]);
            } else {
                return false;
            }
        }
        GenericField::Array(array_value) => match array_value {
            ArrayField::Message(inner_msgs) => {
                return has_field(&inner_msgs[1], &field_path[2..]);
            }
            _ => {
                return field_path.len() == 2 && field_path[1] < array_value.len();
            }
        },
        GenericField::Sequence(sequence_value) => match sequence_value {
            SequenceField::Message(inner_msgs) => {
                return has_field(&inner_msgs[1], &field_path[2..]);
            }
            _ => {
                return field_path.len() == 2 && field_path[1] < sequence_value.len();
            }
        },
        GenericField::BoundedSequence(bounded_sequence_value) => match bounded_sequence_value {
            BoundedSequenceField::Message(inner_msgs) => {
                return has_field(&inner_msgs[1], &field_path[2..]);
            }
            _ => {
                return field_path.len() == 2 && field_path[1] < bounded_sequence_value.len();
            }
        },
    }
}

fn get_last_index_path(message: &GenericMessage, field_path: &[usize]) -> Option<Vec<usize>> {
    if field_path.len() == 0 {
        return get_last_index_path(&message, &[message.len() - 1]);
    }

    let field_index = field_path[0];
    if field_index >= message.len() {
        return None;
    }

    let field = message.get_index(field_index).unwrap();
    match field {
        GenericField::Simple(simple_value) => {
            if let SimpleField::Message(inner_msg) = simple_value {
                if let Some(mut inner_path) = get_last_index_path(&inner_msg, &field_path[1..]) {
                    let mut path = vec![field_index];
                    path.append(&mut inner_path);
                    return Some(path);
                } else {
                    return None;
                }
            } else {
                if field_path.len() == 1 {
                    return Some(vec![field_index]);
                } else {
                    return None;
                }
            }
        }
        GenericField::Array(array_value) => match array_value {
            ArrayField::Message(inner_msgs) => {
                if field_path.len() < 2 {
                    return None;
                }
                let inner_index = field_path[1];
                if inner_index >= inner_msgs.len() {
                    return None;
                }
                if let Some(mut inner_path) =
                    get_last_index_path(&inner_msgs[inner_index], &field_path[2..])
                {
                    let mut path = vec![field_index, inner_index];
                    path.append(&mut inner_path);
                    return Some(path);
                } else {
                    return None;
                }
            }
            _ => {
                if field_path.len() == 2 && field_path[1] < array_value.len() {
                    return Some(vec![field_index, field_path[1]]);
                } else {
                    return Some(vec![field_index, array_value.len() - 1]);
                }
            }
        },
        GenericField::Sequence(sequence_value) => match sequence_value {
            SequenceField::Message(inner_msgs) => {
                if field_path.len() < 2 {
                    return None;
                }
                let inner_index = field_path[1];
                if inner_index >= inner_msgs.len() {
                    return None;
                }
                if let Some(mut inner_path) =
                    get_last_index_path(&inner_msgs[inner_index], &field_path[2..])
                {
                    let mut path = vec![field_index, inner_index];
                    path.append(&mut inner_path);
                    return Some(path);
                } else {
                    return None;
                }
            }
            _ => {
                if field_path.len() == 2 && field_path[1] < sequence_value.len() {
                    return Some(vec![field_index, field_path[1]]);
                } else {
                    return Some(vec![field_index, sequence_value.len() - 1]);
                }
            }
        },
        GenericField::BoundedSequence(bounded_sequence_value) => match bounded_sequence_value {
            BoundedSequenceField::Message(inner_msgs) => {
                if field_path.len() < 2 {
                    return None;
                }
                let inner_index = field_path[1];
                if inner_index >= inner_msgs.len() {
                    return None;
                }
                if let Some(mut inner_path) =
                    get_last_index_path(&inner_msgs[inner_index], &field_path[2..])
                {
                    let mut path = vec![field_index, inner_index];
                    path.append(&mut inner_path);
                    return Some(path);
                } else {
                    return None;
                }
            }
            _ => {
                if field_path.len() == 2 && field_path[1] < bounded_sequence_value.len() {
                    return Some(vec![field_index, field_path[1]]);
                } else {
                    return Some(vec![field_index, bounded_sequence_value.len() - 1]);
                }
            }
        },
    }
}

impl<'a> GenericMessageSelector<'a> {
    pub fn new(message: &'a GenericMessage) -> Self {
        Self { message }
    }

    pub fn down(&self, current_field_path: &[usize]) -> Vec<usize> {
        if self.message.len() == 0 {
            return vec![];
        }

        if current_field_path.len() == 0 {
            return vec![0];
        }

        let mut result = current_field_path.to_vec();

        // Try to go right first
        result.push(0);
        if has_field(&self.message, &result) {
            return result;
        }
        result.pop();

        // Try to go down
        result.last_mut().map(|last| *last += 1);
        while !result.is_empty() && !has_field(&self.message, &result) {
            result.pop();
            if let Some(last) = result.last_mut() {
                *last += 1;
            }
        }

        result
    }

    pub fn up(&self, current_field_path: &[usize]) -> Vec<usize> {
        if self.message.len() == 0 {
            return vec![];
        }

        if current_field_path.len() == 0 {
            return vec![self.message.len() - 1];
        }

        let mut result = current_field_path.to_vec();
        if result.last().unwrap() > &0 {
            result.last_mut().map(|last| *last -= 1);
            if let Some(inner_path) = get_last_index_path(&self.message, &result) {
                return inner_path;
            } else {
                return result;
            }
        }

        // If can't go up, go left
        result.pop();
        return result;
    }

    pub fn left(&self, current_field_path: &[usize]) -> Vec<usize> {
        if self.message.len() == 0 {
            return vec![];
        }

        if current_field_path.len() == 0 {
            return vec![self.message.len() - 1];
        }

        let mut result = current_field_path.to_vec();
        if result.len() == 1 && result[0] > 0 {
            result[0] -= 1;
            return result;
        }

        result.pop();
        return result;
    }

    pub fn right(&self, current_field_path: &[usize]) -> Vec<usize> {
        if self.message.len() == 0 {
            return vec![];
        }

        if current_field_path.len() == 0 {
            return vec![0];
        }

        if current_field_path.len() == 1 && current_field_path[0] + 1 > self.message.len() - 1 {
            return vec![];
        }

        let mut result = current_field_path.to_vec();
        result.last_mut().map(|last| *last += 1);
        if has_field(&self.message, &result) {
            return result;
        } else {
            return current_field_path.to_vec();
        }
    }

    pub fn last_field_path(&self) -> Vec<usize> {
        if self.message.len() == 0 {
            return vec![];
        }

        get_last_index_path(&self.message, &[]).unwrap_or_else(|| vec![])
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

pub fn prev_field(msg: &GenericMessage, current_field_path: &[usize]) -> Option<Vec<usize>> {
    let skip_fields = skip_length_backward(msg.field_count(), current_field_path);
    let mut prev_current_field_path =
        current_field_path[1.min(current_field_path.len())..].to_vec();
    for index in (0..msg.len()).rev().skip(skip_fields) {
        let value = msg.get_index(index).unwrap();
        match value {
            GenericField::Simple(simple_value) => {
                if let SimpleField::Message(inner_msg) = simple_value {
                    if let Some(inner_path) = prev_field(&inner_msg, &prev_current_field_path) {
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
                        if let Some(inner_path) =
                            prev_messages!(inner_msgs, &prev_current_field_path)
                        {
                            let mut path = vec![index];
                            path.extend(inner_path);
                            return Some(path);
                        }
                    }
                    ArrayField::Float(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    ArrayField::Double(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    ArrayField::LongDouble(_) => {
                        todo!("LongDoubleArray is not supported yet")
                    }
                    ArrayField::Int8(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    ArrayField::Uint8(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    ArrayField::Int16(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    ArrayField::Uint16(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    ArrayField::Int32(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    ArrayField::Uint32(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    ArrayField::Int64(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    ArrayField::Uint64(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    ArrayField::Char(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    ArrayField::WChar(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    ArrayField::Octet(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    ArrayField::Boolean(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    ArrayField::String(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    ArrayField::WString(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    ArrayField::BoundedString(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    ArrayField::BoundedWString(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                }
                prev_current_field_path.clear();
            }
            GenericField::Sequence(sequence_value) => {
                match sequence_value {
                    SequenceField::Message(inner_msgs) => {
                        if let Some(inner_path) =
                            prev_messages!(inner_msgs, &prev_current_field_path)
                        {
                            let mut path = vec![index];
                            path.extend(inner_path);
                            return Some(path);
                        }
                    }
                    SequenceField::Float(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    SequenceField::Double(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    SequenceField::LongDouble(_) => {
                        todo!("LongDoubleSequence is not supported yet")
                    }
                    SequenceField::Int8(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    SequenceField::Uint8(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    SequenceField::Int16(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    SequenceField::Uint16(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    SequenceField::Int32(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    SequenceField::Uint32(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    SequenceField::Int64(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    SequenceField::Uint64(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    SequenceField::Octet(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    SequenceField::Char(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    SequenceField::WChar(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    SequenceField::Boolean(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    SequenceField::String(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    SequenceField::WString(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    SequenceField::BoundedString(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    SequenceField::BoundedWString(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                }
                prev_current_field_path.clear();
            }
            GenericField::BoundedSequence(bounded_sequence_value) => {
                match bounded_sequence_value {
                    BoundedSequenceField::Message(inner_msgs) => {
                        if let Some(inner_path) =
                            prev_messages!(inner_msgs, &prev_current_field_path)
                        {
                            let mut path = vec![index];
                            path.extend(inner_path);
                            return Some(path);
                        }
                    }
                    BoundedSequenceField::Float(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    BoundedSequenceField::Double(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    BoundedSequenceField::LongDouble(_) => {
                        todo!("LongDoubleSequence is not supported yet")
                    }
                    BoundedSequenceField::Int8(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    BoundedSequenceField::Uint8(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    BoundedSequenceField::Int16(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    BoundedSequenceField::Uint16(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    BoundedSequenceField::Int32(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    BoundedSequenceField::Uint32(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    BoundedSequenceField::Int64(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    BoundedSequenceField::Uint64(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    BoundedSequenceField::Octet(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    BoundedSequenceField::Char(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    BoundedSequenceField::WChar(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    BoundedSequenceField::Boolean(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    BoundedSequenceField::String(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    BoundedSequenceField::WString(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    BoundedSequenceField::BoundedString(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                    BoundedSequenceField::BoundedWString(inner_values) => {
                        prev_values!(index, inner_values, &prev_current_field_path)
                    }
                }
                prev_current_field_path.clear();
            }
        }
        prev_current_field_path.clear();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use rclrs::DynamicMessage;
    use rclrs::MessageTypeName;

    #[test]
    fn test_has_field() {
        let message_type = MessageTypeName {
            package_name: "nav_msgs".to_owned(),
            type_name: "Odometry".to_owned(),
        };
        let msg = DynamicMessage::new(message_type).unwrap();
        let generic_message = GenericMessage::from(msg.view());

        assert!(has_field(&generic_message, &vec![])); // Root always exists
        assert!(has_field(&generic_message, &vec![0])); // header
        assert!(has_field(&generic_message, &vec![2, 0, 1, 0])); // pose.pose.orientation.x
        assert!(has_field(&generic_message, &vec![2, 1, 35])); // pose.covariance.35
        assert!(!has_field(&generic_message, &vec![10])); // Out of bounds
        assert!(!has_field(&generic_message, &vec![2, 5])); // Out of bounds
        assert!(!has_field(&generic_message, &vec![2, 0, 5])); // Out of bounds
        assert!(!has_field(&generic_message, &vec![2, 0, 1, 5])); // Out of bounds
    }

    #[test]
    fn test_next_field_basic_types() {
        let message_type = MessageTypeName {
            package_name: "test_msgs".to_owned(),
            type_name: "BasicTypes".to_owned(),
        };
        let msg = DynamicMessage::new(message_type).unwrap();
        let generic_message = GenericMessage::from(msg.view());
        let msg_selection = GenericMessageSelector::new(&generic_message);

        let mut selection = vec![];
        selection = msg_selection.down(&selection);
        assert_eq!(selection, vec![0]);
        selection = msg_selection.down(&selection);
        assert_eq!(selection, vec![1]);
        selection = msg_selection.down(&selection);
        assert_eq!(selection, vec![2]);
        selection = msg_selection.down(&selection);
        assert_eq!(selection, vec![3]);
        selection = msg_selection.down(&selection);
        assert_eq!(selection, vec![4]);
        selection = msg_selection.down(&selection);
        assert_eq!(selection, vec![5]);
    }

    #[test]
    fn test_next_field_odometry() {
        let message_type = MessageTypeName {
            package_name: "nav_msgs".to_owned(),
            type_name: "Odometry".to_owned(),
        };
        let msg = DynamicMessage::new(message_type).unwrap();
        let generic_message = GenericMessage::from(msg.view());
        let msg_selection = GenericMessageSelector::new(&generic_message);

        let mut selection = vec![];
        selection = msg_selection.down(&selection); // header
        assert_eq!(selection, vec![0]);
        selection = msg_selection.down(&selection); // child_frame_id
        assert_eq!(selection, vec![1]);
        selection = msg_selection.down(&selection); // pose
        assert_eq!(selection, vec![2]);
        selection = msg_selection.right(&selection); // pose.pose
        assert_eq!(selection, vec![2, 0]);
        selection = msg_selection.right(&selection); // pose.pose.position
        assert_eq!(selection, vec![2, 0, 0]);
        selection = msg_selection.down(&selection); // pose.pose.orientation
        assert_eq!(selection, vec![2, 0, 1]);
        selection = msg_selection.right(&selection); // pose.pose.orientation.x
        assert_eq!(selection, vec![2, 0, 1, 0]);
        selection = msg_selection.right(&selection); // No deeper field, so no change
        assert_eq!(selection, vec![2, 0, 1, 0]);

        selection = vec![2, 0]; // pose.pose
        selection = msg_selection.down(&selection); // pose.covariance
        assert_eq!(selection, vec![2, 1]);
        selection = msg_selection.right(&selection); // pose.covariance.0
        assert_eq!(selection, vec![2, 1, 0]);

        selection = vec![2, 1, 35]; // pose.covariance.35
        selection = msg_selection.down(&selection); // pose.covariance.35
        assert_eq!(selection, vec![2, 1, 35]);
    }

    #[test]
    fn test_previous_field_basic_types() {
        let message_type = MessageTypeName {
            package_name: "test_msgs".to_owned(),
            type_name: "BasicTypes".to_owned(),
        };
        let msg = DynamicMessage::new(message_type).unwrap();
        let generic_message = GenericMessage::from(msg.view());
        let msg_selection = GenericMessageSelector::new(&generic_message);

        let mut selection = vec![];
        selection = msg_selection.down(&selection);
        assert_eq!(selection, vec![0]);
        selection = msg_selection.down(&selection);
        assert_eq!(selection, vec![1]);
        selection = msg_selection.down(&selection);
        assert_eq!(selection, vec![2]);
        selection = msg_selection.down(&selection);
        assert_eq!(selection, vec![3]);
        selection = msg_selection.down(&selection);
        assert_eq!(selection, vec![4]);
        selection = msg_selection.down(&selection);
        assert_eq!(selection, vec![5]);

        // Test going back
        selection = msg_selection.up(&selection);
        assert_eq!(selection, vec![4]);
        selection = msg_selection.up(&selection);
        assert_eq!(selection, vec![3]);
        selection = msg_selection.up(&selection);
        assert_eq!(selection, vec![2]);
        selection = msg_selection.up(&selection);
        assert_eq!(selection, vec![1]);
        selection = msg_selection.up(&selection);
        assert_eq!(selection, vec![0]);
    }

    #[test]
    fn test_previous_field_odometry() {
        let message_type = MessageTypeName {
            package_name: "nav_msgs".to_owned(),
            type_name: "Odometry".to_owned(),
        };
        let msg = DynamicMessage::new(message_type).unwrap();
        let generic_message = GenericMessage::from(msg.view());

        let msg_selection = GenericMessageSelector::new(&generic_message);
        let mut selection = vec![2, 0, 1, 0]; // pose.pose.orientation.x
        selection = msg_selection.left(&selection); // pose.pose.orientation
        assert_eq!(selection, vec![2, 0, 1]);
        selection = msg_selection.left(&selection); // pose.pose
        assert_eq!(selection, vec![2, 0]);
        selection = msg_selection.left(&selection); // pose
        assert_eq!(selection, vec![2]);
        selection = msg_selection.left(&selection); // header
        assert_eq!(selection, vec![]);
    }
}
