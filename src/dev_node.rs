use rclrs::*;
use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

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
    publisher_long_name: Arc<PublisherState<test_msgs::msg::BasicTypes>>,
    publisher_states: Arc<PublisherState<test_msgs::msg::BasicTypes>>,
    subcriber_odometry: Arc<SubscriptionState<nav_msgs::msg::Odometry, Arc<NodeState>>>,
    service_basic_types: Arc<ServiceState<test_msgs::srv::BasicTypes, Arc<NodeState>>>,
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
            .default(std::f64::consts::PI)
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

        let publisher_long_name = node
            .create_publisher("/long/name/to/test/the/display/truncation/in/the/ui/component")
            .unwrap();

        let publisher_states = node.create_publisher("states").unwrap();

        let subcriber_odometry = node
            .create_subscription("odometry", |msg: nav_msgs::msg::Odometry| {
                println!(
                    "Received odometry message: position=({:.2}, {:.2}, {:.2})",
                    msg.pose.pose.position.x, msg.pose.pose.position.y, msg.pose.pose.position.z
                );
            })
            .unwrap();

        let service_basic_types = node
            .create_service(
                "basic_types",
                |request: test_msgs::srv::BasicTypes_Request| {
                    println!(
                        "Received service request: bool_value={}",
                        request.bool_value
                    );
                    test_msgs::srv::BasicTypes_Response {
                        bool_value: !request.bool_value,
                        byte_value: request.byte_value + 1,
                        char_value: request.char_value + 1,
                        float32_value: request.float32_value + 1.0,
                        float64_value: request.float64_value + 1.0,
                        int8_value: request.int8_value + 1,
                        int16_value: request.int16_value + 1,
                        int32_value: request.int32_value + 1,
                        int64_value: request.int64_value + 1,
                        uint8_value: request.uint8_value + 1,
                        uint16_value: request.uint16_value + 1,
                        uint32_value: request.uint32_value + 1,
                        uint64_value: request.uint64_value + 1,
                        string_value: format!("{} response", request.string_value),
                    }
                },
            )
            .unwrap();

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
            publisher_long_name,
            publisher_states,
            subcriber_odometry,
            service_basic_types,
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

        self.publisher_long_name
            .publish(test_msgs::msg::BasicTypes::default())?;
        Ok(())
    }

    fn publish_signal(&self, t: f64) -> Result<(), RclrsError> {
        let mut msg = test_msgs::msg::MultiNested::default();
        msg.array_of_arrays[0].float64_values = [t.sin(), t.cos(), t.tan()];
        self.publisher_sinusoid.publish(msg)?;
        Ok(())
    }

    /// Publish state values with random transitions on a single `BasicTypes` message.
    /// Each of the 13 fields transitions independently at a random interval drawn
    /// from an exponential-ish distribution (some very short, producing bursts).
    ///
    /// Fields and their state vocabularies:
    /// | field          | vocabulary                          |
    /// |----------------|-------------------------------------|
    /// | bool_value     | false / true                        |
    /// | byte_value     | 0–4                                 |
    /// | char_value     | 0–3                                 |
    /// | float32_value  | 0.0 / 0.25 / 0.5 / 0.75 / 1.0      |
    /// | float64_value  | 0.0 / 1.0 / 2.0 / 3.0              |
    /// | int8_value     | -2 / -1 / 0 / 1 / 2 / 3            |
    /// | uint8_value    | 0–5                                 |
    /// | int16_value    | 0 / 100 / 200 / 300                 |
    /// | uint16_value   | 0 / 10 / 20 / 30 / 40               |
    /// | int32_value    | 0–3  (mirrors original state_int)   |
    /// | uint32_value   | 0–4                                 |
    /// | int64_value    | 0–5                                 |
    /// | uint64_value   | 0–3                                 |
    fn publish_states(
        &self,
        rng: &mut u64,
        state_msg: &mut test_msgs::msg::BasicTypes,
        next_changes: &mut [f64; 13],
        t: f64,
    ) -> Result<(), RclrsError> {
        if t >= next_changes[0] {
            state_msg.bool_value = (lcg_next(rng) % 2) == 1;
            next_changes[0] = t + random_interval(rng);
        }
        if t >= next_changes[1] {
            state_msg.byte_value = (lcg_next(rng) % 5) as u8;
            next_changes[1] = t + random_interval(rng);
        }
        if t >= next_changes[2] {
            state_msg.char_value = (lcg_next(rng) % 4) as u8;
            next_changes[2] = t + random_interval(rng);
        }
        if t >= next_changes[3] {
            let levels: [f32; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];
            state_msg.float32_value = levels[lcg_next(rng) as usize % levels.len()];
            next_changes[3] = t + random_interval(rng);
        }
        if t >= next_changes[4] {
            let levels: [f64; 4] = [0.0, 1.0, 2.0, 3.0];
            state_msg.float64_value = levels[lcg_next(rng) as usize % levels.len()];
            next_changes[4] = t + random_interval(rng);
        }
        if t >= next_changes[5] {
            let states: [i8; 6] = [-2, -1, 0, 1, 2, 3];
            state_msg.int8_value = states[lcg_next(rng) as usize % states.len()];
            next_changes[5] = t + random_interval(rng);
        }
        if t >= next_changes[6] {
            state_msg.uint8_value = (lcg_next(rng) % 6) as u8;
            next_changes[6] = t + random_interval(rng);
        }
        if t >= next_changes[7] {
            let states: [i16; 4] = [0, 100, 200, 300];
            state_msg.int16_value = states[lcg_next(rng) as usize % states.len()];
            next_changes[7] = t + random_interval(rng);
        }
        if t >= next_changes[8] {
            state_msg.uint16_value = (lcg_next(rng) % 5 * 10) as u16;
            next_changes[8] = t + random_interval(rng);
        }
        if t >= next_changes[9] {
            state_msg.int32_value = (lcg_next(rng) % 4) as i32;
            next_changes[9] = t + random_interval(rng);
        }
        if t >= next_changes[10] {
            state_msg.uint32_value = (lcg_next(rng) % 5) as u32;
            next_changes[10] = t + random_interval(rng);
        }
        if t >= next_changes[11] {
            state_msg.int64_value = (lcg_next(rng) % 6) as i64;
            next_changes[11] = t + random_interval(rng);
        }
        if t >= next_changes[12] {
            state_msg.uint64_value = lcg_next(rng) % 4;
            next_changes[12] = t + random_interval(rng);
        }

        self.publisher_states.publish(state_msg.clone())?;
        Ok(())
    }
}

