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
      in
      {
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
