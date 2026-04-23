use std::collections::{HashMap, HashSet};

use super::generic_message::{GenericMessage, InterfaceType};

// ---------------------------------------------------------------------------
// Primitive type descriptor
// ---------------------------------------------------------------------------

/// Schema-level descriptor for a single primitive type, mirroring the value
/// variants of [`SimpleField`](super::generic_message::SimpleField) but
/// without carrying any data. Used to describe the shape of a message field
/// before any binary data is received.
///
/// `BoundedString` and `BoundedWString` carry their upper-bound length.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrimitiveKind {
    Float32,
    Float64,
    LongDouble,
    Char,
    WChar,
    Boolean,
    Octet,
    Uint8,
    Int8,
    Uint16,
    Int16,
    Uint32,
    Int32,
    Uint64,
    Int64,
    String,
    BoundedString(usize),
    WString,
    BoundedWString(usize),
}

// ---------------------------------------------------------------------------
// Field type and multiplicity
// ---------------------------------------------------------------------------

/// What type the values of a field have.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldTypeKind {
    /// A built-in primitive type.
    Primitive(PrimitiveKind),
    /// A nested message type, identified by its `InterfaceType`. The
    /// referenced definition must be present in the [`MessageDefinitionStore`]
    /// before binary parsing can succeed.
    Message(InterfaceType),
}

/// How many values a field carries — directly mirrors the outer variants of
/// [`GenericField`](super::generic_message::GenericField).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldMultiplicity {
    /// A single value.
    Single,
    /// A fixed-size array (length known at schema time).
    Array(usize),
    /// An unbounded dynamic sequence.
    Sequence,
    /// A dynamic sequence with a maximum length.
    BoundedSequence(usize),
}

/// Full schema descriptor for one field inside a [`MessageDefinition`].
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDefinition {
    pub kind: FieldTypeKind,
    pub multiplicity: FieldMultiplicity,
}

impl FieldDefinition {
    pub fn new(kind: FieldTypeKind, multiplicity: FieldMultiplicity) -> Self {
        Self { kind, multiplicity }
    }
}

// ---------------------------------------------------------------------------
// MessageDefinition
// ---------------------------------------------------------------------------

/// Schema-level analog of [`GenericMessage`]: describes the *shape* of a
/// message type without holding any values.
///
/// Fields are stored in declaration order as `(name, definition)` pairs.
/// Duplicate field names are not validated here — callers should ensure
/// uniqueness.
#[derive(Debug, Clone, PartialEq)]
pub struct MessageDefinition {
    pub type_name: InterfaceType,
    pub fields: Vec<(String, FieldDefinition)>,
}

impl MessageDefinition {
    pub fn new(type_name: InterfaceType, fields: Vec<(String, FieldDefinition)>) -> Self {
        Self { type_name, fields }
    }
}

// ---------------------------------------------------------------------------
// MessageDefinitionStore
// ---------------------------------------------------------------------------

/// A registry that maps [`InterfaceType`] strings to [`MessageDefinition`]s.
///
/// Connections that provide binary message data should populate the store from
/// the schema strings they receive, then hand it to a [`BinaryMessageParser`]
/// to reconstruct [`GenericMessage`] instances at runtime.
///
/// # Ordering
/// Registration order does not matter — definitions may reference types that
/// have not yet been registered. Call [`validate_completeness`] after all
/// definitions have been registered to check for missing dependencies.
///
/// [`validate_completeness`]: MessageDefinitionStore::validate_completeness
#[derive(Debug, Default)]
pub struct MessageDefinitionStore {
    definitions: HashMap<String, MessageDefinition>,
}

