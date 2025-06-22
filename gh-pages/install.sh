#!/usr/bin/env bash
set -e

repoUrl="https://github.com/adalinesimonian/gdvm"
latestUrl="$repoUrl/releases/latest/download"
apiUrl="https://api.github.com/repos/adalinesimonian/gdvm/releases/latest"

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
echo -e "\e[32m🔄 Downloading gdvm from $binUrl...\e[0m"
curl -sL "$binUrl" -o "$outPath"

echo -e "\e[32m🔄 Fetching checksum from GitHub API...\e[0m"
if [ -n "$GITHUB_TOKEN" ]; then
    apiResponse=$(curl -sL -H "Authorization: token $GITHUB_TOKEN" "$apiUrl")
else
    apiResponse=$(curl -sL "$apiUrl")
fi

# Display any error messages from the API response.
if echo "$apiResponse" | grep -q '"message"'; then
    echo -e "\e[33m⚠️ GitHub API response:\e[0m"
    echo -e "\e[33m$apiResponse\e[0m"
fi

expectedChecksum=$(echo "$apiResponse" | grep -A 30 "\"name\": \"$file\"" | grep '"digest"' | head -1 | cut -d'"' -f4 | cut -d':' -f2)

if [ -z "$expectedChecksum" ]; then
    echo -e "\e[33m⚠️ Warning: Could not fetch checksum from GitHub API. Skipping verification.\e[0m"
else
    echo -e "\e[32m🔍 Verifying checksum...\e[0m"

    if command -v sha256sum >/dev/null 2>&1; then
        actualChecksum=$(sha256sum "$outPath" | cut -d' ' -f1)
    elif command -v shasum >/dev/null 2>&1; then
        actualChecksum=$(shasum -a 256 "$outPath" | cut -d' ' -f1)
    else
        echo -e "\e[33m⚠️ Warning: Neither sha256sum nor shasum found. Skipping checksum verification.\e[0m"
        actualChecksum=""
    fi

    if [ -n "$actualChecksum" ]; then
        if [ "$actualChecksum" = "$expectedChecksum" ]; then
            echo -e "\e[32m✅ Checksum verified successfully.\e[0m"
        else
            echo -e "\e[31m❌ Checksum verification failed!\e[0m"
            echo -e "\e[31mExpected: $expectedChecksum\e[0m"
            echo -e "\e[31mActual:   $actualChecksum\e[0m"
            rm -f "$outPath"
            exit 1
        fi
    fi
fi

chmod +x "$outPath"

echo -e "\e[32m✅ gdvm was installed to $outPath\e[0m"
echo -e "\e[36m🔗 Make sure $installDir and $installDir/current_godot is in your PATH.\e[0m"

# Add the installation directory to PATH for the current session
if [[ ":$PATH:" != *":$installDir:"* ]] && [[ ":$PATH:" != *":$installDir/current_godot:"* ]]; then
    export PATH="$installDir:$installDir/current_godot:$PATH"
    echo -e "\e[32m✅ Updated PATH for the current session.\e[0m"
else
    echo -e "\e[36mℹ️ PATH already includes $installDir or $installDir/current_godot.\e[0m"
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
            echo -e "\e[32m✅ Added $installDir to PATH in $profileFile\e[0m"
        else
            echo -e "\e[36mℹ️ $profileFile already adds $installDir and $installDir/current_godot to PATH.\e[0m"
        fi
    else
        echo -e "\e[31m❌ $errorMessage\e[0m" >&2
    fi
else
    echo -e "\e[31m❌ $errorMessage\e[0m" >&2
fi

echo
"$outPath" --version
echo

# Ask to associate .godot files with gdvm (specifically godot in .gdvm/bin)
# printf "Would you like to associate .godot files with gdvm (specifically godot in $installDir)? [y/N] "
printf "Would you like to create a shortcut for the current Godot version? [y/N] "
read -r REPLY < /dev/tty
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    # echo "Skipping association of .godot files."
    echo "Skipping shortcut creation."
