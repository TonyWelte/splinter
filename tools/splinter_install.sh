#!/bin/bash
set -e

# Function to print error messages and exit
error_exit() {
    echo "$1" 1>&2
    exit 1
}

# Check for non-interactive mode
NON_INTERACTIVE=false
if [[ "$1" == "--non-interactive" ]]; then
    NON_INTERACTIVE=true
fi

# Function to prompt for confirmation (only if not in non-interactive mode)
prompt_confirm() {
    if [[ "$NON_INTERACTIVE" == true ]]; then
        return 0
    fi
    read -p "$1 [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        return 1
    fi
    return 0
}

echo "========================================================================"
echo "STARTING INSTALLATION SCRIPT"
echo "========================================================================"

echo ""
echo "-----------------------------------------------------------------------"
echo "CHECKING USER PRIVILEGES"
echo "-----------------------------------------------------------------------"
# Warn if running as root (only if not in Docker/non-interactive)
if [[ "$NON_INTERACTIVE" != true && "$(id -u)" -eq 0 ]]; then
    echo "WARNING: This script is being run as root. Running as root is not recommended."
    if ! prompt_confirm "Do you want to continue?"; then
        error_exit "Installation aborted."
    fi
fi

echo ""
echo "-----------------------------------------------------------------------"
echo "CHECKING ROS_DISTRO"
echo "-----------------------------------------------------------------------"
# Check if ROS_DISTRO is set
if [ -z "$ROS_DISTRO" ]; then
    error_exit "ROS_DISTRO is not set. Please source your ROS 2 environment or build the Docker image with --build-arg ROS_DISTRO=<your_ros_distro>."
fi

echo ""
echo "-----------------------------------------------------------------------"
echo "INSTALLING RUST"
echo "-----------------------------------------------------------------------"
# Install Rust
if ! command -v rustc &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "Rust is already installed."
fi

echo ""
echo "-----------------------------------------------------------------------"
echo "INSTALLING SYSTEM PACKAGES"
echo "-----------------------------------------------------------------------"
# Install required system packages
echo "Installing required system packages..."
if [[ "$NON_INTERACTIVE" == true ]]; then
    apt-get update && apt-get install -y --no-install-recommends git libclang-dev python3-pip python3-vcstool
else
    sudo apt update && sudo apt install -y git libclang-dev python3-pip python3-vcstool
fi
cargo install cargo-ament-build

echo ""
echo "-----------------------------------------------------------------------"
echo "SETTING UP WORKSPACE"
echo "-----------------------------------------------------------------------"
# Setup workspace
echo "Setting up workspace and cloning dependencies..."
mkdir -p ./splinter_ws
cd ./splinter_ws
mkdir -p src

echo ""
echo "-----------------------------------------------------------------------"
echo "INSTALLING COLCON PLUGINS"
echo "-----------------------------------------------------------------------"
# Install colcon plugins for Rust
echo "Installing colcon plugins for Rust..."

# Check ROS_DISTRO to determine if --break-system-packages is needed
if [[ "$ROS_DISTRO" != "humble" ]]; then
    echo "ROS_DISTRO is $ROS_DISTRO. Using --break-system-packages for pip install."
    if [[ "$NON_INTERACTIVE" == true ]]; then
        pip install --break-system-packages git+https://github.com/colcon/colcon-cargo.git
        pip install --break-system-packages git+https://github.com/colcon/colcon-ros-cargo.git
    else
        if prompt_confirm "Do you want to continue with --break-system-packages?"; then
            pip install --break-system-packages git+https://github.com/colcon/colcon-cargo.git
            pip install --break-system-packages git+https://github.com/colcon/colcon-ros-cargo.git
        else
            error_exit "Installation aborted by user."
        fi
    fi
else
    pip install git+https://github.com/colcon/colcon-cargo.git
    pip install git+https://github.com/colcon/colcon-ros-cargo.git
fi

echo ""
echo "-----------------------------------------------------------------------"
echo "CLONING REPOSITORIES"
echo "-----------------------------------------------------------------------"
# Clone ros2_rust and splinter
git clone -b main https://github.com/Tonywelte/splinter.git src/splinter || true
vcs import src < src/splinter/tools/packages.repos
# Import dependencies using the detected ROS_DISTRO
vcs import src < src/ros2_rust/ros2_rust_${ROS_DISTRO}.repos

echo ""
echo "-----------------------------------------------------------------------"
echo "BUILDING WORKSPACE"
echo "-----------------------------------------------------------------------"
# Build the workspace
echo "Building the workspace..."
source /opt/ros/$ROS_DISTRO/setup.bash
colcon build --packages-up-to splinter

echo ""
echo "-----------------------------------------------------------------------"
echo "INSTALLATION COMPLETE"
echo "-----------------------------------------------------------------------"
# Source the workspace
echo "Installation complete! Please source the workspace to start using splinter."
echo "source $(pwd)/install/setup.bash"
echo "ros2 run splinter splinter"
