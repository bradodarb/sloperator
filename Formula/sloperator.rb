# Rendered by packaging/render-formula.sh from packaging/sloperator.rb.tmpl on each release. Edit the template.
class Sloperator < Formula
  desc "Lint and rewrite LLM-generated prose so it reads as human-written"
  homepage "https://github.com/bradodarb/sloperator"
  license "MIT"

  on_macos do
    depends_on arch: :arm64

    on_arm do
      url "https://github.com/bradodarb/sloperator/releases/download/v0.1.0/sloperator-0.1.0-aarch64-apple-darwin.tar.gz"
      sha256 "cd3fa2727c3ca8c7d1b41b493e1e1fa24de2ef4ee254b3b65e2d3c48c3d6259b"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/bradodarb/sloperator/releases/download/v0.1.0/sloperator-0.1.0-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "52e795a9e48fb1f3bdf716f60747f1c8cdb9e2e3ee395c1e67d0aaab5319c214"
    end
    on_intel do
      url "https://github.com/bradodarb/sloperator/releases/download/v0.1.0/sloperator-0.1.0-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "899fe04bec1b65d14b5fb4dceb3d8702384f1ad72524aa175aa5d86a9670a782"
    end
  end

  def install
    bin.install "sloperator"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/sloperator --version")
  end
end
