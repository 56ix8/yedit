# YEDIT app icon

A **placeholder**: a white "Y" on a full-bleed orange-to-violet gradient tile, with four crop
marks in the corners. Replace it with the final logo by editing `yedit.svg` (and `yedit-small.svg`)
and running `packaging/icons.sh`.

## Palette

| Colour | Hex | Used for |
|---|---|---|
| Orange | `#ff8a4c` | gradient start (top left) |
| Violet | `#7b2ff7` | gradient end (bottom right) |
| White | `#ffffff` | the Y; the crop marks at 50% opacity |

## Geometry

A 512-unit tile (`viewBox="0 0 512 512"`), rounded square with `rx=112`, no border. Everything
that matters sits inside 22..490: the macOS renders pad the tile onto Apple's 824/1024 icon grid,
and the Windows and Linux renders crop 22 units off each side.

## Files

- `yedit.svg`: canonical master.
- `yedit-small.svg`: without the crop marks and with a heavier Y; the hicolor scalable icon.
- `yedit-1024.png`: 1024 px render on the macOS grid.
- `yedit.icns`: macOS bundle icon (`CFBundleIconFile`).
- `yedit.ico`: Windows icon, 16–256 px, embedded in the `.exe` by `apps/yedit/build.rs`.
- `hicolor/<size>/apps/com.fesilabs.yedit.png` (16–512) and `hicolor/scalable/...svg`: Linux icon
  theme.

All renders come from `packaging/icons.sh` (resvg, plus `cargo xtask ico`). License: `LICENSE.txt`.
