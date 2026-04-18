#!/usr/bin/env python3
"""
Combined test scenario launch file for Splinter.

Launches a full Gazebo simulation with a TurtleBot3 Waffle navigating
autonomously via Nav2, plus supplementary nodes for topic variety — ideal
for exercising every part of the Splinter TUI.

Usage:
    ros2 launch splinter/tools/test_scenario.launch.py
    ros2 launch splinter/tools/test_scenario.launch.py headless:=False  # show Gazebo GUI
    ros2 launch splinter/tools/test_scenario.launch.py use_rviz:=True   # also open RVIZ
"""

import os

from ament_index_python.packages import get_package_share_directory

from launch import LaunchDescription
from launch.actions import (
    DeclareLaunchArgument,
    ExecuteProcess,
    IncludeLaunchDescription,
    TimerAction,
)
from launch.conditions import IfCondition
from launch.launch_description_sources import PythonLaunchDescriptionSource
from launch.substitutions import LaunchConfiguration
from launch_ros.actions import Node


def generate_launch_description():
    # Directories
    nav2_bringup_dir = get_package_share_directory('nav2_bringup')
    dummy_robot_dir = get_package_share_directory('dummy_robot_bringup')

    this_dir = os.path.dirname(os.path.abspath(__file__))

    # Launch arguments
    use_turtlesim = LaunchConfiguration('use_turtlesim')
    headless = LaunchConfiguration('headless')
    use_rviz = LaunchConfiguration('use_rviz')

    declare_use_turtlesim = DeclareLaunchArgument(
        'use_turtlesim',
        default_value='False',
        description='Also launch turtlesim + draw_square for extra topic variety',
    )
    declare_headless = DeclareLaunchArgument(
        'headless',
        default_value='True',
        description='Run Gazebo in headless mode (server only, no GUI)',
    )
    declare_use_rviz = DeclareLaunchArgument(
        'use_rviz',
        default_value='False',
        description='Launch RVIZ2 for visualization',
    )

    # --- 1. Full Gazebo TB3 simulation with Nav2 ---
    # This single launch file starts:
    #   - Gazebo server (+ GUI if headless:=False)
    #   - TB3 Waffle robot with lidar, odometry, IMU, camera
    #   - robot_state_publisher (/tf, /tf_static, /robot_description)
    #   - Full Nav2 stack (AMCL, planner, controller, behavior, costmaps, etc.)
    #   - RVIZ (if use_rviz:=True)
    nav2_gazebo = IncludeLaunchDescription(
        PythonLaunchDescriptionSource(
            os.path.join(nav2_bringup_dir, 'launch',
                         'tb3_simulation_launch.py')
        ),
        launch_arguments={
            'headless': headless,
            'use_rviz': use_rviz,
            'use_composition': 'False',
            'autostart': 'True',
        }.items(),
    )

    # --- 2. Dummy robot bringup ---
    # Adds extra topics: /joint_states, /scan (dummy), /map (dummy),
    # /tf, /robot_description from an RRBot arm model.
    # dummy_robot = IncludeLaunchDescription(
    #     PythonLaunchDescriptionSource(
    #         os.path.join(dummy_robot_dir, 'launch',
    #                      'dummy_robot_bringup_launch.py')
    #     ),
    # )

    # --- 3. Autonomous velocity driver ---
    # Delayed start to let Gazebo + Nav2 initialize.
    # Drives the robot in square + figure-eight patterns via cmd_vel.
    waypoint_follower = TimerAction(
        period=15.0,
        actions=[
            ExecuteProcess(
                cmd=['python3', os.path.join(this_dir, 'autonomous_nav.py')],
                output='screen',
            ),
        ],
    )

    # --- 4. Turtlesim + draw_square (optional extra topics) ---
    turtlesim_node = ExecuteProcess(
        cmd=['ros2', 'run', 'turtlesim', 'turtlesim_node'],
        output='screen',
        condition=IfCondition(use_turtlesim),
    )
    draw_square = TimerAction(
        period=2.0,
        actions=[
            ExecuteProcess(
                cmd=['ros2', 'run', 'turtlesim', 'draw_square'],
                output='screen',
                condition=IfCondition(use_turtlesim),
            ),
        ],
    )

    # --- 5. Demo talker (std_msgs/String on /chatter) ---
    talker = ExecuteProcess(
        cmd=['ros2', 'run', 'demo_nodes_cpp', 'talker'],
        output='screen',
    )

    return LaunchDescription([
        declare_use_turtlesim,
        declare_headless,
        declare_use_rviz,
        nav2_gazebo,
        # dummy_robot,
        waypoint_follower,
        turtlesim_node,
        draw_square,
        talker,
    ])
