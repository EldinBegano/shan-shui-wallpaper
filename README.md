# shan-shui-wallpaper

An endless Chinese landscape painting that slowly scrolls across your desktop
background. It is a Rust port of LingDong-'s
[shan-shui-inf](https://github.com/LingDong-/shan-shui-inf) generator, shown
as a wlr-layer-shell background surface (Hyprland, Sway, river, KDE, ...).

The generator is ported statement by statement and produces exactly the same
shapes as the web version: a seed paints the same landscape in both
(`--dump` output is checked against the JavaScript for several seeds). The one
difference: the original's modern easter eggs, the "Pizza Hut" sign and the
power-line pylons, are left out, so `--dump` lacks those shapes.

## Installing

Arch Linux, from the [AUR](https://aur.archlinux.org/packages/shan-shui-wallpaper):

```sh
yay -S shan-shui-wallpaper
```

## Running

```sh
cargo build --release
./target/release/shan-shui-wallpaper            # new landscape every start
./target/release/shan-shui-wallpaper --seed 42  # same landscape as ?seed=42 on the web
```

Options:

| option | default | |
|---|---|---|
| `--seed <text>` | current time | landscape seed |
| `--speed <px/s>` | 12 | scroll speed in screen pixels per second (0 = still) |
| `--fps <n>` | 30 | frame rate cap |

To start it with Hyprland, add to `hyprland.conf`:

```
exec-once = /path/to/shan-shui-wallpaper
```

It sits on the background layer, above other wallpaper daemons (swww/awww,
hyprpaper) if those are still running.

## Debugging

```sh
shan-shui-wallpaper --png out.png --seed 42 --x 1000   # one frame, at world x
shan-shui-wallpaper --bench 42                         # generation/paint timings
shan-shui-wallpaper --dump 42 > rust.dump              # web-compatible SVG dump
SHAN_SHUI_DEBUG=1 shan-shui-wallpaper                  # log every commit
```

## Limitations

- One surface, on the output the compositor picks; multi-monitor setups
  aren't handled yet. When that output goes away (unplugged, or disabled when
  the lid closes), the landscape moves on to the output the compositor picks
  next.

## Releasing

Bump `version` in `Cargo.toml`, commit, then tag and push:

```sh
git tag v0.2.0 && git push origin v0.2.0
```

`.github/workflows/release.yml` checks the tag against `Cargo.toml`, runs the
tests, creates the GitHub release and publishes the AUR package through
`packaging/aur/publish.sh`. It needs the AUR SSH private key in the
`AUR_SSH_KEY` repository secret. `packaging/aur/publish.sh shan-shui-wallpaper
<version>` without `--push` builds and checks the package locally.
