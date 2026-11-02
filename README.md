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

## How your design survives updates

```
overlay/            YOUR DESIGN: the updater never changes this folder
  files/            new files dropped into TextureCraft (the theme module saltyshade.rs)
  assets/           logo.png (transparent), logo-glow.png, icons/, saltyshade.ico, source/
  rules.json        small, targeted edits to TextureCraft's code (hooks, rename, gizmo, masks)
tools/
  updater.ps1       fetch -> reset upstream/ to the newest commit -> apply overlay -> build -> swap
  apply_overlay.py  lays overlay/ over a fresh upstream checkout
  make_logo.py      re-cuts the transparent logo and icons from overlay/assets/source/
upstream/           pristine TextureCraft clone (managed by the updater: don't edit)
dist/               SaltyShade3D.exe
state/              last applied commit, update.log
```

Every update starts from an **untouched** copy of TextureCraft, then applies your overlay, so
there are never merge conflicts. If TextureCraft changes code that a *required* rule
hooks into, or the build fails, the update stops, your current `SaltyShade3D.exe` keeps working, and
`state/update.log` says which rule needs adjusting. Cosmetic rules that stop matching are skipped
with a warning.

## Changing the design

- Colours and glow: edit `overlay/files/crates/ui-egui/src/saltyshade.rs` (`OBSIDIAN #0b0914`,
  `VIOLET #9d4edd`, `CYAN #00f5d4`, gizmo axis colours, glass transparency).
- Logo: replace `overlay/assets/source/logo-source.jpg`, run `python tools/make_logo.py`.
- Then run `Update-SaltyShade.bat`: edits to `overlay/` trigger a rebuild even when
  TextureCraft hasn't changed.

The Brightness and Accent settings inside the app are overridden by the Neon Cyber-Studio theme.

## Credits

Built on TextureCraft by the ArtCraft team and community (MIT OR Apache-2.0; its license files stay in
`upstream/`). The ArtCraft brand marks are not used in SaltyShade 3D's interface. The SaltyShade 3D logo is your own artwork.
