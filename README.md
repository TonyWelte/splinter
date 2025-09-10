# Splinter: A ROS2 Terminal User Interface

**Splinter** is a **Terminal User Interface (TUI)** for **ROS2**. It provides a keyboard-driven way to interact with ROS2 topics and nodes from your terminal.

TODO: Add animation

## Features

| Category   | Features                                           |
| ---------- | -------------------------------------------------- |
| **Topics** | List, publish, echo, and monitor message frequency |
| **Nodes**  | List active nodes                                  |
| **Plots**  | Multi-line plots, frequency plots                  |

## Installing

### Requirements

- **ROS2**
- **Rust/Cargo**

### Quick Install

Run the following command to install Splinter automatically:

```sh
curl https://raw.githubusercontent.com/TonyWelte/splinter/refs/heads/develop/tools/splinter_install.sh | bash
```

**What the script does:**

- Checks for a sourced ROS2 environment.
- Installs Rust (if missing) and required system packages.
- Sets up a ROS2 workspace with ros2_rust and splinter, and builds the project.

## Roadmap

### Current Priorities

| Feature                     | Notes                                  |
| --------------------------- | -------------------------------------- |
| Search/Filter topics/nodes  | Improve usability for large workspaces |
| Parameters (List, Get, Set) | Full parameter management              |

### Upcoming Features

| Feature                                    | Notes                            |
| ------------------------------------------ | -------------------------------- |
| New Plots: Delay plot                      | Visualize message delays         |
| New Plots: Bitrate plot                    | Monitor topic bandwidth          |
| Message Widget: Limit array/sequence sizes | Prevent UI overload              |
| Message Widget: Folding                    | Collapse/expand message sections |

### Long Term Plan

| Feature                                   | Notes                         |
| ----------------------------------------- | ----------------------------- |
| New Connections: Foxglove bridge          | Remote visualization support  |
| New Connections: Rosbridge                | Web-based ROS2 interaction    |
| New Connections: Multi-connection support | Manage multiple ROS2 networks |
| Services                                  | List and call ROS2 services   |
| GenericMessage (non-copying)              | Optimize memory usage         |
| Grid view                                 | Customizable widget layouts   |

## Why Rust ?

I picked Rust to learn the language, period. Is it a good choice for this project? Absolutely not.

As of now, the dynamic message support Splinter needs to function isn’t even merged into ros2_rust. If you’re writing a TUI for ROS2, use Python. Save yourself the trouble.

Don’t get me wrong, Rust is a fantastic language. But wrestling with static types for a tool that has to handle messages whose types are only known at runtime? That’s just masochism.

## Where Does the Name Come From?

Splinter is named after the **wise rat sensei** from *Teenage Mutant Ninja Turtles*. It's a nod to both the TUI library **Ratatui** (symbolized by a rat) and **ROS2** (symbolized by turtles).
