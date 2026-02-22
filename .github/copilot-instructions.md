# Splinter – AI Coding Instructions

Splinter is a **Ratatui TUI for ROS2**, written in Rust. ROS2 message types are only known at runtime, making dynamic dispatch and runtime typing central to the entire codebase.

## Build & Run

Before using `cargo` from `src/splinter/`, you must source the entire ROS2 workspace with `source /root/ros_ws/install/setup.zsh` to set up the environment.

## Architecture Overview

```
App (common/app.rs)
├── ConnectionType (connections/) — ROS2 executor on background thread
├── Vec<Rc<RefCell<dyn TuiView>>> — open panels, switched with Tab
└── Option<Box<dyn TuiPopup>> — modal overlay
```

**Three component layers**, each with its own trait:

| Layer | Trait | Location | Role |
|---|---|---|---|
| Views | `TuiView` | `src/views/` | Full-screen panels (TopicList, NodeList, RawMessage, TopicPublisher, HzPlot, LivePlot, NodeDetails) |
| Widgets | `TuiWidget` | `src/widgets/` | Reusable sub-components (ListWidget, MessageWidget, EditValueWidget) |
| Popups | `TuiPopup` | `src/popups/` | Modal overlays (NewTopicPopup, NewNodePopup, NewFieldPopup, TextPopup) |

## Event Flow

`App::run()` polls crossterm at 100 ms. Events are dispatched as `Event` enums:

```
crossterm event → Event::Key → active TuiView::handle_event() → returned Event → App::handle_event()
```

Returned `Event` variants route higher-level actions:
- `Event::NewTopic(TopicInfo)` → opens `NewTopicPopupState` (lets user pick RawMessage / TopicPublisher / HzPlot)
- `Event::NewNode(NodeInfo)` → opens `NewNodePopupState`
- `Event::NewField(FieldInfo)` → opens `NewFieldPopupState` → LivePlot
- `Event::NewView(...)` → pushes a new panel onto `App::widgets`
- `Event::ClosePopup` → dismisses the active popup

Global keys handled in `App`: `q`/`Esc` = quit, `Tab`/`BackTab` = cycle panels, `x` = close panel, `?` = help popup (calls `TuiView::get_help_text()`).

## GenericMessage — The Core Abstraction

ROS2 types are runtime-only. The entire message system builds on:

- `InterfaceType` — parses `"package/category/type"` (e.g. `"std_msgs/msg/String"`)
- `GenericMessage` — `IndexMap<String, GenericField>` for ordered, named fields
- `GenericField` — `Simple(SimpleField) | Array(ArrayField) | Sequence(...) | BoundedSequence(...)`
- `GenericMessageSelector` — navigates a message tree with a `Vec<usize>` path (e.g. `[0, 2, 1]` = field[0].field[2].array_element[1])

All views that display or edit messages use `MessageWidget` + `MessageWidgetState` and track cursor position as a `Vec<usize>` stored in `selected_fields`.

## Adding a New View

1. Create `src/views/my_view.rs` with a `MyViewState` (state) and optional `MyViewWidget` (stateless `StatefulWidget`).
2. Implement `TuiView` — all methods required, especially `needs_redraw()` (dirty-flag pattern, must avoid spurious redraws).
3. Implement the appropriate factory trait (`FromTopic`, `FromNode`, `FromConnection`, `FromField`).
4. If the view accepts additional topics/nodes/fields after creation, implement `AcceptsTopic` / `AcceptsNode` / `AcceptsField` and override `TuiView::as_topic_acceptor()` etc.
5. Register in the relevant popup factory map (e.g. `FROM_NEW_TOPIC_FACTORIES` in `src/popups/new_topic_popup.rs`) using `once_cell::sync::Lazy<IndexMap<...>>`.
6. `pub mod my_view;` in `src/views/mod.rs` and import in `src/common/app.rs`.

## Key Conventions

- **State/Widget split**: every view has a `*State` struct and, when needed, a stateless `*Widget` that implements `ratatui::widgets::StatefulWidget<State = *State>`.
- **Styles**: use only `HEADER_STYLE` / `SELECTED_STYLE` from `src/common/style.rs`.
- **Fuzzy search**: all list views use `nucleo-matcher` via `ListWidget<ItemType: ListItemTrait>` in `src/widgets/list_widget.rs`.
- **`ratatui` dependency**: pinned to a fork (`Yomguithereal/ratatui`, branch `fix-334`), not crates.io — do not change this.
- **`Connection` dispatch**: `ConnectionType` uses `enum_dispatch` crate; add new backends to `connections/mod.rs` enum and the `Connection` impl.

## Known Limitations / Active TODOs

- `RawMessageState::new()` busy-waits up to 1 s on the main thread waiting for topic type discovery (see FIXME in `src/views/raw_message.rs`).
- `FROM_NEW_CONNECTION_FACTORIES` in `app.rs` is scaffolding only — multi-connection support is not yet implemented.
- `ConnectionROS2` `Debug` impl panics (`todo!()`).
- Services are not supported until `rclrs` adds DynamicServices.
