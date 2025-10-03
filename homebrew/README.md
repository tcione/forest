# Forest Homebrew Tap

This is the official Homebrew tap for [forest](https://github.com/tcione/forest), a CLI tool that manages git worktrees for you.

## Installation

### Add the tap and install forest

```bash
# Add the tap
brew tap tcione/forest

# Install forest
brew install forest

# Verify installation
forest --help
```

### One-liner installation

```bash
brew install tcione/forest/forest
```

## What gets installed

- **forest binary**: The main CLI tool
- **Dependencies**: Git (if not already installed)

## Updating

```bash
# Update the tap
brew update

# Upgrade forest
brew upgrade forest
```

## Uninstalling

```bash
# Remove forest
brew uninstall forest

# Remove the tap (optional)
brew untap tcione/forest
```

## Supported Platforms

- **macOS Intel** (x86_64)
- **macOS Apple Silicon** (ARM64/M1/M2/M3)

## Requirements

- **macOS 10.15+** (Catalina or later)
- **Git** (automatically installed if missing)

## Development

To test the formula locally:

```bash
# Install from local formula
brew install --build-from-source ./homebrew/forest.rb

# Test the formula
brew test forest

# Audit the formula
brew audit --strict forest
```

## Support

- **Issues**: [GitHub Issues](https://github.com/tcione/forest/issues)
- **Documentation**: [Main Repository](https://github.com/tcione/forest)
- **License**: MIT OR Apache-2.0
