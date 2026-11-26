<div align="center">
  <img src=".github/banner.svg" alt="Adamo">
  <h1>Adamo Cloud Gaming</h1>
  <p>Stream games from your Linux PC to a browser or phone, peer to peer.</p>

  ![Host](https://img.shields.io/badge/host-Linux-FCC624?logo=linux&logoColor=black)
  ![Players](https://img.shields.io/badge/players-Chrome%20%7C%20Firefox%20%7C%20Safari%20%7C%20iOS%20%7C%20Android-informational)
  ![Video](https://img.shields.io/badge/video-AV1%20%7C%20H.264-blueviolet)
  ![Rust](https://img.shields.io/badge/rust-2024-orange?logo=rust)
  ![License](https://img.shields.io/github/license/adamo-tech/cloud-gaming)
</div>

## Set up the gaming PC

You need:

- Linux with systemd, PipeWire and pipewire-pulse.
- A GPU that encodes AV1: NVIDIA RTX 40 or newer, Intel Arc, or AMD RDNA3 or newer. Older GPUs can stream H.264 instead (see [Tips](#tips)).
- [gamescope](https://github.com/ValveSoftware/gamescope) and Steam.

Install the build dependencies (Ubuntu 24.04 and Debian 13 package names) and Rust:

```
sudo apt install build-essential pkg-config clang libclang-dev cmake libx264-dev libdav1d-dev libopus-dev libasound2-dev \
  libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libgstreamer-plugins-bad1.0-dev gstreamer1.0-pipewire \
  libevdev-dev libva-dev libc++-dev pulseaudio-utils
curl https://sh.rustup.rs -sSf | sh
```

Let the host create virtual gamepads, then log out and back in so the group change applies:

```
echo 'KERNEL=="uinput", MODE="0660", GROUP="input"' | sudo tee /etc/udev/rules.d/60-cloud-gaming.rules
sudo udevadm control --reload && sudo udevadm trigger --name-match=uinput
sudo usermod -aG input $USER
```

Players connect directly to the PC on port 17447. Allow it from your local network, for example with ufw:

```
sudo ufw allow from 192.168.1.0/24 to any port 17447
```

## Start the host

Close Steam on the desktop first: Steam runs one instance per user, and the host starts its own inside gamescope.

```
git clone https://github.com/adamo-tech/cloud-gaming.git
cd cloud-gaming
cargo run --release -- -- steam -gamepadui
```

Everything after the second `--` is the game command, so you can launch any game or launcher, not only Steam.
When the command is `steam`, gamescope runs in Steam mode (`-e`) and Steam picks the window to stream. Any other command streams its own window.

The host prints a line like:

```
Connect from the player to this PC's address, port 17447, with pairing key <key>
```

The pairing key is what lets a player in. The host saves it to `pairing-key` (readable only by you) and reuses it on every start, so you pair each player once. To revoke every player, stop the host, delete `pairing-key` and start it again: it prints a new key.

The host waits for a player before it starts gamescope, at that player's screen size. It keeps the PC awake while a game is streaming, and makes Cloud-Gaming the PC's default sound output so every game's sound goes to the player. After you stop the host, pick your speakers again in the desktop's sound settings.

| Option | Default | What it does |
| --- | --- | --- |
| `--fps` | `60` | Frame rate of gamescope and the stream |
| `--bitrate-kbps` | `20000` | Video bitrate |
| `--codec` | `av1` | `av1` or `h264` |
| `--encoder` | automatic | Force a specific encoder |
| `--max-width`, `--max-height` | `2560`, `1440` | Largest resolution; bigger screens are scaled down to fit |
| `--mouse-smoothing-ms` | `8` | Spreads mouse motion over this many milliseconds; `0` turns it off |
| `--pairing-key-file` | `pairing-key` | Where the pairing key is saved |
| `--name` | `cloud-gaming` | The host's name, shown in the browser player |
| `--experimental` | `true` | The Adamo SDK's experimental mode, turn off if you observe latency issues |

## Play in a browser

The browser player runs in Chrome, Firefox and Safari. Safari decodes AV1 only up to 1080p, so for Safari start the host with `--max-width 1920 --max-height 1080`, or with `--codec h264`. Only Chrome locks the keyboard, so in Firefox and Safari some shortcuts still go to the browser.

Open [play.adamohq.com](https://play.adamohq.com), enter the gaming PC's address (for example `192.168.1.20`) and the pairing key, and press Play. The player goes fullscreen and captures the mouse and keyboard. Press Esc to release them, and click the video to capture them again. Up to four gamepads connected to the computer are forwarded.

To run the player yourself, open http://localhost:5179 after:

```
cd web
npm install
npm run dev -- --port 5179
```

Adamo's relays accept browser players only from play.adamohq.com and http://localhost:5179, so use that address for Adamo relay mode.

## Play on Android

Adamo Play runs on 64-bit Android 12 or newer. To build it you need the Android SDK with NDK 27.2.12479018, JDK 17, and:

```
rustup target add aarch64-linux-android
cargo install cargo-ndk
```

Connect the phone with USB debugging on, then:

```
cd mobile/android
./gradlew installDebug
```

## Play on iPhone

Adamo Play runs on iOS 17 or newer. AV1 needs an iPhone with an A17 Pro or newer; for older iPhones, start the host with `--codec h264`. To build it you need Xcode and:

```
brew install xcodegen
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
mobile/build-core.sh iphonesimulator
cd mobile/ios
xcodegen
open AdamoPlay.xcodeproj
```

To run on your iPhone, pick it as the run destination and set your team under Signing & Capabilities.

## Use the phone apps

Enter the gaming PC's address and the pairing key, and tap Play. The app remembers both.

- Pair a Bluetooth controller in the phone's settings, or play with the on-screen controller, which shows while no controller is connected.
- Hold anywhere on the screen for the menu: turn the on-screen controller on or off, turn phone vibration on or off, switch the on-screen buttons to PlayStation labels, show the frame rate and latency, or disconnect.
- Controller rumble plays on the controller, or on the phone when you play with the on-screen controller.
- On a foldable held half open, the game sits above the hinge and the controller below it.

## PS5 Remote Play

A PS5 on the same network as the gaming PC can be played from any player: [chiaki-ng](https://github.com/streetpea/chiaki-ng) runs as the game and streams the PS5 over Remote Play, and the host streams chiaki-ng to the player.

1. On the PS5, turn on Settings, System, Remote Play, Enable Remote Play. To wake the PS5 from rest mode, also turn on Settings, System, Power Saving, Features Available in Rest Mode, Stay Connected to the Internet and Enable Turning On PS5 from Network.
2. Install chiaki-ng and xclip on the gaming PC:

   ```
   flatpak install flathub io.github.streetpea.Chiaki4deck
   sudo apt install xclip
   ```

3. Start the host with chiaki-ng as the game. `--nosocket=wayland` keeps chiaki-ng inside gamescope instead of the desktop:

   ```
   cargo run --release -- -- flatpak run --socket=x11 --nosocket=wayland --env=SDL_AUDIODRIVER=pulseaudio io.github.streetpea.Chiaki4deck
   ```

4. Connect from a player. In chiaki-ng's settings, sign in to PSN so chiaki-ng gets your account ID. The player doesn't share its clipboard with the PC, so move the two links with xclip in an SSH session on the PC. First point xclip at gamescope's display:

   ```
   export DISPLAY=:$(ls -t /tmp/.X11-unix | head -1 | tr -d X)
   ```

   Click "Click to Re-Copy URL" in chiaki-ng, then print the login link and open it in a browser on your computer:

   ```
   xclip -o -selection clipboard
   ```

   After you sign in, the browser lands on an address starting with `https://remoteplay.dl.playstation.net/remoteplay/redirect?code=`. The page may look blank. Copy the whole address, put it on the PC's clipboard, and click "Click to Paste URL" in chiaki-ng:

   ```
   echo '<redirect address>' | setsid xclip -i -selection clipboard
   ```

5. chiaki-ng finds the PS5 on the network. Pick it, and enter the code the PS5 shows under Settings, System, Remote Play, Link Device. You register once; afterwards, pick the PS5 to play.

## Play away from home

To play outside your home network, forward port 17447, TCP and UDP, on your router to the gaming PC, and connect the player to your home's public address. If the PC sits behind two routers, forward the port on both. Allow the port from anywhere on the PC too, for example with `sudo ufw allow 17447`; the pairing key still decides who gets in.

## Use Adamo's relays

You can also connect through Adamo's relays. Sign up at [operate.adamohq.com](https://operate.adamohq.com), go to Settings, API Keys, and create a key (it starts with `ak_`). Start the host with it:

```
ADAMO_API_KEY=<API key> cargo run --release -- -- steam -gamepadui
```

In the player, pick Adamo relay and enter the host's name (`--name`, `cloud-gaming` by default) and the API key. The player still connects directly when it can reach the PC, and goes through the relay otherwise.

## Tips

- **Older GPUs and iPhones:** `--codec h264` works on far more hardware than AV1.
- **Stutter on iPhone:** turn off AirDrop (Control Center, AirDrop, Receiving Off) and Handoff (Settings, General, AirPlay & Continuity). While they are on, the iPhone's Wi-Fi leaves your network for about 100 ms roughly once a second, which freezes the video.
- **High or jumpy latency:** try `--experimental false`, which turns off the SDK's experimental BBRv3 congestion control.
- **Stuttering mouse-look:** a 125 Hz mouse sends two or three reports per frame, so the host smooths motion over 8 ms by default. With a 500 Hz or faster gaming mouse, `--mouse-smoothing-ms 0` sends every report as it arrives, with no added latency.
- **Changing screens:** reconnecting from the same screen keeps the game running. A player with a different screen size restarts gamescope at the new size, which closes the game.
- **One player at a time:** the host is built for a single player.

## License

[Apache-2.0](LICENSE)
