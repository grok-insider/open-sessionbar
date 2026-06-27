# Installing open-sessionbar

Two pieces:

1. The **app** (`opensessions`) — a single binary; runs anywhere.
2. The **OpenCode plugin** — embedded in the app; install it with one command.

You never touch npm. The app carries the plugin.

---

## 1. Install the app

### NixOS (flake + Home Manager)

`flake.nix`:
```nix
inputs.open-sessionbar.url = "github:grok-insider/open-sessionbar";
# pass it to home-manager via extraSpecialArgs = { inherit open-sessionbar; };
```

`home.nix`:
```nix
imports = [ open-sessionbar.homeManagerModules.default ];

programs.open-sessionbar = {
  enable = true;
  package = open-sessionbar.packages.${pkgs.stdenv.hostPlatform.system}.default;
  # opt-in: also drop + register the OpenCode plugin on activation
  opencodePlugin.enable = true;
};
```

### Other Linux / macOS

```sh
cargo install --git https://github.com/grok-insider/open-sessionbar --locked
```

Or grab a prebuilt archive from [Releases](https://github.com/grok-insider/open-sessionbar/releases)
(each with a `.sha256`) and put `opensessions` on your `PATH`:

- Linux: `open-sessionbar-<version>-x86_64-unknown-linux-musl.tar.gz` (or `aarch64-…`)
- macOS: `open-sessionbar-<version>-x86_64-apple-darwin.tar.gz` (or `aarch64-…`)

### Windows

Download `open-sessionbar-<version>-x86_64-pc-windows-msvc.zip` from
[Releases](https://github.com/grok-insider/open-sessionbar/releases) (it contains
`opensessions.exe`), or `cargo install --git https://github.com/grok-insider/open-sessionbar --locked`.

---

## 2. Install the OpenCode plugin

```sh
opensessions plugin install
```

This writes the plugin to `~/.config/opencode/plugins/opencode-sessionbar/` and
adds `"./plugins/opencode-sessionbar"` to your `tui.json`. Restart OpenCode.

> On NixOS with `opencodePlugin.enable = true`, this runs automatically on
> activation.

Verify:
```sh
opensessions plugin status     # installed / registered / server live?
```

Manage:
```sh
opensessions plugin update     # after upgrading the binary
opensessions plugin uninstall
```

---

## 3. Wire it into your bar / desktop

The app is desktop-environment-agnostic. Pick the format for your bar:

```sh
opensessions bar --format waybar     # waybar custom module JSON
opensessions bar --format i3blocks   # i3blocks (full/short/color)
opensessions bar --format polybar    # polybar markup
opensessions bar --format eww        # eww JSON (deflisten)
opensessions bar --format plain      # generic text
opensessions json                    # raw snapshot
opensessions watch --format <F>      # stream (SSE), one line per change
opensessions tui                     # live fullscreen popup
```

Copy-paste bar snippets live in [`../contrib/`](../contrib/).
