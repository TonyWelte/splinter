# Splinter: A ROS2 Terminal User Interface

**Splinter** is a **Terminal User Interface (TUI)** for **ROS2**. It provides a keyboard-driven way to interact with ROS2 topics and nodes from your terminal.

TODO: Add animation

## Features

| Category   | Features                                           |
| ---------- | -------------------------------------------------- |
| **Topics** | List, publish, echo, and monitor message frequency |
| **Nodes**  | List active nodes                                  |
| **Plots**  | Multi-line plots, frequency plots                  |

## Installation Guide

### Manual Installation

1. **Set up ros2_rust**  
    Follow the [installation instructions](https://github.com/ros2-rust/ros2_rust/tree/main?tab=readme-ov-file#sounds-great-how-can-i-try-this-out) for `ros2_rust`.

2. **Clone the Splinter repository**  
    Run the following command to clone the Splinter repository:
    ```sh
    git clone https://github.com/TonyWelte/splinter.git
    ```

3. **Build the project**  
    Use `colcon` to build Splinter:
    ```sh
    colcon build --packages-up-to splinter
    ```

### Quick Installation

> [!WARNING]  
> Review the script before running it. It may prompt for your password to install dependencies. On some systems, it requires running `pip` with the `--break-system-packages` flag.

To quickly install Splinter, download and execute the installation script:

```sh
curl -O https://raw.githubusercontent.com/TonyWelte/splinter/refs/heads/main/tools/splinter_install.sh
chmod +x splinter_install.sh
./splinter_install.sh
```

#### What the Script Does:

- Verifies that a ROS2 environment is sourced.
- Installs Rust (if not already installed) and required system dependencies.
- Creates a `splinter_ws` ROS2 workspace, sets up `ros2_rust` and Splinter, and builds the project.

## Roadmap

### Current Priorities

| Feature                     | Notes                                  |
| --------------------------- | -------------------------------------- |
| Fuzzy search                | Improve usability for large workspaces |
| Parameters (List, Get, Set) | Full parameter management              |

### Upcoming Features

| Feature                                    | Notes                            |
| ------------------------------------------ | -------------------------------- |
| New Plots: Delay plot                      | Visualize message delays         |
| New Plots: Bitrate plot                    | Monitor topic bandwidth          |
| Message Widget: Limit array/sequence sizes | Prevent UI overload              |
| Message Widget: Folding                    | Collapse/expand message sections |

### Long Term Plan

| Feature                                   | Notes                                           |
| ----------------------------------------- | ----------------------------------------------- |
| MCAP Reader                               | Navigate and visualize MCAP files               |
| New Connections: Foxglove bridge          | Remote visualization support                    |
| New Connections: Rosbridge                | Web-based ROS2 interaction                      |
| New Connections: Multi-connection support | Manage multiple ROS2 connections simultaneously |
| Services                                  | List and call ROS2 services                     |
| GenericMessage (non-copying)              | Optimize memory usage                           |
| Grid view                                 | Customizable widget layouts                     |

## Why Rust ?

I picked Rust to learn the language, period. Is it a good choice for this project? Absolutely not.

As of now, the dynamic message support Splinter needs to function isn’t even merged into ros2_rust. If you’re writing a TUI for ROS2, use Python. Save yourself the trouble.

Don’t get me wrong, Rust is a fantastic language. But wrestling with static types for a tool that has to handle messages whose types are only known at runtime? That’s just masochism.

## Where Does the Name Come From?

Splinter is named after the **wise rat sensei** from *Teenage Mutant Ninja Turtles*. It's a nod to both the TUI library **Ratatui** (symbolized by a rat) and **ROS2** (symbolized by turtles).
