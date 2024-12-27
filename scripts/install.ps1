Param()

# Set install directory
$installDir = Join-Path $HOME '.gdvm\bin'
New-Item -ItemType Directory -Force -Path $installDir | Out-Null

# Architecture (assuming Windows)
if ($env:PROCESSOR_ARCHITECTURE -eq 'ARM64') {
    $arch = 'aarch64-pc-windows-msvc'
} elseif ([Environment]::Is64BitOperatingSystem) {
    $arch = 'x86_64-pc-windows-msvc'
} else {
    $arch = 'i686-pc-windows-msvc'
}

$repoUrl   = 'https://github.com/adalinesimonian/gdvm'
$latestUrl = "$repoUrl/releases/latest/download"
$file      = "gdvm-$arch.exe"
$binUrl    = "$latestUrl/$file"
$outFile   = Join-Path $installDir "gdvm.exe"

Write-Host "Downloading gdvm from $binUrl..."
Invoke-WebRequest -Uri $binUrl -OutFile $outFile -UseBasicParsing

# Grant execution rights
& icacls $outFile /grant Everyone:F > $null

Write-Host "gdvm was installed to $outFile"

$failedPaths = @()

function Update-UserPath {
    param($pathToAdd)
    $existingUserPath = [System.Environment]::GetEnvironmentVariable('PATH', [System.EnvironmentVariableTarget]::User)
    if (!$existingUserPath -or (-not $existingUserPath.ToLower().Contains($pathToAdd.ToLower()))) {
        $Env:PATH = "$Env:PATH;$pathToAdd"
        try {
            [System.Environment]::SetEnvironmentVariable('PATH', "$existingUserPath;$pathToAdd", [System.EnvironmentVariableTarget]::User)
            Write-Host "Added $pathToAdd to the user environment PATH."
        } catch {
            $failedPaths += $pathToAdd
        }
        Write-Host "You may need to log out and log back in for the changes to take effect."
    } else {
        Write-Host "$pathToAdd is already in the user environment PATH."
    }
}

# Update current user's PATH for $installDir
Update-UserPath $installDir

$godotDir = Join-Path $installDir 'current_godot'
Update-UserPath $godotDir

if ($failedPaths.Count -gt 0) {
    Write-Host "Failed to update the following paths to the user environment PATH:"
    foreach ($path in $failedPaths) {
        Write-Host "- $path"
    }
    Write-Host "Please add them manually using the following instructions:"
    Write-Host "1. Open Start Search, type 'env', and select 'Edit the system environment variables'."
    Write-Host "2. Under 'User variables', select 'Path' and click 'Edit...'."
    Write-Host "3. Click 'New' and add the paths listed above."
    Write-Host "4. Click 'OK' to close all windows."
}

Write-Host ""
& "$outFile" --version

# Ask to associate .godot files with gdvm (specifically godot.exe in .gdvm/bin)
$godotExe = Join-Path $installDir 'godot.exe'
$godotConsoleExe = Join-Path $installDir 'godot_console.exe'
$godotAssoc = Read-Host "Would you like to associate .godot files with gdvm (specifically godot.exe in .gdvm/bin)? (y/n)"

if ($godotAssoc -eq 'y') {
    try {
        New-Item -Path "HKCU:\Software\Classes\.godot" -Force | Out-Null
        Set-ItemProperty -Path "HKCU:\Software\Classes\.godot" -Name "(Default)" -Value "godot"

        New-Item -Path "HKCU:\Software\Classes\godot" -Force | Out-Null
        Set-ItemProperty -Path "HKCU:\Software\Classes\godot" -Name "(Default)" -Value "Godot Engine Project"

        New-Item -Path "HKCU:\Software\Classes\godot\shell\open\command" -Force | Out-Null
        Set-ItemProperty -Path "HKCU:\Software\Classes\godot\shell\open\command" -Name "(Default)" -Value "$godotExe ""%1"""

        # Add Open with Godot and Open with Godot (show console) to the context menu
        $godotContextMenu = "HKCU:\Software\Classes\godot\shell"
        New-Item -Path $godotContextMenu -Force | Out-Null
        New-Item -Path "$godotContextMenu\Open with Godot" -Force | Out-Null
        New-Item -Path "$godotContextMenu\Open with Godot (show console)" -Force | Out-Null

        New-Item -Path "$godotContextMenu\Open with Godot\command" -Force | Out-Null
        Set-ItemProperty -Path "$godotContextMenu\Open with Godot\command" -Name "(Default)" -Value "$godotExe ""%1"""

        New-Item -Path "$godotContextMenu\Open with Godot (show console)\command" -Force | Out-Null
        Set-ItemProperty -Path "$godotContextMenu\Open with Godot (show console)\command" -Name "(Default)" -Value "$godotConsoleExe ""%1"""

        try {
            $iconUrl = 'https://godotengine.org/favicon.ico'
            $iconPath = Join-Path $installDir 'godot.ico'

            Invoke-WebRequest -Uri $iconUrl -OutFile $iconPath -UseBasicParsing

            # Set the icon for the .godot file type and context menu

            New-Item -Path "HKCU:\Software\Classes\godot\DefaultIcon" -Force | Out-Null
            Set-ItemProperty -Path "HKCU:\Software\Classes\godot\DefaultIcon" -Name "(Default)" -Value "$iconPath,0"

            Set-ItemProperty -Path "$godotContextMenu\Open with Godot" -Name "Icon" -Value "$iconPath,0"
            Set-ItemProperty -Path "$godotContextMenu\Open with Godot (show console)" -Name "Icon" -Value "$iconPath,0"

            # Refresh the icon cache
            $iconCachePath = "$env:LOCALAPPDATA\Microsoft\Windows\Explorer"
            Remove-Item -Path $iconCachePath\iconcache* -Force -ErrorAction SilentlyContinue
        } catch {
            Write-Error "Failed to download the Godot icon."
        }

        Write-Host "Associated .godot files with gdvm."
    } catch {
        Write-Error "Failed to associate .godot files with gdvm."
    }
}

Write-Host ""
Write-Host "To get started, run:"
Write-Host "Usage: gdvm --help"
Write-Host ""
Write-Host "You may possibly need to restart your terminal or IDE (or log out and log back in) for the changes to take effect."
