{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.programs.forest;
  tomlFormat = pkgs.formats.toml {};
  forestConfig = {
    general = {
      base_dir = cfg.settings.general.baseDir;
      copy = cfg.settings.general.copy;
      exec = cfg.settings.general.exec;
    };
    roots = cfg.settings.roots;
  };
  configFile = tomlFormat.generate "config.toml" forestConfig;

in {
  options.programs.forest = {
    enable = mkEnableOption "forest worktree management tool";

    package = mkOption {
      type = types.package;
      default = pkgs.forest;
      description = "The forest package to use";
    };

    enableBashIntegration = mkOption {
      type = types.bool;
      default = true;
      description = "Whether to enable Bash integration for fogo command";
    };

    enableZshIntegration = mkOption {
      type = types.bool;
      default = true;
      description = "Whether to enable Zsh integration for fogo command";
    };

    enableFishIntegration = mkOption {
      type = types.bool;
      default = true;
      description = "Whether to enable Fish integration for fogo command";
    };

    settings = {
      general = {
        baseDir = mkOption {
          type = types.str;
          default = "${config.home.homeDirectory}/Projects";
          description = "Base directory for repositories";
        };

        copy = mkOption {
          type = types.listOf types.str;
          default = [ ".env" ".envrc" ];
          description = "Files to copy when creating worktrees";
        };

        exec = mkOption {
          type = types.listOf types.str;
          default = [];
          description = "Commands to execute when creating worktrees";
        };
      };

      roots = mkOption {
        type = types.attrsOf (types.submodule {
          options = {
            copy = mkOption {
              type = types.listOf types.str;
              default = [];
              description = "Repository-specific files to copy";
            };

            exec = mkOption {
              type = types.listOf types.str;
              default = [];
              description = "Repository-specific commands to execute";
            };
          };
        });
        default = {};
        description = "Per-repository configuration overrides";
      };
    };
  };

  config = mkIf cfg.enable {
    home.packages = [ cfg.package ];
    xdg.configFile."forest/config.toml".source = configFile;

    programs.bash.initExtra = mkIf cfg.enableBashIntegration (
      builtins.readFile ./fogo.bash
    );

    programs.zsh.initContent = mkIf cfg.enableZshIntegration (
      builtins.readFile ./fogo.bash
    );

    programs.fish.interactiveShellInit = mkIf cfg.enableFishIntegration (
      builtins.readFile ./fogo.fish
    );
  };
}
