# Splinter: A ROS2 Terminal User Interface

**Splinter** is a **Terminal User Interface (TUI)** for **ROS2**. It provides a keyboard-driven way to interact with ROS2 topics and nodes from your terminal.

## Demo

### Topic Graph and Plots

Browse topics using fuzzy search, select a topic to inspect its message fields, and add fields to a live plot.

<img src="./doc/img/splinter-graph.gif"></img>

### State Messages

Browse topics using fuzzy search, select a topic to inspect its message fields, and add fields to a state graph.

<img src="./doc/img/splinter-states.gif"></img>

### Topic Publishing

Fill in a message field by field using arrow keys or `j`/`k`, edit values in place, and publish with `p`.

<img src="./doc/img/splinter-publishing.gif"></img>

### Calling Services

Navigate to a node, pick one of its services, edit the request fields, and call the service.

<img src="./doc/img/splinter-service.gif"></img>

## Features

| Category   | Features                                           |
| ---------- | -------------------------------------------------- |
| **Topics** | List, publish, echo, and monitor message frequency |
| **Nodes**  | List active nodes                                  |
| **Plots**  | Multi-line plots, frequency plots                  |

## Installation Guide

### Debian/Ubuntu Installation

1. **Download the latest release**  
    See https://github.com/TonyWelte/splinter/releases

2. **Install the .deb archive**
    ```sh
    sudo apt-get install ./splinter_VERSION.deb
    ```

3. **Run splinter**
    ```sh
    splinter
    ```

### Manual Installation

1. **Set up ros2_rust**  
    Follow the [installation instructions](https://github.com/ros2-rust/ros2_rust/tree/main?tab=readme-ov-file#sounds-great-how-can-i-try-this-out) for `ros2_rust`.

2. **Clone the Splinter repository**  
    ```sh
    git clone https://github.com/TonyWelte/splinter.git
    ```

3. **Build the project**  
    ```sh
    colcon build --packages-up-to splinter
    ```

4. **Run splinter**
    ```sh
    ros2 run splinter splinter
    ```

## Roadmap

### Upcoming Features

| Feature                 | Notes                            |
| ----------------------- | -------------------------------- |
| New Plots: Delay plot   | Visualize message delays         |
| New Plots: Bitrate plot | Monitor topic bandwidth          |
| Message Widget: Folding | Collapse/expand message sections |
| Parameters (Set List)   |                                  |

### Long Term Plan

| Feature                                   | Notes                                 |
| ----------------------------------------- | ------------------------------------- |
| MCAP Reader                               | Navigate and visualize MCAP files     |
| New Connections: Foxglove bridge          | -                                     |
| New Connections: Rosbridge                | -                                     |
| New Connections: Multi-connection support | Multiple ROS_DOMAIN_ID simultaneously |
| Grid layout                               | Customizable widget layouts           |

## Why Rust ?

I picked Rust to learn the language, period. Is it a good choice for this project? Absolutely not.

If you’re writing a TUI for ROS2, use Python. Save yourself the trouble.

Don’t get me wrong, Rust is a fantastic language. But wrestling with static types for a tool that has to handle messages whose types are only known at runtime? That’s just masochism.

## Where Does the Name Come From?

Splinter is named after the **wise rat sensei** from *Teenage Mutant Ninja Turtles*. It's a nod to both the TUI library **Ratatui** (symbolized by a rat) and **ROS2** (symbolized by turtles).
