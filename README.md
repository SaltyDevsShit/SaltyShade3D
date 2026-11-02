# SaltyShade 3D



## Use it

| Double-click | What it does |
|---|---|
| `SaltyShade3D.bat` | Starts SaltyShade 3D, then checks GitHub in the background. A new build is used the next time you start the app. |
| `Update-SaltyShade.bat` | Updates now and shows the output. |

**First time:** run this once (it installs Rust, then builds; the first build takes 10-20 minutes):

```bat
Update-SaltyShade.bat -InstallRust
```

Optional: `Update-SaltyShade.bat -InstallSchedule` also checks for updates every day at noon
(remove the check with `-RemoveSchedule`). `-CheckOnly` only says whether an update is available, and `-Force` rebuilds anyway.

Needs: Windows, git, Python 3 and Rust. `-InstallRust` installs Rust; Visual Studio C++ build tools are used if you have them.

## Importing materials

**Assets > Materials > Import** (or **File > Import material...**) adds a material as a fill layer
with an image per channel:

- **Texture sets**: a folder, a `.zip` or loose images from ambientCG, Poly Haven, Poliigon,
  Substance exports and the like. Base color, normal, roughness (or glossiness, inverted),
  metallic, height, AO, opacity and emissive maps are recognised by name, and each can be
  reassigned. DirectX normals are flipped to OpenGL.
- **Substance `.sbsar`**: the window shows the material's thumbnail, author, presets and every
  parameter. **Render and add fill layer** runs Adobe's `sbsrender` with your settings, then
  imports the maps. A `.sbsar` holds a compiled graph that only Adobe's Substance engine can run,
  so this needs Substance 3D Designer or the Substance 3D Automation Toolkit installed (found
  automatically, or via **Locate sbsrender.exe...**, or `SALTYSHADE_SBSRENDER`). Without it, the window
  says so and offers the texture-set import instead.

The code is `overlay/files/crates/ui-egui/src/saltyshade_import.rs`.

## Sharing it: installer and auto-updates for other people

Send people the newest `SaltyShade3D-<version>-Setup.exe` from your GitHub repo's **Releases**
page. It installs per user (no admin prompt) with a desktop icon, a Start menu entry and an
uninstaller (Settings > Apps).

The icon opens `SaltyShade3D-Launcher.exe`. It asks your GitHub releases for a newer version
(a 4-second check, skipped when offline), installs it silently if there is one, then opens the app.

New releases come from `.github/workflows/release.yml` on GitHub's servers. Every day it checks
TextureCraft for changes, and when TextureCraft or your design changed it builds, packages and
publishes a release. Pushing a design change publishes one right away. You can also run it by hand
from Actions > Release > Run workflow. Your PC doesn't need to be on.

To make an installer locally (testing, or sending one by hand): `Make-Installer.bat`. It packages
`dist\SaltyShade3D.exe` from the last `Update-SaltyShade.bat` build. `release.json` names the repo
that locally made installers update from.

| Piece | File |
|---|---|
| Launcher (update check) | `launcher/src/main.rs` |
| Installer (Inno Setup) | `installer/SaltyShade3D.iss`, banner `installer/wizard.png` |
| Packaging script | `tools/make_installer.ps1` |
| Release pipeline | `.github/workflows/release.yml` |

The installer isn't code-signed, so Windows SmartScreen shows "Windows protected your PC" the first
time someone runs it (More info > Run anyway). Silent updates after that don't show it. A
code-signing certificate removes the warning.


## Credits

Built on TextureCraft by the ArtCraft team and community (MIT OR Apache-2.0; its license files stay in
`upstream/`). The ArtCraft brand marks are not used in SaltyShade 3D's interface. The SaltyShade 3D logo is your own artwork.