/// Simple linear congruential generator (LCG) PRNG.
/// Returns the next pseudo-random value and advances the state.
fn lcg_next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state >> 33
}

/// Produce a random interval with a distribution biased towards short bursts.
/// Returns values in the range \[0.05, 4.0\] seconds.
/// Roughly 20% of the time the interval is ≤0.3s, which — at the 100ms
/// publish rate — puts 3+ state transitions inside a single character cell.
fn random_interval(rng: &mut u64) -> f64 {
    let r = (lcg_next(rng) % 1000) as f64 / 1000.0; // uniform in [0, 1)
                                                    // Exponential-ish mapping: many short intervals, some long ones.
                                                    // 0.05 + 3.95 * r^2  →  r=0 ⇒ 0.05s, r=0.27 ⇒ ~0.34s, r=1 ⇒ 4.0s
    0.05 + 3.95 * r * r
}

fn main() -> Result<(), RclrsError> {
    let mut executor = Context::default_from_env().unwrap().create_basic_executor();
    let node = DevNode::new(&executor).unwrap();

    thread::spawn(move || {
        let start = Instant::now();
        // Seed the PRNG from the current time
        let mut rng: u64 = start.elapsed().as_nanos() as u64 ^ 0xDEAD_BEEF;
        let mut state_msg = test_msgs::msg::BasicTypes::default();
        let mut next_changes = [0.0f64; 13];

        let mut t = 0.0;
        loop {
            thread::sleep(Duration::from_millis(100));
            node.publish_signal(t).unwrap();
            node.publish_states(&mut rng, &mut state_msg, &mut next_changes, t)
                .unwrap();
            t += 0.1;

            // Publish all other test messages at a slower rate
            if (t * 10.0) as i32 % 10 == 0 {
                node.publish_data().unwrap();
            }
        }
    });
    executor.spin(SpinOptions::default()).first_error()
}
