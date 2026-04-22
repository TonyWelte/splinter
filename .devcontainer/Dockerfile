FROM osrf/ros:jazzy-desktop-full

# Upgrade all packages
RUN sudo apt update && sudo apt upgrade -y

# Install essential packages
RUN sudo apt install -y wget ninja-build clangd-19 ssh nano neovim less curl libclang-dev python3-vcstool python3-pip zsh sudo golang ffmpeg ttyd tmux git-lfs

# GUI / Wayland support for rviz2 and other graphical tools
RUN sudo apt install -y \
    libwayland-client0 \
    libwayland-egl1 \
    qtwayland5 \
    && mkdir -p /tmp/runtime-root && chmod 0700 /tmp/runtime-root

RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Rust related ROS packages
RUN cargo install --debug cargo-ament-build
RUN pip3 install --break-system-packages git+https://github.com/colcon/colcon-cargo.git
RUN pip3 install --break-system-packages git+https://github.com/colcon/colcon-ros-cargo.git

# Create overlay workspace
WORKDIR /root/ros_ws/src

COPY packages.repos .

RUN vcs import < packages.repos; \
    vcs import < ros2_rust/ros2_rust_rolling.repos; \
    cd ..; \
    rosdep install --from-paths src --ignore-src -r -y;

# Install Nav2 + Gazebo TB3 packages for test scenario (tools/test_scenario.launch.py)
RUN sudo apt install -y \
    ros-${ROS_DISTRO}-navigation2 \
    ros-${ROS_DISTRO}-nav2-bringup \
    ros-${ROS_DISTRO}-nav2-simple-commander \
    ros-${ROS_DISTRO}-nav2-minimal-tb3-sim \
    ros-${ROS_DISTRO}-nav2-loopback-sim \
    ros-${ROS_DISTRO}-turtlebot3-gazebo

RUN cd /tmp && \
    git clone --depth 1 --branch caption-overlay https://github.com/pkazmier/vhs.git && \
    cd vhs && \
    go build && \
    mv vhs /usr/local/bin/ && \
    cd .. && rm -rf vhs


RUN echo "source /opt/ros/${ROS_DISTRO}/setup.zsh" >> ~/.zshrc
RUN echo "source /root/ros_ws/setup.zsh" >> ~/.zshrc