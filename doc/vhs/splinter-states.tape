Output doc/img/splinter-states.gif
Set Shell "bash"
Set Framerate 30
Set Theme "GitHub Dark"
Set FontSize 18
Set Padding 0

Require cargo
Require ros2

Hide
Type "source /opt/ros/$ROS_DISTRO/setup.bash"
Enter
Type "cd /root/ros_ws/src/splinter"
Enter
Type "./target/release/dev_node >/tmp/splinter-vhs-talker.log 2>&1 &"
Enter
Type "export DEV_NODE_PID=$!"
Enter
Type "clear"
Enter

Show
CaptionOn
Type "splinter"
Sleep 2s
Enter
Sleep 2s

# View the state message
Type "/state"
Enter
Enter
Enter

Sleep 1s

# Select first state
Down
Sleep 1s
Enter
Sleep 1s
Down
Sleep 1s
Enter

Sleep 3s

# Select second state
Shift+Tab # Move back to message view
Sleep 1s
Down
Down
Down
Down
Down
Sleep 1s
Enter
Sleep 1s
Down
Down
Sleep 1s
Enter

Sleep 1s
Tab

Sleep 10s

Hide
Type "kill $DEV_NODE_PID"
Enter