impl MessageDefinitionStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a [`MessageDefinition`]. If a definition for the same type was
    /// already present it is replaced — the most recently registered definition
    /// wins.
    pub fn register(&mut self, def: MessageDefinition) {
        self.definitions.insert(def.type_name.to_string(), def);
    }

    /// Look up a definition by its [`InterfaceType`].
    pub fn get(&self, type_name: &InterfaceType) -> Option<&MessageDefinition> {
        self.definitions.get(&type_name.to_string())
    }

    /// Returns `true` if a definition for `type_name` has been registered.
    pub fn contains(&self, type_name: &InterfaceType) -> bool {
        self.definitions.contains_key(&type_name.to_string())
    }

    /// Walk the full dependency tree of `root` and collect every
    /// `FieldTypeKind::Message` reference that has no corresponding
    /// registered definition.
    ///
    /// Returns `Ok(())` when the tree is complete, or `Err(missing)` where
    /// `missing` is the de-duplicated list of unresolved [`InterfaceType`]s.
    ///
    /// # Cycle safety
    /// Types that have already been visited are skipped, so mutually recursive
    /// definitions (`A` contains `B` contains `A`) do not cause infinite
    /// recursion.
    pub fn validate_completeness(&self, root: &InterfaceType) -> Result<(), Vec<InterfaceType>> {
        let mut missing: Vec<InterfaceType> = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();
        self.collect_missing(root, &mut visited, &mut missing);
        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }

    // DFS helper — visits `current`, then recurses into all
    // `FieldTypeKind::Message` fields it finds.
    fn collect_missing(
        &self,
        current: &InterfaceType,
        visited: &mut HashSet<String>,
        missing: &mut Vec<InterfaceType>,
    ) {
        let key = current.to_string();
        if !visited.insert(key) {
            // Already processed — skip to avoid cycles.
            return;
        }
        match self.definitions.get(&current.to_string()) {
            None => {
                missing.push(current.clone());
            }
            Some(def) => {
                for (_name, field_def) in &def.fields {
                    if let FieldTypeKind::Message(nested) = &field_def.kind {
                        self.collect_missing(nested, visited, missing);
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Parser traits
// ---------------------------------------------------------------------------

/// Parses raw binary data into a [`GenericMessage`] using the type
/// definitions held in a [`MessageDefinitionStore`].
///
/// One implementation per binary encoding (CDR, custom, etc.) should be
/// provided by the corresponding connection module.
pub trait BinaryMessageParser {
    /// Parse `data` as an instance of `root_type`.
    ///
    /// The `store` must contain a complete dependency tree for `root_type`
    /// (i.e. [`MessageDefinitionStore::validate_completeness`] must return
    /// `Ok(())`); implementations may return an `Err` if a required nested
    /// type is missing.
    fn parse(
        &self,
        data: &[u8],
        root_type: &InterfaceType,
        store: &MessageDefinitionStore,
    ) -> Result<GenericMessage, String>;
}

/// Parses a schema description string into one or more [`MessageDefinition`]s.
///
/// One implementation per schema format (ROS2 `.msg`, IDL, custom, etc.)
/// should be provided by the corresponding connection module.
///
/// A single schema string may define multiple types (e.g. MCAP's `ros2msg`
/// format separates them with `===` markers), so the method returns a
/// `Vec<MessageDefinition>`. Callers should pass every returned definition
/// to [`MessageDefinitionStore::register`].
pub trait SchemaStringParser {
    /// Parse `schema` into message definitions.
    ///
    /// `root_type` identifies the primary / outermost type described by the
    /// schema string. Parsers that can infer the type name from the schema
    /// itself may ignore this argument; others use it as the identity for
    /// the first definition in the returned list.
    fn parse_schema(
        &self,
        schema: &str,
        root_type: InterfaceType,
    ) -> Result<Vec<MessageDefinition>, String>;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_type(name: &str) -> InterfaceType {
        InterfaceType::new(name).unwrap()
    }

    fn float_field() -> FieldDefinition {
        FieldDefinition::new(
            FieldTypeKind::Primitive(PrimitiveKind::Float64),
            FieldMultiplicity::Single,
        )
    }

    fn msg_field(t: &str) -> FieldDefinition {
        FieldDefinition::new(
            FieldTypeKind::Message(make_type(t)),
            FieldMultiplicity::Single,
        )
    }

    // ------------------------------------------------------------------
    // Store registration
    // ------------------------------------------------------------------

    #[test]
    fn register_and_get() {
        let mut store = MessageDefinitionStore::new();
        let def = MessageDefinition::new(
            make_type("std_msgs/msg/Float64"),
            vec![("data".to_string(), float_field())],
        );
        store.register(def.clone());
        assert!(store.contains(&make_type("std_msgs/msg/Float64")));
        assert_eq!(store.get(&make_type("std_msgs/msg/Float64")), Some(&def));
    }

    #[test]
    fn latest_registration_wins() {
        let mut store = MessageDefinitionStore::new();
        let t = make_type("std_msgs/msg/Float64");
        let def1 = MessageDefinition::new(t.clone(), vec![("x".to_string(), float_field())]);
        let def2 = MessageDefinition::new(t.clone(), vec![("y".to_string(), float_field())]);
        store.register(def1);
        store.register(def2.clone());
        assert_eq!(store.get(&t), Some(&def2));
    }

    // ------------------------------------------------------------------
    // validate_completeness
    // ------------------------------------------------------------------

    #[test]
    fn complete_flat_definition() {
        let mut store = MessageDefinitionStore::new();
        store.register(MessageDefinition::new(
            make_type("std_msgs/msg/Float64"),
            vec![("data".to_string(), float_field())],
        ));
        assert!(store
            .validate_completeness(&make_type("std_msgs/msg/Float64"))
            .is_ok());
    }

    #[test]
    fn complete_nested_definition() {
        let mut store = MessageDefinitionStore::new();
        store.register(MessageDefinition::new(
            make_type("std_msgs/msg/Float64"),
            vec![("data".to_string(), float_field())],
        ));
        store.register(MessageDefinition::new(
            make_type("geometry_msgs/msg/Point"),
            vec![
                ("x".to_string(), msg_field("std_msgs/msg/Float64")),
                ("y".to_string(), msg_field("std_msgs/msg/Float64")),
                ("z".to_string(), msg_field("std_msgs/msg/Float64")),
            ],
        ));
        assert!(store
            .validate_completeness(&make_type("geometry_msgs/msg/Point"))
            .is_ok());
    }

    #[test]
    fn missing_nested_type() {
        let mut store = MessageDefinitionStore::new();
        // Register parent but NOT the nested type it references.
        store.register(MessageDefinition::new(
            make_type("geometry_msgs/msg/Point"),
            vec![("inner".to_string(), msg_field("std_msgs/msg/Float64"))],
        ));
        let err = store
            .validate_completeness(&make_type("geometry_msgs/msg/Point"))
            .unwrap_err();
        assert_eq!(err, vec![make_type("std_msgs/msg/Float64")]);
    }

    #[test]
    fn missing_root_type() {
        let store = MessageDefinitionStore::new();
        let err = store
            .validate_completeness(&make_type("pkg/msg/Missing"))
            .unwrap_err();
        assert_eq!(err, vec![make_type("pkg/msg/Missing")]);
    }

    #[test]
    fn cycle_does_not_loop_forever() {
        // A → B → A: both are registered; should complete without panic.
        let mut store = MessageDefinitionStore::new();
        store.register(MessageDefinition::new(
            make_type("pkg/msg/A"),
            vec![("b".to_string(), msg_field("pkg/msg/B"))],
        ));
        store.register(MessageDefinition::new(
            make_type("pkg/msg/B"),
            vec![("a".to_string(), msg_field("pkg/msg/A"))],
        ));
        assert!(store.validate_completeness(&make_type("pkg/msg/A")).is_ok());
    }
}
