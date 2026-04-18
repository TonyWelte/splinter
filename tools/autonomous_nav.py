#!/usr/bin/env python3
"""
Autonomous velocity driver for Splinter testing.

Publishes cmd_vel commands that drive the simulated robot in repeating
patterns (straight, turn, straight, turn…).  Works with both the Gazebo
simulation and the Nav2 loopback simulation — both react to cmd_vel and
produce odom / scan / tf data.

No Nav2 navigation stack interaction needed — this just moves the robot.
"""

import math
import time

from geometry_msgs.msg import Twist
import rclpy
from rclpy.node import Node


class PatternDriver(Node):
    def __init__(self):
        super().__init__('pattern_driver')
        self.pub = self.create_publisher(Twist, 'cmd_vel', 10)
        self.get_logger().info('Pattern driver started — publishing cmd_vel')

    def drive(self, linear: float, angular: float, duration: float):
        """Publish a constant twist for *duration* seconds."""
        msg = Twist()
        msg.linear.x = linear
        msg.angular.z = angular
        end = time.monotonic() + duration
        while time.monotonic() < end and rclpy.ok():
            self.pub.publish(msg)
            time.sleep(0.05)  # 20 Hz

    def stop(self, duration: float = 0.3):
        self.drive(0.0, 0.0, duration)


def main():
    rclpy.init()
    driver = PatternDriver()

    time.sleep(2.0)  # Let the sim settle

    try:
        cycle = 0
        while rclpy.ok():
            cycle += 1
            driver.get_logger().info(f'--- Pattern cycle {cycle} ---')

            # Square-ish loop
            for side in range(4):
                driver.get_logger().info(f'  Straight segment {side + 1}/4')
                driver.drive(linear=0.2, angular=0.0, duration=3.0)
                driver.stop()
                driver.get_logger().info(f'  Turn {side + 1}/4')
                driver.drive(linear=0.0, angular=0.5, duration=math.pi / 0.5)
                driver.stop()

            # Figure-eight arc
            driver.get_logger().info('  Figure-eight arc')
            driver.drive(linear=0.15, angular=0.3, duration=6.0)
            driver.drive(linear=0.15, angular=-0.3, duration=6.0)
            driver.stop(0.5)

    except KeyboardInterrupt:
        pass
    finally:
        driver.stop(0.5)
        driver.destroy_node()
        rclpy.shutdown()


if __name__ == '__main__':
    main()
