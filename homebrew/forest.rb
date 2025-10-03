class Forest < Formula
  desc "A tool that manages worktrees for you"
  homepage "https://github.com/tcione/forest"
  version "0.1.0"
  license "MIT OR Apache-2.0"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/tcione/forest/releases/download/v#{version}/forest-macos-x86_64"
      sha256 "SHA256_PLACEHOLDER_INTEL"
    else
      url "https://github.com/tcione/forest/releases/download/v#{version}/forest-macos-aarch64"
      sha256 "SHA256_PLACEHOLDER_ARM"
    end
  end

  depends_on "git"

  def install
    # Install the binary
    if Hardware::CPU.intel?
      bin.install "forest-macos-x86_64" => "forest"
    else
      bin.install "forest-macos-aarch64" => "forest"
    end
  end

  test do
    system "#{bin}/forest", "--help"
    assert_match "Convention-over-configuration CLI tool to manager git worktrees", shell_output("#{bin}/forest --help")
  end
end