elif [ "$os" = "apple-darwin" ]; then
    echo "Creating a minimal .app to associate .godot files with gdvm on macOS..."

    # Download the Godot universal macOS zip to extract its .icns icon
    icnsZipUrl="https://github.com/godotengine/godot/releases/download/4.3-stable/Godot_v4.3-stable_macos.universal.zip"
    tmpZip="$installDir/godotIconMac.zip"
    haveIcon="false"

    echo -e "\e[32m🔄 Downloading Godot macOS zip to extract icon...\e[0m"
    if ! curl -sL "$icnsZipUrl" -o "$tmpZip"; then
        echo -e "\e[33m⚠️ Failed to download $icnsZipUrl. Continuing without icon.\e[0m" >&2
    else
        # Extract the icon (Godot.icns or Godot.icns). We'll use Godot.icns for a project file icon.
        echo -e "\e[32m🔄 Extracting icon from $tmpZip...\e[0m"
        if command -v unzip >/dev/null 2>&1; then
            # Unzip quietly into a subfolder in $installDir (to not overwrite anything else).
            unzip -qo "$tmpZip" -d "$installDir"

            # Check if the extracted Godot.app and icon file exist:
            extractedIcnsPath="$installDir/Godot.app/Contents/Resources/Godot.icns"
            if [ -f "$extractedIcnsPath" ]; then
                haveIcon="true"
                echo -e "\e[32m✅ Extracted Godot.icns to $extractedIcnsPath\e[0m"
            else
                echo -e "\e[33m⚠️ Godot.icns not found in extracted folder. Will continue without an icon.\e[0m" >&2
            fi
        else
            echo -e "\e[33m⚠️ 'unzip' not available on this system. Cannot extract icon. Continuing without icon.\e[0m" >&2
        fi
    fi

    # Create the minimal .app structure
    userAppDir="$HOME/Applications"

    mkdir -p "$userAppDir"

    appName="Godot (via gdvm).app"
    appDir="$installDir/$appName"
    contentsDir="$appDir/Contents"
    macOsDir="$contentsDir/MacOS"
    resourcesDir="$contentsDir/Resources"

    mkdir -p "$macOsDir" "$resourcesDir"

    # Generate the Info.plist
    infoPlist="$contentsDir/Info.plist"
    cat > "$infoPlist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
   "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>gdvm-godot</string>
    <key>CFBundleDisplayName</key>
    <string>gdvm Godot Stub</string>
    <key>CFBundleIdentifier</key>
    <string>com.example.gdvm-godot</string>
    <key>CFBundleExecutable</key>
    <string>godot-wrapper</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
EOF

    if [ "$haveIcon" = "true" ]; then
        cat >> "$infoPlist" <<EOF
    <key>CFBundleIconFile</key>
    <string>Godot.icns</string>
EOF
    fi

    cat >> "$infoPlist" <<EOF

    <!-- Tells macOS we handle .godot files -->
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>Godot Project</string>
            <key>LSItemContentTypes</key>
            <array>
                <string>com.gdvm.godotproject</string>
            </array>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>godot</string>
            </array>
        </dict>
    </array>

    <key>UTExportedTypeDeclarations</key>
    <array>
        <dict>
            <key>UTTypeIdentifier</key>
            <string>com.gdvm.godotproject</string>
            <key>UTTypeDescription</key>
            <string>Godot Project</string>
            <key>UTTypeConformsTo</key>
            <array>
                <string>public.data</string>
            </array>
            <key>UTTypeTagSpecification</key>
            <dict>
                <key>public.filename-extension</key>
                <string>godot</string>
            </dict>
        </dict>
    </array>
</dict>
</plist>
EOF

    # Create the wrapper script in Contents/MacOS
    cat > "$macOsDir/godot-wrapper" <<'EOF'
#!/usr/bin/env bash

# Path to the actual Godot binary
realGodotBin="__REAL_GODOT_BIN__"

# Execute gdvm, capture both stdout and stderr
output=$("$realGodotBin" "$@" 2>&1)
exit_code=$?

# Display an error alert if gdvm failed
if [ $exit_code -ne 0 ]; then
    # Strip ANSI color codes from the output
    output=$(echo "$output" | sed 's/\x1B\[[0-9;]*[JKmsu]//g')

    # Escape backslashes and double quotes
    escaped_output=$(printf '%s' "$output" | sed 's/\\/\\\\/g; s/"/\\"/g')

    osascript -e "display alert \"gdvm Error\" message \"${escaped_output}\" as critical buttons {\"OK\"}"
fi

