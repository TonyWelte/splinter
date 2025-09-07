#!/bin/bash
set -e

# Function to print error messages and exit
error_exit() {
    echo "$1" 1>&2
    exit 1
}

# Warn if running as root
if [ "$(id -u)" -eq 0 ]; then
    echo "WARNING: This script is being run as root. Running as root is not recommended."
    read -p "Do you want to continue? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        error_exit "Installation aborted."
    fi
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
cargo install cargo-ament-build

# Setup workspace
echo "Setting up workspace and cloning dependencies..."
mkdir -p ./ros2tui_ws
cd ./ros2tui_ws
mkdir -p src

# Install colcon plugins for Rust
echo "Installing colcon plugins for Rust..."
pip install --break-system-packages git+https://github.com/colcon/colcon-cargo.git
pip install --break-system-packages git+https://github.com/colcon/colcon-ros-cargo.git

# Clone ros2_rust and ros2tui
git clone -b develop https://github.com/Tonywelte/ros2tui.git src/ros2tui || true
vcs import src < src/ros2tui/tools/packages.repos

# Import dependencies using the detected ROS_DISTRO
vcs import src < src/ros2_rust/ros2_rust_${ROS_DISTRO}.repos

# Build the workspace
echo "Building the workspace..."
source /opt/ros/$ROS_DISTRO/setup.bash
colcon build --packages-up-to ros2tui

# Source the workspace
echo "To use this workspace, run:"
echo "source $(pwd)/install/setup.bash"

echo "Installation complete! Please source the workspace and virtual environment to start using ros2tui."
