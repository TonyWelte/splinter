use std::usize;

use crate::common::generic_message::{
    ArrayField, BoundedSequenceField, GenericField, GenericMessage, Length, SequenceField,
    SimpleField,
};

pub struct GenericMessageSelector<'a> {
    message: &'a GenericMessage,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FieldCategory {
    Base,
    Message,
    Array,
    Sequence,
    BoundedSequence,
}

pub fn get_field_category(message: &GenericMessage, field_path: &[usize]) -> Option<FieldCategory> {
    if field_path.len() == 0 {
        return Some(FieldCategory::Message);
    }

    if field_path.len() == 1 {
        if field_path[0] < message.len() {
            return match message.get_index(field_path[0]).unwrap() {
                GenericField::Simple(SimpleField::Message(_)) => Some(FieldCategory::Message),
                GenericField::Simple(_) => Some(FieldCategory::Base),
                GenericField::Array(_) => Some(FieldCategory::Array),
                GenericField::Sequence(_) => Some(FieldCategory::Sequence),
                GenericField::BoundedSequence(_) => Some(FieldCategory::BoundedSequence),
            };
        } else {
            return None;
        }
    }

    let field_index = field_path[0];
    if field_index >= message.len() {
        return None;
    }

    let field = message.get_index(field_index).unwrap();
    match field {
        GenericField::Simple(simple_value) => {
            if let SimpleField::Message(inner_msg) = simple_value {
                return get_field_category(&inner_msg, &field_path[1..]);
            } else {
                return None;
            }
        }
        GenericField::Array(array_value) => match array_value {
            ArrayField::Message(inner_msgs) => {
                if field_path[1] >= inner_msgs.len() {
                    return None;
                }
                if field_path.len() == 2 {
                    return Some(FieldCategory::Message);
                }
                return get_field_category(&inner_msgs[field_path[1]], &field_path[2..]);
            }
            _ => {
                if field_path.len() == 2 && field_path[1] < array_value.len() {
                    return Some(FieldCategory::Base);
                } else {
                    return None;
                }
            }
        },
        GenericField::Sequence(sequence_value) => match sequence_value {
            SequenceField::Message(inner_msgs) => {
                if field_path[1] >= inner_msgs.len() {
                    return None;
                }
                if field_path.len() == 2 {
                    return Some(FieldCategory::Message);
                }
                return get_field_category(&inner_msgs[field_path[1]], &field_path[2..]);
            }
            _ => {
                if field_path.len() == 2 && field_path[1] < sequence_value.len() {
                    return Some(FieldCategory::Base);
                } else {
                    return None;
                }
            }
        },
        GenericField::BoundedSequence(bounded_sequence_value) => match bounded_sequence_value {
            BoundedSequenceField::Message(inner_msgs, _) => {
                if field_path[1] >= inner_msgs.len() {
                    return None;
                }
                if field_path.len() == 2 {
                    return Some(FieldCategory::Message);
                }
                return get_field_category(&inner_msgs[field_path[1]], &field_path[2..]);
            }
            _ => {
                if field_path.len() == 2 && field_path[1] < bounded_sequence_value.len() {
                    return Some(FieldCategory::Base);
                } else {
                    return None;
                }
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
                if field_path.len() >= 2 && field_path[1] >= sequence_value.len() {
                    return None;
                }
                if field_path.len() == 2 {
                    return Some(vec![field_index, field_path[1]]);
                } else {
                    if sequence_value.len() == 0 {
                        return None;
                    }
                    return Some(vec![field_index, sequence_value.len() - 1]);
                }
            }
        },
        GenericField::BoundedSequence(bounded_sequence_value) => match bounded_sequence_value {
            BoundedSequenceField::Message(inner_msgs, _) => {
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
                if field_path.len() >= 2 && field_path[1] >= bounded_sequence_value.len() {
                    return None;
                }
                if field_path.len() == 2 {
                    return Some(vec![field_index, field_path[1]]);
                } else {
                    if bounded_sequence_value.len() == 0 {
                        return None;
                    }
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
        if get_field_category(&self.message, &result).is_some() {
            return result;
        }
        result.pop();

        // Try to go down
        result.last_mut().map(|last| *last += 1);
        while !result.is_empty() && get_field_category(&self.message, &result).is_none() {
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
        if get_field_category(&self.message, &result).is_some() {
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

        assert!(get_field_category(&generic_message, &vec![]).is_some()); // Root always exists
        assert!(get_field_category(&generic_message, &vec![0]).is_some()); // header
        assert!(get_field_category(&generic_message, &vec![2, 0, 1, 0]).is_some()); // pose.pose.orientation.x
        assert!(get_field_category(&generic_message, &vec![2, 1, 35]).is_some()); // pose.covariance.35

        // Out of bounds
        assert!(get_field_category(&generic_message, &vec![10]).is_none());
        assert!(get_field_category(&generic_message, &vec![2, 5]).is_none());
        assert!(get_field_category(&generic_message, &vec![2, 0, 5]).is_none());
        assert!(get_field_category(&generic_message, &vec![2, 0, 1, 5]).is_none());
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
        selection = msg_selection.down(&selection); // header.stamp
        assert_eq!(selection, vec![0, 0]);
        selection = msg_selection.down(&selection); // header.stamp.sec
        assert_eq!(selection, vec![0, 0, 0]);
        selection = msg_selection.down(&selection); // header.stamp.nanosec
        assert_eq!(selection, vec![0, 0, 1]);
        selection = msg_selection.down(&selection); // header.frame_id
        assert_eq!(selection, vec![0, 1]);
        selection = msg_selection.down(&selection); // child_frame_id
        assert_eq!(selection, vec![1]);
        selection = msg_selection.down(&selection); // pose
        assert_eq!(selection, vec![2]);
        selection = msg_selection.down(&selection); // pose.pose
        assert_eq!(selection, vec![2, 0]);
        selection = msg_selection.down(&selection); // pose.pose.position
        assert_eq!(selection, vec![2, 0, 0]);

        selection = vec![2, 1];
        selection = msg_selection.down(&selection); // pose.covariance.0
        assert_eq!(selection, vec![2, 1, 0]);
        selection = msg_selection.down(&selection); // pose.covariance.1
        assert_eq!(selection, vec![2, 1, 1]);
        selection = msg_selection.down(&selection); // pose.covariance.2
        assert_eq!(selection, vec![2, 1, 2]);

        selection = vec![2, 1, 35]; // pose.covariance.35
        selection = msg_selection.down(&selection); // pose.covariance.35
        assert_eq!(selection, vec![3]);
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
        selection = msg_selection.up(&selection); // pose.pose.orientation
        assert_eq!(selection, vec![2, 0, 1]);
        selection = msg_selection.up(&selection); // pose.pose.position.z
        assert_eq!(selection, vec![2, 0, 0, 2]);
        selection = msg_selection.up(&selection); // pose.pose.position.y
        assert_eq!(selection, vec![2, 0, 0, 1]);
        selection = msg_selection.up(&selection); // pose.pose.position.x
        assert_eq!(selection, vec![2, 0, 0, 0]);
        selection = msg_selection.up(&selection); // pose.pose.position
        assert_eq!(selection, vec![2, 0, 0]);
        selection = msg_selection.up(&selection); // pose.pose
        assert_eq!(selection, vec![2, 0]);
        selection = msg_selection.up(&selection); // pose
        assert_eq!(selection, vec![2]);
    }
}
