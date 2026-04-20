Output doc/img/splinter-graph.gif
Set Shell "bash"
Set Framerate 30
Set Theme "GitHub Dark"
Set FontSize 18
Set Padding 0
Set CaptionInactivityTimer 2000ms

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
Type "/"
Sleep 2s
Type "sinu"
Sleep 1s
Enter
Sleep 1s
Enter
Sleep 1s
Enter
Sleep 1s
Down 20
Sleep 1s
Enter
Sleep 1s
Enter
Sleep 1s
Shift+Tab
Sleep 1s
Down
Sleep 1s
Enter
Sleep 1s
Down
Sleep 1s
Enter
Sleep 1s
Tab
Sleep 10s

Hide
Type "kill $DEV_NODE_PID"
Enter