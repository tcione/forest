{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.programs.forest;

in {
  options.programs.forest = {
    enable = mkEnableOption "forest worktree management tool";

    package = mkOption {
      type = types.package;
      default = pkgs.forest;
      description = "The forest package to use";
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    # Note: NixOS module provides system-wide installation only
    # For user configuration, use the Home Manager module instead
  };
}