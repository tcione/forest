# Example Home Manager configuration for forest
# Add this to your home.nix or use as a standalone config

{
  programs.forest = {
    enable = true;

    # Shell integration for fogo command (default: true)
    # enableBashIntegration = true;
    # enableZshIntegration = true;
    # enableFishIntegration = true;

    settings = {
      general = {
        baseDir = "/home/user/Development";
        copy = [".env" ".envrc"];
        exec = ["direnv allow"];
      };

      roots = {
        "my-project" = {
          copy = [".env.local" "docker-compose.yml"];
          exec = ["npm install" "docker-compose up -d"];
        };

        "work-repo" = {
          copy = [".env.production"];
          exec = ["make setup"];
        };
      };
    };
  };
}
