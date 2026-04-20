Output doc/img/splinter-publishing.gif
Set Shell "bash"
Set Framerate 30
Set Theme "GitHub Dark"
Set FontSize 18
Set Padding 0

Require cargo
Require tmux

Hide
Type "cd /root/ros_ws/src/splinter"
Enter

# Create tmux session with two panes
Type "tmux kill-session -t splinter_demo"
Enter
Type "tmux new-session -d -s splinter_demo"
Enter

# Left pane: run dev_node
Type "tmux send-keys -t splinter_demo:0 'source ../../install/setup.bash' Enter"
Enter
Type "tmux send-keys -t splinter_demo:0 'splinter topic pub /odom nav_msgs/msg/Odometry' Enter"
Enter

# Right pane: split and prepare listener
Type "tmux split-window -h -t splinter_demo:0"
Enter
Type "tmux send-keys -t splinter_demo:0.1 'cd /root/ros_ws/src/splinter && source ../../install/setup.bash' Enter"
Enter

# Start splinter in the right pane
Type "tmux send-keys -t splinter_demo:0.1 'splinter topic echo /odom' Enter"
Enter
Type "tmux select-pane -t splinter_demo:0.0"
Enter

# Show the split view
Type "tmux a"
Enter

Show
CaptionOn

Sleep 2s

# Edit the message field
Down
Down
Down
Down
Down
Sleep 1
Enter
Type "Move up and down with arrows or j/k"
Sleep 3
Enter

Down
Enter
Type "Press Enter to edit value"
Sleep 3
Enter
Sleep 1
Enter
Type "Enter again to submit edit"
Sleep 3
Enter

Down
Down
Down
Down
Enter
Type "Press 'p' to publish one message"
Sleep 3
Enter
Sleep 1
Enter
Sleep 1
Enter
Type "3.1415"
Sleep 1
Enter

# Confirm and publish
Sleep 1s
Type "p"
Sleep 2s
Type "p"
Sleep 2s
Type "p"
Sleep 2s
Type "p"
Sleep 2s

Hide
Type "tmux kill-session -t splinter_demo"
Enter
