{
  description = "A tool that manages worktrees for you";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        forest = pkgs.rustPlatform.buildRustPackage {
          pname = "forest";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          # Skip tests in Nix build - they require network access
          doCheck = false;

          meta = with pkgs.lib; {
            description = "A tool that manages worktrees for you";
            homepage = "https://github.com/tcione/forest";
            license = licenses.mit;
            maintainers = [ "@tcione" ];
          };
        };

        nixosModule = import ./nix/nixos-module.nix;
        homeManagerModule = import ./nix/home-manager-module.nix;
      in
      {
        packages = {
          default = forest;
          forest = forest;
        };

        nixosModules.default = nixosModule;
        homeManagerModules.default = homeManagerModule;

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            rustfmt
            rustycli
            cargo
          ];

          shellHook = ''
            echo "🏗️  FOREST!"
            echo ""
          '';
        };
      });
}
