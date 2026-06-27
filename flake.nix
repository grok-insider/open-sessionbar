{
  description = "open-sessionbar — desktop-agnostic OpenCode session monitor (status-bar module + live TUI, fed by an embedded OpenCode plugin)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  # Advertise the binary cache so `nix run/build github:grok-insider/open-sessionbar`
  # can pull prebuilt closures instead of compiling. Users must trust these
  # (Nix will prompt, or add them to nix.settings on NixOS).
  nixConfig = {
    extra-substituters = [
      "https://grok-insider.cachix.org"
      "https://nix-community.cachix.org"
    ];
    extra-trusted-public-keys = [
      "grok-insider.cachix.org-1:ZxLVOxJ1CjdY3vQl1I99qCtwNZwIU4+/QwqSvntB/5w="
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
    ];
  };

  outputs = { self, nixpkgs }:
    let
      lib = nixpkgs.lib;
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = lib.genAttrs systems;

      packageFor = system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        pkgs.rustPlatform.buildRustPackage {
          pname = "open-sessionbar";
          version = (lib.importTOML ./Cargo.toml).package.version;
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          # reqwest uses rustls (no system OpenSSL). The plugin sources are
          # embedded via include_str! at compile time, so plugin/ must be in src
          # (it is — whole repo is the source).
          nativeBuildInputs = [ pkgs.stdenv.cc ];
          buildInputs = [ ];

          meta = {
            description = "Desktop-agnostic OpenCode session monitor (status bar + TUI)";
            mainProgram = "opensessions";
            license = lib.licenses.mit;
            platforms = systems;
          };
        };
    in
    {
      packages = forAllSystems (system: rec {
        default = packageFor system;
        open-sessionbar = default;
      });

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/opensessions";
        };
      });

      homeManagerModules.default = { config, lib, pkgs, ... }:
        let
          cfg = config.programs.open-sessionbar;
        in
        {
          options.programs.open-sessionbar = {
            enable = lib.mkEnableOption "open-sessionbar OpenCode session monitor";

            package = lib.mkOption {
              type = lib.types.package;
              default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
              defaultText = lib.literalExpression "open-sessionbar.packages.\${pkgs.stdenv.hostPlatform.system}.default";
              description = "open-sessionbar package to install (provides the `opensessions` binary).";
            };

            port = lib.mkOption {
              type = lib.types.port;
              default = 4098;
              description = ''
                Port the OpenCode plugin serves on (127.0.0.1). Consumers read
                OPENCODE_SESSIONBAR_PORT or pass --port; this sets the env var
                for the user session.
              '';
            };

            opencodePlugin.enable = lib.mkOption {
              type = lib.types.bool;
              default = false;
              description = ''
                Run `opensessions plugin install` on activation to drop the
                embedded OpenCode TUI plugin into ~/.config/opencode/plugins/ and
                register it in tui.json. Off by default so Nix never edits a
                hand-managed tui.json unless you opt in.
              '';
            };
          };

          config = lib.mkIf cfg.enable {
            home.packages = [ cfg.package ];

            home.sessionVariables = lib.mkIf (cfg.port != 4098) {
              OPENCODE_SESSIONBAR_PORT = toString cfg.port;
            };

            # Idempotent: `plugin install` is a no-op when already registered.
            home.activation.openSessionbarPlugin =
              lib.mkIf cfg.opencodePlugin.enable
                (lib.hm.dag.entryAfter [ "writeBoundary" ] ''
                  run ${cfg.package}/bin/opensessions plugin install --global || true
                '');
          };
        };

      checks = forAllSystems (system: {
        default = self.packages.${system}.default;
      });

      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            packages = [
              pkgs.cargo
              pkgs.rustc
              pkgs.rustfmt
              pkgs.clippy
              pkgs.rust-analyzer
              pkgs.bun
            ];
          };
        });
    };
}
