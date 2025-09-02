use rclrs::*;
use std::{sync::Arc, thread, time::Duration};

// Node publishing all test_msgs types
struct DevNode {
    publisher_empty: Arc<PublisherState<test_msgs::msg::Empty>>,
    publisher_bounded_plain_sequences: Arc<PublisherState<test_msgs::msg::BoundedPlainSequences>>,
    publisher_bounded_sequences: Arc<PublisherState<test_msgs::msg::BoundedSequences>>,
    publisher_strings: Arc<PublisherState<test_msgs::msg::Strings>>,
    publisher_basic_types: Arc<PublisherState<test_msgs::msg::BasicTypes>>,
    publisher_arrays: Arc<PublisherState<test_msgs::msg::Arrays>>,
    publisher_multi_nested: Arc<PublisherState<test_msgs::msg::MultiNested>>,
    publisher_defaults: Arc<PublisherState<test_msgs::msg::Defaults>>,
    publisher_wstrings: Arc<PublisherState<test_msgs::msg::WStrings>>,
    publisher_unbounded_sequences: Arc<PublisherState<test_msgs::msg::UnboundedSequences>>,
    publisher_builtins: Arc<PublisherState<test_msgs::msg::Builtins>>,
    publisher_constants: Arc<PublisherState<test_msgs::msg::Constants>>,
    publisher_nested: Arc<PublisherState<test_msgs::msg::Nested>>,
}

impl DevNode {
    fn new(executor: &Executor) -> Result<Self, RclrsError> {
        let node = executor.create_node("dev_node").unwrap();

        let publisher_empty = node.create_publisher("empty").unwrap();
        let publisher_bounded_plain_sequences =
            node.create_publisher("bounded_plain_sequences").unwrap();
        let publisher_bounded_sequences = node.create_publisher("bounded_sequences").unwrap();
        let publisher_strings = node.create_publisher("strings").unwrap();
        let publisher_basic_types = node.create_publisher("basic_types").unwrap();
        let publisher_arrays = node.create_publisher("arrays").unwrap();
        let publisher_multi_nested = node.create_publisher("multi_nested").unwrap();
        let publisher_defaults = node.create_publisher("defaults").unwrap();
        let publisher_wstrings = node.create_publisher("w_strings").unwrap();
        let publisher_unbounded_sequences = node.create_publisher("unbounded_sequences").unwrap();
        let publisher_builtins = node.create_publisher("builtins").unwrap();
        let publisher_constants = node.create_publisher("constants").unwrap();
        let publisher_nested = node.create_publisher("nested").unwrap();

        Ok(Self {
            publisher_empty,
            publisher_bounded_plain_sequences,
            publisher_bounded_sequences,
            publisher_strings,
            publisher_basic_types,
            publisher_arrays,
            publisher_multi_nested,
            publisher_defaults,
            publisher_wstrings,
            publisher_unbounded_sequences,
            publisher_builtins,
            publisher_constants,
            publisher_nested,
        })
    }

    fn publish_data(&self) -> Result<(), RclrsError> {
        self.publisher_empty
            .publish(test_msgs::msg::Empty::default())?;
        self.publisher_bounded_plain_sequences
            .publish(test_msgs::msg::BoundedPlainSequences::default())?;
        self.publisher_bounded_sequences
            .publish(test_msgs::msg::BoundedSequences::default())?;
        self.publisher_strings
            .publish(test_msgs::msg::Strings::default())?;
        self.publisher_basic_types
            .publish(test_msgs::msg::BasicTypes::default())?;
        self.publisher_arrays
            .publish(test_msgs::msg::Arrays::default())?;
        self.publisher_multi_nested
            .publish(test_msgs::msg::MultiNested::default())?;
        self.publisher_defaults
            .publish(test_msgs::msg::Defaults::default())?;
        self.publisher_wstrings
            .publish(test_msgs::msg::WStrings::default())?;
        self.publisher_unbounded_sequences
            .publish(test_msgs::msg::UnboundedSequences::default())?;
        self.publisher_builtins
            .publish(test_msgs::msg::Builtins::default())?;
        self.publisher_constants
            .publish(test_msgs::msg::Constants::default())?;
        self.publisher_nested
            .publish(test_msgs::msg::Nested::default())?;
        Ok(())
    }
}

fn main() -> Result<(), RclrsError> {
    let mut executor = Context::default_from_env().unwrap().create_basic_executor();
    let node = DevNode::new(&executor).unwrap();

    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(1000));
        node.publish_data().unwrap();
    });
    executor.spin(SpinOptions::default()).first_error()
}
