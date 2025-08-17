FROM osrf/ros:jazzy-desktop-full

# Upgrade all packages
RUN sudo apt update && sudo apt upgrade -y

# Install essential packages
RUN sudo apt install -y wget ninja-build clangd-19 ssh nano neovim less curl libclang-dev python3-vcstool python3-pip zsh sudo
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

RUN echo "source /opt/ros/${ROS_DISTRO}/setup.zsh" >> ~/.zshrc
RUN echo "source /root/ros_ws/setup.zsh" >> ~/.zshrc