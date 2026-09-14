{
  description = "Mewtion - a Linux implementation of Apple's Vehicle Motion Cues";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      inherit (nixpkgs) lib;

      # Layer Shell and the IIO sysfs interface are Linux-only.
      systems = lib.filter (lib.hasSuffix "-linux") lib.systems.flakeExposed;
      forAllSystems = f: lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});

      # The overlay links these at build time; the iced control panel loads its
      # wgpu backends at runtime, so they are also wrapped onto the binaries.
      guiLibs = pkgs: with pkgs; [
        glib
        gtk4
        gtk4-layer-shell
        libxkbcommon
        librsvg
        vulkan-loader
        wayland
      ];
    in
    {
      packages = forAllSystems (pkgs: rec {
        mewtion = pkgs.rustPlatform.buildRustPackage {
          pname = "mewtion";
          version = (lib.importTOML ./Cargo.toml).package.version;
          src = self;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [ pkg-config wrapGAppsHook4 ];
          buildInputs = guiLibs pkgs;

          preFixup = ''
            gappsWrapperArgs+=(
              --prefix LD_LIBRARY_PATH : "${lib.makeLibraryPath (guiLibs pkgs)}"
            )
          '';

          meta = {
            description = "Overlay of drifting dots that cue vehicle motion, to reduce motion sickness";
            homepage = "https://github.com/aayuxh-vim/Mewtion";
            license = lib.licenses.mit;
            platforms = systems;
            mainProgram = "Mewtion";
          };
        };

        default = mewtion;
      });

      apps = forAllSystems (pkgs: rec {
        mewtion = {
          type = "app";
          program = lib.getExe self.packages.${pkgs.system}.mewtion;
        };

        control-panel = {
          type = "app";
          program = lib.getExe' self.packages.${pkgs.system}.mewtion "control_panel";
        };

        default = mewtion;
      });

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [ cargo clippy rust-analyzer rustc rustfmt pkg-config ]
            ++ guiLibs pkgs;

          # `cargo run` builds outside the wrapper, so the runtime loader needs
          # the same library path the packaged binaries get.
          LD_LIBRARY_PATH = lib.makeLibraryPath (guiLibs pkgs);
        };
      });

      formatter = forAllSystems (pkgs: pkgs.nixpkgs-fmt);
    };
}
