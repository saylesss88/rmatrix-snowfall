{
  description = "rmatrix-snowfall: A NixOS-themed matrix snowfall screensaver";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = manifest.name;
          version = manifest.version;

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          # Dependencies required for ncurses
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.ncurses ];

          meta = with pkgs.lib; {
            description = manifest.description;
            homepage = manifest.repository;
            license = licenses.gpl3Plus;
            mainProgram = "rmatrix-snowfall";
          };
        };

        # Allows 'nix run'
        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
        };

        # Allows 'nix develop'
        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${system}.default ];
          packages = with pkgs; [
            cargo
            rustc
            rust-analyzer
            clippy
            rustfmt
          ];
        };
      }
    );
}
