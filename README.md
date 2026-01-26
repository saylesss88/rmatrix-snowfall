# r-matrix-snowfall

[![Nix Flake](https://img.shields.io/badge/Nix_Flake-Geared-dddd00?logo=nixos&logoColor=white)](https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-flake.html)

[![Nix](https://img.shields.io/badge/Nix-5277C3?style=flat&logo=nixos&logoColor=white)](https://nixos.org)

This is a fork of [r-matrix](https://crates.io/crates/r-matrix)

Generates a 'Matrix'-like screen of falling snowflake characters and lambdas in
your terminal [![rmatrix](demo.gif)](https://asciinema.org/a/rjfdPpajCzwPYD98)

The original [`cmatrix`](https://github.com/abishekvashok/cmatrix) was written
in C, and crashes when you wildly resize the window. The rust version is
memory-safe, and doesn't crash so easily. Both versions have comparable
performance, due to the underlying use of `ncurses`.

## Controls

| Key         | Control                                             |
| ----------- | --------------------------------------------------- |
| 1-9         | Speed the letters fall (1 is fastest, 9 is slowest) |
| Shift + 1-9 | Color of the characters                             |
| r           | Rainbow mode                                        |

## Installation

```bash
cargo install r-matrix-snowfall
```

**Nix**

```bash
nix run github:saylesss88/rmatrix-snowfall
```

**Flake Input**

```nix
rmatrix-snowfall.url = "github:saylesss88/rmatrix-snowfall";
```

NixOS `systemPackages`:

```nix
{ inputs, pkgs, ... }: {
environment.systemPackages = [ inputs.rmatrix-snowfall.packages.${pkgs.stdenv.hostPlatform.system}.default ];
}
```

> To use `inputs`, pass it through `specialArgs`
