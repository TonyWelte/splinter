Output doc/img/splinter-service.gif
Set Shell "bash"
Set Framerate 30
Set Theme "GitHub Dark"
Set FontSize 18
Set Padding 0
Set CaptionInactivityTimer 2000ms

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

# Navigate to the node view and select the dev_node
Sleep 2s
Tab
Sleep 2s
Type "/"
Sleep 2s
Type "dev"
Sleep 1s
Enter
Sleep 1s
Down
Sleep 1s
Enter
Sleep 1s
Enter
Sleep 1s

# Navigate to the service to call
Down
Sleep 1s
Down
Sleep 1s
Down
Sleep 1s
Right
Sleep 1s
Enter
Sleep 1s
Enter
Sleep 1s

# Edit message
Down
Sleep 1s
Enter
Sleep 1s
Type "true"
Sleep 1s
Enter
Sleep 2s

Type "c"
Sleep 2s

Down
Down
Down
Sleep 1s
Enter
Sleep 1s
Type "3.1415"
Sleep 1s
Enter
Sleep 1s

Type "c"
Sleep 2s

Down
Down
Down
Sleep 1s
Enter
Sleep 1s
Type "42"
Sleep 1s
Enter
Sleep 1s

Type "c"
Sleep 2s

Type "G"
Sleep 1s
Enter
Sleep 1s
Type "Hello World"
Sleep 1s
Enter
Sleep 1s

Type "c"
Sleep 5s

Hide
Type "kill $DEV_NODE_PID"
Enter