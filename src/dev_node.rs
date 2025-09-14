use rclrs::*;
use std::{sync::Arc, thread, time::Duration};

// Node publishing all test_msgs types
struct DevNode {
    param_bool: OptionalParameter<bool>,
    param_integer: OptionalParameter<i64>,
    param_double: OptionalParameter<f64>,
    param_string: OptionalParameter<Arc<str>>,
    param_bools: OptionalParameter<Arc<[bool]>>,
    param_integers: OptionalParameter<Arc<[i64]>>,
    param_doubles: OptionalParameter<Arc<[f64]>>,
    param_strings: OptionalParameter<Arc<[Arc<str>]>>,
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
    publisher_sinusoid: Arc<PublisherState<test_msgs::msg::MultiNested>>,
}

impl DevNode {
    fn new(executor: &Executor) -> Result<Self, RclrsError> {
        let node = executor.create_node("dev_node").unwrap();

        let param_bool = node
            .declare_parameter("param_bool")
            .default(true)
            .optional()
            .unwrap();
        let param_integer = node
            .declare_parameter("param_integer")
            .default(42)
            .optional()
            .unwrap();
        let param_double = node
            .declare_parameter("param_double")
            .default(3.14)
            .optional()
            .unwrap();
        let param_string = node
            .declare_parameter::<Arc<str>>("param_string")
            .default(Arc::from("hello world"))
            .optional()
            .unwrap();
        let param_bools = node
            .declare_parameter::<Arc<[bool]>>("param_bools")
            .default(Arc::from([true, false, true]))
            .optional()
            .unwrap();
        let param_integers = node
            .declare_parameter::<Arc<[i64]>>("param_integers")
            .default(Arc::from([1, 2, 3, 4, 5]))
            .optional()
            .unwrap();
        let param_doubles = node
            .declare_parameter::<Arc<[f64]>>("param_doubles")
            .default(Arc::from([1.1, 2.2, 3.3]))
            .optional()
            .unwrap();
        let param_strings = node
            .declare_parameter::<Arc<[Arc<str>]>>("param_strings")
            .default(Arc::from([
                Arc::from("foo"),
                Arc::from("bar"),
                Arc::from("baz"),
            ]))
            .optional()
            .unwrap();

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

        let publisher_sinusoid = node.create_publisher("sinusoid").unwrap();

        Ok(Self {
            param_bool,
            param_integer,
            param_double,
            param_string,
            param_bools,
            param_integers,
            param_doubles,
            param_strings,
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
            publisher_sinusoid,
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

        let basic_msg = test_msgs::msg::BasicTypes {
            bool_value: self.param_bool.get().unwrap_or(false),
            byte_value: 0,
            char_value: 0,
            float32_value: self.param_double.get().unwrap_or(0.0) as f32,
            float64_value: self.param_double.get().unwrap_or(0.0),
            int8_value: self.param_integer.get().unwrap_or(0) as i8,
            int16_value: self.param_integer.get().unwrap_or(0) as i16,
            int32_value: self.param_integer.get().unwrap_or(0) as i32,
            int64_value: self.param_integer.get().unwrap_or(0),
            uint8_value: self.param_integer.get().unwrap_or(0) as u8,
            uint16_value: self.param_integer.get().unwrap_or(0) as u16,
            uint32_value: self.param_integer.get().unwrap_or(0) as u32,
            uint64_value: self.param_integer.get().unwrap_or(0) as u64,
        };
        self.publisher_basic_types.publish(basic_msg)?;

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

    fn publish_signal(&self, t: f64) -> Result<(), RclrsError> {
        let mut msg = test_msgs::msg::MultiNested::default();
        msg.array_of_arrays[0].float64_values = [t.sin(), t.cos(), t.tan()];
        self.publisher_sinusoid.publish(msg)?;
        Ok(())
    }
}

fn main() -> Result<(), RclrsError> {
    let mut executor = Context::default_from_env().unwrap().create_basic_executor();
    let node = DevNode::new(&executor).unwrap();

    thread::spawn(move || {
        let mut t = 0.0;
        loop {
            thread::sleep(Duration::from_millis(100));
            node.publish_signal(t).unwrap();
            t += 0.1;

            // Publish all other test messages at a slower rate
            if (t * 10.0) as i32 % 10 == 0 {
                node.publish_data().unwrap();
            }
        }
    });
    executor.spin(SpinOptions::default()).first_error()
}
