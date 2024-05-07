# Reality

Usage:

    cargo run -r -- -p <pif-rom-file> <rom-file>

For some IPL3 variants, the PIF ROM may be omitted:

    cargo run -r -- <rom-file>

## Status

| Component | Status |
| --------- | ------ |
| CPU       | Done, except for floating point rounding modes and some rarely used instructions. |
| RSP       | Done, except for some rarely used instructions. |
| RDP (Graphics) | Texture engine TMEM usage is not accurate and is likely leading to texture bugs that are visibile in some software. Z-buffer implementation is incomplete. Alpha blending implementation is incomplete. Some lesser-used features are missing. |
| Audio     | Glitchy. Audio engine needs a rewrite. |
