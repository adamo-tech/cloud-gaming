# cloud-gaming

Stream games from your own Linux PC to any browser with Adamo.

The host runs your game in a headless gamescope session at the resolution you ask for and publishes gamescope's PipeWire screen stream as AV1 video and the game's audio as Opus through the Adamo SDK. Keyboard and mouse input reach gamescope through its libei socket, gamepads become virtual Xbox pads, and the cursor and gamepad rumble are sent back to the player. The PC does not sleep while a game is streaming.

## Requirements

- Linux with a GPU that encodes AV1: NVIDIA RTX 40 or newer, Intel Arc, or AMD RDNA3 or newer.
- gamescope and Steam.
- PipeWire with pipewire-pulse (`pactl`), and systemd.
- Write access to `/dev/uinput` for virtual gamepads:
  ```
  echo 'KERNEL=="uinput", MODE="0660", GROUP="input"' | sudo tee /etc/udev/rules.d/60-cloud-gaming.rules
  sudo usermod -aG input $USER
  ```
- Build dependencies (Ubuntu 24.04 names):
  ```
  sudo apt install build-essential pkg-config clang libclang-dev cmake libx264-dev libdav1d-dev libopus-dev libasound2-dev \
    libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libgstreamer-plugins-bad1.0-dev gstreamer1.0-pipewire libevdev-dev libva-dev libc++-dev
  ```

The host builds against a checkout of `adamo` next to this repository.

## Run

Close Steam on the desktop first: Steam runs one instance per user, and the host starts its own inside gamescope.

```
ADAMO_API_KEY=... cargo run --release -- --width 1920 --height 1080 --fps 60 -- steam -gamepadui
```

The robot is called `cloud-gaming` unless you pass `--name`.

## Input channels

| Channel | Message |
| --- | --- |
| `json/keyboard` | `Keyboard`, key codes as the browser's `KeyboardEvent.code` |
| `json/mouse` | `Mouse`, running totals of motion and wheel, buttons as `MouseEvent.buttons` |
| `joy/<index>` | `Joy` or `JoystickCommand` in the standard gamepad layout, one channel per pad |

Rumble for pad `<index>` arrives on `adamo/{org}/{robot}/feedback/gamepad/rumble`. The cursor arrives on `adamo/{org}/{robot}/cursor/image` (RGBA and hotspot, on change and once a second) and `adamo/{org}/{robot}/cursor/position` (on change), for the player to draw over the video.

## Player

`web/` is the browser player. It needs Chrome for AV1 decoding and keyboard lock.

```
cd web && npm install && npm run dev
```
