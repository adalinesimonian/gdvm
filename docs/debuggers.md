# Using gdvm with debuggers

While for most purposes it is more than enough to run `godot`, the shim provided by gdvm, or `gdvm run` directly, debuggers often need to attach directly to the Godot process. To do so, they typically require a path to the Godot binary to launch or attach to.

In these cases, gdvm can provide that path in a way that is derived from your pinned version and is easy to refresh when you change versions, and works on all platforms.

## General pattern

1. Pin the version you want for the project, for example:

   ```bash
   gdvm pin 4.5.1-stable --csharp
   ```

2. Use `gdvm link` to create a project-local Godot binary or app bundle that your debugger can point at, for example inside a `.gdvm/` directory in your workspace:

   ```bash
   # Linux and other Unix-like systems
   gdvm link --csharp --force .gdvm/godot

   # Windows
   gdvm link --csharp --console --force .gdvm/godot.exe

   # macOS (the binary is inside the .app bundle)
   gdvm link --csharp --force .gdvm/Godot.app
   ```

Any tool that can launch an executable from a path can use this: configure it to use the appropriate `.gdvm/...` path for your platform. When you change the pinned version or install a new one, rerun `gdvm link` or have your debugger run it as a pre-launch step so the link is updated.

## Example: debugging a C# project in Visual Studio Code

For Visual Studio Code with the C# debugger, you can automate this pattern using a task and launch configuration in your project.

First, add `.gdvm/` to your `.gitignore` so the linked binary or bundle is not checked into version control:

```gitignore
# Ignore gdvm linked binaries
/.gdvm
```

Then, create `.vscode/tasks.json`:

```jsonc
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "link-godot-binary",
      "command": "gdvm",
      "detail": "Link/copy Godot via gdvm into .gdvm for this workspace.",
      "args": ["link", "--csharp", "--force", "${workspaceFolder}/.gdvm/godot"],
      "windows": {
        "args": [
          "link",
          "--csharp",
          "--force",
          "${workspaceFolder}\\.gdvm\\godot.exe",
        ],
      },
      "osx": {
        "args": [
          "link",
          "--csharp",
          "--force",
          "${workspaceFolder}/.gdvm/Godot.app",
        ],
      },
    },
  ],
}
```

Then, create `.vscode/launch.json`:

```jsonc
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Godot via gdvm (launch)",
      "type": "coreclr",
      "request": "launch",
      "program": "${workspaceFolder}/.gdvm/godot",
      "args": ["--path", "${workspaceFolder}"],
      "cwd": "${workspaceFolder}",
      "preLaunchTask": "link-godot-binary",
      "windows": {
        "program": "${workspaceFolder}\\.gdvm\\godot.exe",
      },
      "osx": {
        "program": "${workspaceFolder}/.gdvm/Godot.app/Contents/MacOS/Godot",
      },
    },
  ],
}
```

With this setup, the `link-godot-binary` task runs `gdvm link` before each debug session, creating or updating the project-local Godot binary or bundle in `.gdvm/` based on your current pinned version. If you change the pinned version, the next debug run will recreate the link automatically.

This way, the VS Code debugger always launches the right Godot for the project without hardcoding any installation paths.

## Other debuggers and integrations

You can apply the same general pattern with other debuggers and IDEs:

- Link the Godot binary or app bundle into a known location inside your project using `gdvm link`, and ignore that location in version control.
- Configure your debugger, or the debugger integration inside your IDE, to launch that path with the appropriate arguments, for example `--path /path/to/project`.
- If your debugger does not support pre-launch tasks, make sure to rerun `gdvm link` yourself whenever you change the pinned version or switch to a different installed version.
