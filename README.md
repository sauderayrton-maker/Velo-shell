# Velo Shell

A `waybar`-style status bar for Hyprland, styled to match **Velo Browser**,
**Velo Player**, **Velo Files** and **Velo Launcher** — the same dark glass
panels, `#8ab4d4` accent, rounded corners and accent-tinted scrollbars as the
rest of the suite.

Velo Shell docks a thin glass strip to the **top edge** of the screen via
`gtk4-layer-shell` and reserves its own space (an exclusive zone), so the
rest of your windows tile below it.

## Modules

| Module          | Shows                                                   | Interaction |
|------------------|--------------------------------------------------------|-------------|
| **Workspaces**   | Pills for workspaces 1–10 (occupied/active highlighted), plus the focused window's title | Click a pill to switch workspace |
| **Clock**        | Time and date, centered                                 | Click for a calendar popover |
| **CPU / RAM**    | Live usage from `/proc/stat` and `/proc/meminfo`        | — |
| **Network**      | Wi-Fi SSID + signal icon, wired, or offline             | — |
| **Volume**       | Output icon + percentage                                | Scroll to adjust, left-click to mute, right-click for a slider |
| **Battery**      | Charge percentage with charging state (hidden on desktops with no battery) | — |
| **Power**        | Power icon                                              | Click for lock / suspend / log out / restart / shut down |

All live data updates via Hyprland's event socket (workspaces, window
titles) or short polling intervals (clock, stats, network, volume, battery).

## Requirements

- GTK4 (4.12+)
- [gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell)
- A recent [Rust toolchain](https://rustup.rs) (stable, 2021 edition)
- At runtime: `hyprctl` (Hyprland), `pactl` (PipeWire/PulseAudio), `nmcli`
  (NetworkManager) — all standard on a typical Hyprland desktop

| Component       | Arch package      | Debian/Ubuntu package            | Fedora package         |
|------------------|-------------------|------------------------------------|--------------------------|
| Build tools      | `base-devel`      | `build-essential`, `pkg-config`   | `gcc`, `pkg-config`     |
| GTK4             | `gtk4`            | `libgtk-4-dev`                     | `gtk4-devel`            |
| gtk4-layer-shell | `gtk4-layer-shell`| `libgtk4-layer-shell-dev`          | `gtk4-layer-shell-devel`|

## Installation

### Quick install (recommended)

```bash
git clone https://github.com/sauderayrton-maker/Velo-shell.git
cd Velo-shell
./install.sh
```

This detects your package manager, installs the system dependencies above,
builds `velo-shell` in release mode, and installs it via `make install`
(requires `sudo` for the final install step).

### Manual build

```bash
cargo build --release                  # the bar
sudo make install PREFIX=/usr/local    # install
```

To remove everything Velo Shell installed:

```bash
make uninstall            # or: ./uninstall.sh
```

A copy is also installed as `velo-shell-uninstall`, so it works even if
you've deleted this cloned repo.

### Update

```bash
./update.sh
```

Pulls the latest commit, rebuilds, and reinstalls the binary in place.

### Run without installing

```bash
cargo run --release
```

## Hyprland setup

Start Velo Shell with Hyprland by adding it to `hyprland.conf`:

```ini
exec-once = velo-shell
```

If you're replacing an existing bar (waybar, Quickshell, etc.), remove its
`exec-once` line — running two exclusive-zone bars at once will both reserve
space and stack below each other.

Velo Shell sets its layer-shell namespace to `velo-shell`, so you can target
it with a `layerrule` for blur to match the rest of the suite:

```ini
layerrule {
    name = "velo_shell"
    match:namespace = ^(velo-shell)$
    blur = on
}
```

No `windowrule` is needed — the bar is a layer-shell surface and positions
itself along the top edge, reserving its height via an exclusive zone so
other windows tile below it.
