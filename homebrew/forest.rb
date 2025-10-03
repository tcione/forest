class Forest < Formula
  desc "A CLI tool to make working with easier by establishing a few conventions and abstracting away some git commands."
  homepage "https://github.com/tcione/forest"
  version "0.10.0"
  license "MIT OR Apache-2.0"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/tcione/forest/releases/download/v#{version}/forest-macos-x86_64"
      sha256 "e5492f50537b83a92682177109c3b9c51b6d6644c439c4f57e5c801267b03d9d"
    else
      url "https://github.com/tcione/forest/releases/download/v#{version}/forest-macos-aarch64"
      sha256 "290d3c079f3a4b0f5d3b57795f5898b931174d2d00160133ef809ae4a4e45317"
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
