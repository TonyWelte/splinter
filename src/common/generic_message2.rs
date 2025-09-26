use rclrs::DynamicMessage;

use crate::common::generic_message::{AnyTypeRef, FieldType, InterfaceType};

pub mod generic_value;

enum GenericMessageInternal {
    Rcl(rclrs::DynamicMessage),
}

struct GenericMessage2 {
    internal: GenericMessageInternal,
}

impl From<rclrs::DynamicMessage> for GenericMessage2 {
    fn from(msg: rclrs::DynamicMessage) -> Self {
        GenericMessage2 {
            internal: GenericMessageInternal::Rcl(msg),
        }
    }
}

impl GenericMessage2 {
    pub fn type_name(&self) -> InterfaceType {
        match &self.internal {
            GenericMessageInternal::Rcl(msg) => {
                let parts = msg.namespace.split("__").collect::<Vec<&str>>();
                InterfaceType {
                    package_name: parts.first().unwrap_or(&"").to_string(),
                    category: parts.get(1).unwrap_or(&"").to_string(),
                    type_name: msg.type_name.clone(),
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        match &self.internal {
            GenericMessageInternal::Rcl(msg) => msg.view().fields.len(),
        }
    }

    pub fn get(&self, field_name: &str) -> Result<generic_value::GenericValue, String> {
        match &self.internal {
            GenericMessageInternal::Rcl(msg) => {
                let field = msg.view().get(field_name).ok_or(format!(
                    "Field '{}' does not exist in message '{}'",
                    field_name, msg.type_name
                ))?;
                Ok(generic_value::GenericValue::from(field))
            }
        }
    }

    pub fn get_index(&self, index: usize) -> Result<generic_value::GenericValue, String> {
        match &self.internal {
            GenericMessageInternal::Rcl(msg) => {
                let msg_view = msg.view();
                let field_info = msg_view
                    .fields
                    .get(index)
                    .ok_or("Index out of bounds".to_string())?;
                let field = msg_view.get(&field_info.name).ok_or(format!(
                    "Field '{}' does not exist in message '{}'",
                    field_info.name, msg.type_name
                ))?;
                Ok(generic_value::GenericValue::from(field))
            }
        }
    }

    pub fn get_deep_index(&'_ self, field_index_path: &[usize]) -> Result<AnyTypeRef<'_>, String> {
        let next_index = field_index_path
            .first()
            .ok_or("Field index path is empty".to_string())?;

        let value = self.get_index(*next_index)?;
        value.get_deep_index(&field_index_path[1..])
    }

    pub fn get_field_type(&self, field_index_path: &[usize]) -> Result<FieldType, String> {
        let next_index = field_index_path
            .first()
            .ok_or("Field index path is empty".to_string())?;

        let value = self.get_index(*next_index)?;
        value.get_field_type(&field_index_path[1..])
    }

    pub fn get_field_name(&self, field_index_path: &[usize]) -> Result<String, String> {
        let next_index = field_index_path
            .first()
            .ok_or("Field index path is empty".to_string())?;

        let value = self.get_index(*next_index)?;
        value.get_field_name(&field_index_path[1..])
    }
}

#[cfg(test)]
mod tests {
    use rclrs::dynamic_message;

    use super::*;

    #[test]
    fn test_generic_message_type_name() {
        let dynamic_message =
            DynamicMessage::new("std_msgs/msg/String".try_into().unwrap()).unwrap();

        let generic_message = GenericMessage2::from(dynamic_message);

        assert_eq!(
            generic_message.type_name(),
            InterfaceType {
                package_name: "std_msgs".to_string(),
                category: "msg".to_string(),
                type_name: "String".to_string(),
            }
        );
    }

    #[test]
    fn test_generic_message_len() {
        let dynamic_message =
            DynamicMessage::new("std_msgs/msg/String".try_into().unwrap()).unwrap();
        let generic_message = GenericMessage2::from(dynamic_message);

        assert_eq!(generic_message.len(), 1); // String message has one field: "data"

        let dynamic_message =
            DynamicMessage::new("geometry_msgs/msg/Point".try_into().unwrap()).unwrap();
        let generic_message = GenericMessage2::from(dynamic_message);

        assert_eq!(generic_message.len(), 3); // Point message has three fields: "x", "y", "z"
    }

    #[test]
    fn test_generic_message_get() {
        let dynamic_message =
            DynamicMessage::new("test_msgs/msg/Nested".try_into().unwrap()).unwrap();
        let generic_message = GenericMessage2::from(dynamic_message);

        // Test getting an existing field
        let field = generic_message.get("basic_types_value").unwrap();
        // TODO: Further assertions can be made on the field

        // Test getting a non-existing field
        let err = generic_message.get("non_existing_field").err().unwrap();
        assert_eq!(
            err,
            "Field 'non_existing_field' does not exist in message 'Nested'"
        );
    }
}