# Exit with the original exit code from gdvm
exit $exit_code
EOF
    sed -i '' "s|__REAL_GODOT_BIN__|$installDir/godot|g" "$macOsDir/godot-wrapper"
    chmod +x "$macOsDir/godot-wrapper"

    # If we successfully extracted an icon, place it in Contents/Resources
    if [ "$haveIcon" = "true" ]; then
        cp "$extractedIcnsPath" "$resourcesDir/Godot.icns"
        # Clean up the leftover Godot.app from the extraction
        rm -rf "$installDir/Godot.app"
    fi

    # Also remove the downloaded zip
    rm -f "$tmpZip"

    echo
    echo -e "\e[32m✅ Created a shortcut for the current Godot version in the user Applications folder.\e[0m"


    printf "Would you like to associate .godot files with gdvm (specifically godot in %s)? [y/N] " "$installDir"
    read -r REPLY < /dev/tty
    echo

    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Skipping association of .godot files."
    else
        showManualSteps="false"

        # Optionally try to set defaults with 'duti' if installed:
        if command -v duti >/dev/null 2>&1; then
            echo
            echo -e "\e[32m🔄 Attempting to set .godot -> $appName as default with duti...\e[0m"
            if ! duti -s com.example.gdvm-godot .godot all; then
                echo -e "\e[33m⚠️ Failed to set .godot -> $appName as default with duti.\e[0m" >&2
                showManualSteps="true"
            fi
        elif command -v brew >/dev/null 2>&1; then
            if ! brew install duti; then
                echo -e "\e[33m⚠️ Failed to install 'duti' with Homebrew.\e[0m" >&2
                showManualSteps="true"
            else
                echo
                echo -e "\e[32m🔄 Attempting to set .godot -> $appName as default with duti...\e[0m"
                if ! duti -s com.example.gdvm-godot .godot all; then
                    echo -e "\e[33m⚠️ Failed to set .godot -> $appName as default with duti.\e[0m" >&2
                    showManualSteps="true"
                fi
            fi
        else
            showManualSteps="true"
        fi

        echo
        if [ "$showManualSteps" = "true" ]; then
            echo
            echo -e "\e[36mℹ️ You will need to set the default app manually. To do so:\e[0m"
            echo -e "\e[36m  1) Right-click a .godot file, select 'Get Info'\e[0m"
            echo -e "\e[36m  2) Under 'Open with:', choose $appName\e[0m"
            echo -e "\e[36m  3) Click 'Change All...' to apply for all .godot files.\e[0m"
        fi
    fi
elif [ "$os" = "unknown-linux-gnu" ]; then
    echo "You appear to be on Linux."
    wrapperScript="$installDir/godot-wrapper-linux"
    cat > "$wrapperScript" <<'EOF'
#!/usr/bin/env bash

# Execute gdvm and capture both stdout and stderr
output=$("$HOME/.gdvm/bin/godot" "$@" 2>&1)
exitCode=$?

if [ $exitCode -ne 0 ]; then
    # Strip ANSI color codes from the output
    output="${output//\x1B\[[0-9;]*[JKmsu]/}"

    # Escape < and > characters for safe display with XML-based dialogs
    escaped="${output//</\&lt;}"
    escaped="${escaped//>/\&gt;}"

    zenity --error --text="$escaped" || \
    kdialog --error "$escaped" || \
    Xdialog --msgbox "$escaped" 0 0 || \
    notify-send "gdvm Error" "$escaped" || \
    echo "$output"
fi

exit $exitCode
EOF
    chmod +x "$wrapperScript"
    desktopDir="$HOME/.local/share/applications"
    mkdir -p "$desktopDir"
    desktopFile="$desktopDir/godot-gdvm.desktop"

    cat <<EOF > "$desktopFile"
[Desktop Entry]
Name=Godot (via gdvm)
Exec="$wrapperScript" %f
Type=Application
MimeType=application/x-godot-project;
Comment=Launch Godot projects via gdvm
Hidden=false
Categories=Development;Game;
Terminal=false
EOF

    # Download the Godot icon
    iconUrl="https://godotengine.org/assets/press/icon_color_outline.svg"
    iconFile="$installDir/godot.svg"

    echo -e "\e[32m🔄 Downloading Godot icon from $iconUrl...\e[0m"
    if ! curl -sL "$iconUrl" -o "$iconFile"; then
        echo -e "\e[33m⚠️ Failed to download Godot icon. Will continue without an icon.\e[0m" >&2
    else
        echo "Icon=$iconFile" >> "$desktopFile"
        echo -e "\e[32m✅ Downloaded Godot icon to $iconFile\e[0m"
    fi

    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "$desktopDir" 2>/dev/null || echo -e "\e[33m⚠️ Failed to update desktop database.\e[0m" >&2
    else
        echo -e "\e[33m⚠️ update-desktop-database not found. You may need to install or run it manually.\e[0m" >&2
    fi

    echo -e "\e[32m✅ Created $desktopFile as a shortcut for the current Godot version.\e[0m"
    echo -e "\e[36mℹ️ You can set it as the default app for .godot files in your file manager.\e[0m"
fi
echo
echo -e "\e[36mℹ️ To get started, run:\e[0m"
echo -e "\e[36mUsage: gdvm --help\e[0m"
echo
echo "You may possibly need to restart your shell session or reload your profile file."
