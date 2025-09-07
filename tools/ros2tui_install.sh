#!/bin/bash
set -e

# Function to print error messages and exit
error_exit() {
    echo "$1" 1>&2
    exit 1
}

# Check if running as root
if [ "$(id -u)" -eq 0 ]; then
    error_exit "This script should not be run as root. Please run as a normal user."
fi

# Check if ROS_DISTRO is set
if [ -z "$ROS_DISTRO" ]; then
    error_exit "ROS_DISTRO is not set. Please source your ROS 2 environment before running this script."
fi

# Install Rust
if ! command -v rustc &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "Rust is already installed."
fi

# Install required system packages
echo "Installing required system packages..."
sudo apt update
sudo apt install -y git libclang-dev python3-pip python3-vcstool

# Install colcon plugins for Rust
echo "Installing colcon plugins for Rust..."
pip install git+https://github.com/colcon/colcon-cargo.git
pip install git+https://github.com/colcon/colcon-ros-cargo.git

# Setup workspace and clone ros2_rust in the current directory
echo "Setting up workspace and cloning ros2_rust..."
mkdir -p ./ros2tui_ws/src
cd ./ros2tui_ws

# Clone ros2tui and ros2_rust repositories
git clone --branch develop https://github.com/Tonywelte/ros2tui.git src/ros2tui
vcs import src < src/ros2tui/tools/packages.repos
vcs import src < src/ros2_rust/ros2_rust_${ROS_DISTRO}.repos

# Build the workspace
echo "Building the workspace..."
colcon build

# Source the workspace
echo "To use this workspace, run:"
echo "source $(pwd)/install/setup.bash"

echo "Installation complete! Please source the workspace to start using ros2tui."
