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
outFile="gdvm"

# Windows binaries have a .exe extension
if [ "$os" = "pc-windows-gnu" ]; then
    file="${file}.exe"
    outFile="${outFile}.exe"
fi

binUrl="$latestUrl/$file"
outPath="$installDir/$outFile"

# Download the binary
echo "Downloading gdvm from $binUrl..."
curl -sL "$binUrl" -o "$outPath"
chmod +x "$outPath"

# ...existing code...
echo "gdvm was installed to $outPath"
echo "Make sure $installDir and $installDir/current_godot is in your PATH."

# Add the installation directory to PATH for the current session
if [[ ":$PATH:" != *":$installDir:"* ]] && [[ ":$PATH:" != *":$installDir/current_godot:"* ]]; then
    export PATH="$installDir:$installDir/current_godot:$PATH"
    echo "Updated PATH for the current session."
else
    echo "PATH already includes $installDir or $installDir/current_godot."
fi

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
        if ! grep -Fxq "export PATH=\"$installDir/current_godot:$installDir:\$PATH\"" "$profileFile"; then
            echo "export PATH=\"$installDir/current_godot:$installDir:\$PATH\"" >> "$profileFile"
            echo "Added $installDir to PATH in $profileFile"
        else
            echo "$profileFile already adds $installDir and $installDir/current_godot to PATH."
        fi
    else
        echo "$errorMessage"
    fi
else
    echo "$errorMessage"
fi
echo
echo "To get started, run:"
echo "Usage: gdvm --help"
echo
echo "You may possibly need to restart your shell session or reload your profile file."
