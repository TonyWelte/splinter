# Recording TUI Demos with VHS

This folder contains a ready-to-run [VHS](https://github.com/charmbracelet/vhs) tape for Splinter.

## Prerequisites

1. Install VHS from the upstream release page (see https://github.com/charmbracelet/vhs/pull/719).
2. Install runtime dependencies used by VHS:

```bash
sudo apt update
sudo apt install -y ffmpeg ttyd
```

3. Make sure ROS 2 is installed and your workspace is built so `splinter` can run.

## Run the demo tape

From the repository root:

```bash
VHS_NO_SANDBOX=false vhs doc/vhs/splinter-graph.tape
```

This writes a GIF to:

- `doc/img/splinter-vhs-demo.gif`

## Optional: export MP4

```bash
ffmpeg -y -i doc/img/splinter-vhs-demo.gif -movflags faststart -pix_fmt yuv420p doc/img/splinter-vhs-demo.mp4
```

## Customize

Open `doc/vhs/splinter-demo.tape` and adjust:

- `Set Width` / `Set Height`
- `Set FontSize`
- `Set Theme`
- `Type`, `Sleep`, and key presses (`Enter`, `Escape`, etc.)
