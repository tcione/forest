{
  description = "A CLI tool to make working with easier by establishing a few conventions and abstracting away some git commands";

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
          version = "0.11.1";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          # Skip tests in Nix build - they require network access
          doCheck = false;

          meta = with pkgs.lib; {
            description = "A CLI tool to make working with easier by establishing a few conventions and abstracting away some git commands";
            homepage = "https://github.com/tcione/forest";
            license = with licenses; [ mit asl20 ];
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
