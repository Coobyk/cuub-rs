# cuub-rs

A native Rust + [GPUI](https://www.gpui.rs/) port of [jvqtil/cuub](https://github.com/jvqtil/cuub), a minimalist 3×3 speedcubing timer.

## Run

From the repository, with current stable Rust installed:

```sh
cargo run --locked
```

On macOS, install the Xcode Command Line Tools first (`xcode-select --install`). On Debian/Ubuntu, install the build libraries and a Vulkan driver (Mesa covers Intel, AMD, and software rendering):

```sh
sudo apt-get install build-essential pkg-config libfontconfig1-dev libwayland-dev libxcb1-dev libxkbcommon-dev libxkbcommon-x11-dev libvulkan1 mesa-vulkan-drivers
```

Use a graphical session with Vulkan on Linux. The app was verified on Wayland (Sway with Mesa llvmpipe). If the window is blank on a virtual X11 display, try a Wayland session. macOS and Windows have not been verified.

## Controls

| Input | Action |
| --- | --- |
| Space or left click | Start, stop, then reset and generate a new scramble |
| `S` | Show or hide the scramble and matching cube net |
| `+` / `-` / `0` | Increase, decrease, or reset timer text size |
| `F` | Toggle fullscreen |
| Escape | Close the window |

The scramble hides on start and reappears when you stop. Each scramble is 25 random face turns with no consecutive turns of the same face, like the original—not an official WCA random-state scramble.

Run the timer and cube model tests with `cargo test --locked`.

Keep the lockfile when building: GPUI 0.2.2 transitively uses `xattr` 0.2, which does not compile against `libc` 0.2.190 or later on Linux.

The bundled [Oxanium](https://github.com/sevmeyer/oxanium) typeface is distributed under the [SIL Open Font License](assets/Oxanium-OFL.txt).
