#!/usr/bin/env bash
set -e

repoUrl="https://github.com/adalinesimonian/gdvm"
latestUrl="$repoUrl/releases/latest/download"

# Create a local bin directory if it doesn't exist
installDir="$HOME/.gdvm/bin"
mkdir -p "$installDir"

# Detect OS/arch to download correct binary
os="$(uname -s)"
arch="$(uname -m)"
file="gdvm"

case "$os" in
    Linux*)   os="unknown-linux-gnu" ;;
    Darwin*)  os="apple-darwin" ;;
    MINGW*|MSYS*|CYGWIN*) os="pc-windows-gnu" ;;
    *)        echo "Unsupported OS: $os"; exit 1 ;;
esac

case "$arch" in
    x86_64) arch="x86_64" ;;
    i686)   arch="i686" ;;
    arm64)  arch="aarch64" ;;
    *)      echo "Unsupported ARCH: $arch"; exit 1 ;;
esac

file="gdvm-${arch}-${os}"

# Windows binaries have a .exe extension
if [ "$os" = "pc-windows-gnu" ]; then
    file="${file}.exe"
fi

binUrl="$latestUrl/$file"

# Download the binary
echo "Downloading gdvm from $binUrl..."
curl -sL "$binUrl" -o "$installDir/$file"
chmod +x "$installDir/$file"

# ...existing code...
echo "gdvm was installed to $installDir/$file"
echo "Make sure $installDir and $installDir/current_godot is in your PATH."

# Add the installation directory to PATH for the current session
export PATH="$installDir:$installDir/current_godot:$PATH"

errorMessage="Could not detect shell profile file. Please add \$installDir to your PATH manually.
For example, add the following line to your shell's profile file:
export PATH=\"\$installDir:\$PATH\""

# Detect the user's shell and update the appropriate profile file
if [ -n "$SHELL" ]; then
    shellName=$(basename "$SHELL")
    case "$shellName" in
        bash)
            profileFile="$HOME/.bashrc"
            ;;
        zsh)
            profileFile="$HOME/.zshrc"
            ;;
        fish)
            profileFile="$HOME/.config/fish/config.fish"
            ;;
        *)
            profileFile=""
            ;;
    esac

    if [ -n "$profileFile" ]; then
        echo "export PATH=\"$installDir/current_godot:\$installDir:\$PATH\"" >> "$profileFile"
        echo "Added $installDir to PATH in $profileFile"
    else
        echo "$errorMessage"
    fi
else
    echo "$errorMessage"
fi
echo "Usage: gdvm --help"
