# cloud-gaming

Stream games from your own Linux PC to any browser with Adamo.

The host runs your game in a headless gamescope session at the player's screen resolution and publishes gamescope's PipeWire screen stream as AV1 video and the game's audio as Opus through the Adamo SDK. Keyboard and mouse input reach gamescope through its libei socket, gamepads become virtual Xbox pads, and the cursor and gamepad rumble are sent back to the player. The PC does not sleep while a game is streaming.

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
cargo run --release -- --fps 60 -- steam -gamepadui
```

The host waits for a player to connect and starts gamescope at that player's screen size, scaled down to fit within `--max-width` by `--max-height` (2560 by 1440 by default). When a player connects with a different screen size, the host restarts gamescope at the new size, which closes the running game. Reconnecting from the same screen keeps the game running. The host assumes one player at a time.

The host runs fully offline: no account, API key or relay. It prints a pairing key and saves it to `pairing-key` (`--pairing-key-file`) so the next start reuses it. In the player, enter this PC's address and the pairing key. The player connects directly over WebTransport on port 17447, so allow that port through the PC's firewall, for example `sudo ufw allow from 192.168.1.0/24 to any port 17447`. From outside your home network, reach the PC through Tailscale or a forwarded port.

The robot is called `cloud-gaming` unless you pass `--name`.

Mouse motion is spread over `--mouse-smoothing-ms` (8 by default). A 125 Hz mouse delivers two or three reports per 60 fps frame, and streaming jitter moves reports between frames, so mouse-look stutters without it. With a 500 Hz or faster gaming mouse, pass `--mouse-smoothing-ms 0` to inject each update as it arrives, like Moonlight and Selkies, with no added latency.

## Input channels

| Channel | Message |
| --- | --- |
| `json/display` | `{"width", "height"}`, the player's screen size in pixels, sent once on connect |
| `json/keyboard` | `Keyboard`, key codes as the browser's `KeyboardEvent.code` |
| `json/mouse` | `Mouse`, running totals of motion and wheel, buttons as `MouseEvent.buttons` |
| `joy/<index>` | `Joy` or `JoystickCommand` in the standard gamepad layout, one channel per pad |

Rumble for pad `<index>` arrives on `adamo/{org}/{robot}/feedback/gamepad/rumble`. The cursor arrives on `adamo/{org}/{robot}/cursor/image` (RGBA and hotspot, on change and once a second) and `adamo/{org}/{robot}/cursor/position` (on change), for the player to draw over the video.

## Player

`web/` is the browser player. It needs Chrome for AV1 decoding and keyboard lock.

```
cd web && npm install && npm run dev
```

## Adamo Play

`mobile/` is the phone player for iOS and Android. Pair a controller with the phone, or play with the on-screen controller that shows while no controller is connected. `mobile/core` is a Rust crate on the Adamo SDK that connects to the PC, receives AV1 or H.264 video and audio, sends the gamepad and receives rumble; UniFFI generates its Swift and Kotlin bindings. Each app decodes video in hardware (VideoToolbox, MediaCodec) and reads controllers natively (GameController, Android input events).

AV1 needs an iPhone with an A17 Pro or newer; for older iPhones, run the host with `--codec h264`. Android phones without an AV1 decoder fall back to Android's software decoder.

Android, with the Android SDK, NDK 27.2.12479018 and `cargo install cargo-ndk`:

```
cd mobile/android && ./gradlew installDebug
```

iOS, with Xcode and `brew install xcodegen`:

```
mobile/build-core.sh iphonesimulator && cd mobile/ios && xcodegen && open AdamoPlay.xcodeproj
```
