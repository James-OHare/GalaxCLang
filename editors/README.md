# GalaxC Editor Support

This directory contains editor extensions and system integrations for the GalaxC programming language.

## VS Code Extension

To install the GalaxC extension for Visual Studio Code:

1. Copy the `editors/vscode` directory to your extensions folder:
   - **Windows**: `%USERPROFILE%\.vscode\extensions\galaxc-vscode`
   - **macOS/Linux**: `~/.vscode/extensions/galaxc-vscode`
2. Restart VS Code.

Alternatively, you can create a symbolic link:
```powershell
New-Item -ItemType Junction -Path "$HOME\.vscode\extensions\galaxc-vscode" -Target "C:\Users\SIGMA\Documents\GalaxC\editors\vscode"
```

## Windows File Association

To make `.gxc` files recognized by Windows as "GalaxC Source Files" rather than plain text:

1. Double-click the `galaxc_association.reg` file in the root of the repository.
2. Confirm the registry update.

This will:
- Register the `.gxc` extension.
- Set the file type description to "GalaxC Source File".
- Assign a distinct source code icon.
- Add a basic "Open with Notepad" fallback (though VS Code's association will typically take precedence).
