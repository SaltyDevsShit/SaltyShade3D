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

SaltyShade 3D can bring in materials made elsewhere. Every import ends the same way: a new **fill
layer** in the Layers panel, named after the material, with each of its maps plugged into the
matching channel.

### Opening the importer

Either of these opens the **Import material** window:

- the **Import** button next to the **Materials** heading in the **Assets** panel (left side, on
  the **Materials** tab: the sphere icon, shown by default), or
- **File > Import material (maps, .zip, .sbsar)...** in the menu bar.

The window offers two buttons:

| Button | Use it for |
|---|---|
| **Choose files...** | A `.sbsar` file, a `.zip` of maps, or one or more map images selected together (hold Ctrl to pick several). |
| **Choose folder...** | A folder of map images. Subfolders up to two levels deep are searched too. |

What you pick decides what comes next: a `.sbsar` opens the [Substance window](#substance-materials-sbsar),
anything else opens the [texture set window](#texture-sets-folders-zips-and-images).

### What the channels mean

A PBR material is a set of images (*maps*). Each map controls one property of the surface:

| Channel | What it controls | Typical file name ending |
|---|---|---|
| **Base color** | The surface colour with no lighting or shadows baked in. | `_Color`, `_BaseColor`, `_Albedo`, `_Diffuse` |
| **Normal** | Fine surface bumps that catch the light, without changing the shape. | `_Normal`, `_NormalGL`, `_NormalDX`, `_nor_gl` |
| **Roughness** | Black = mirror-smooth and glossy, white = rough and matte. | `_Roughness`, `_Rough` |
| **Metallic** | White = bare metal, black = everything else (paint, rust, plastic, wood). | `_Metallic`, `_Metalness` |
| **Height** | Raised (white) and sunken (black) areas, for deeper relief than a normal map. | `_Height`, `_Displacement`, `_Disp` |
| **Ambient occlusion** | Soft contact shadows in cracks and crevices. | `_AO`, `_AmbientOcclusion` |
| **Opacity** | White = solid, black = see-through (leaves, fences, decals). | `_Opacity`, `_Alpha` |
| **Emissive** | Parts that glow by themselves (screens, lights, lava). | `_Emissive`, `_Emission` |

### Texture sets (folders, zips and images)

This works with free and paid material libraries: [ambientCG](https://ambientcg.com),
[Poly Haven](https://polyhaven.com/textures), Poliigon, Quixel, textures exported from Substance
3D Painter or Designer, or your own maps. It needs no extra software.

1. Download the material. A `.zip` doesn't need to be unpacked first.
2. Click **Import** and use **Choose files...** for the `.zip` or images, or **Choose folder...** for
   an unpacked folder.
3. The window lists every image it found, with the channel it guessed:
   - **Layer name**: what the new layer will be called. It comes from the file name with the
     resolution and format removed (`Metal009_2K-PNG.zip` becomes `Metal009`). Type to change it.
   - **The list**: each file with a dropdown showing its channel. If a guess is wrong, pick the
     right channel. Pick **Skip** to leave a file out (previews, sphere renders, `.blend` thumbnails).
   - **Normal map is DirectX (green down); flip it to OpenGL**: see [normal maps](#directx-and-opengl-normal-maps).
     It's ticked automatically when the normal map's name ends in `DX`.
4. Click **Add as fill layer**. A green message confirms how many maps were added, and the layer
   appears above the selected layer.

**How names are recognised.** The *last word* of each file name decides its channel. Words are
split on `_`, `-`, spaces and dots; resolution and format words (`1K`, `2K`, `4K`, `PNG`, `JPG`,
`EXR`, `16bit`...) are ignored first, and a final `GL` or `DX` only tells which normal type it is.
So `Rock051_4K-PNG_Roughness.png` is **Roughness**, and `metal_plate_nor_dx_2k.exr` is a
**Normal** (DirectX). Glossiness maps (`_Gloss`, `_Glossiness`) are the opposite of roughness, so
they're inverted automatically and used as **Roughness**; the list shows *Roughness (from glossiness)*.

If a set includes both a DirectX and an OpenGL normal map, the OpenGL one is used and the other is
set to **Skip**. Only the first map found for each channel is used.

Supported image formats: PNG, JPG, TGA, TIFF, EXR, HDR, WebP and BMP.

### Substance materials (.sbsar)

A `.sbsar` file is a **procedural** material: rather than fixed images, it contains a recipe that
can produce endless variations (more rust, different metal, new scratch patterns) at any
resolution. That recipe is compiled for **Adobe's Substance engine**, and no other software can run
it. So SaltyShade 3D uses Adobe's command-line renderer, `sbsrender.exe`, to turn your chosen
settings into maps, then imports them like a texture set.

**What you need:** `sbsrender.exe` comes with **Adobe Substance 3D Designer** and the **Substance 3D
Automation Toolkit** (both need an Adobe Substance licence). Substance 3D Painter and Sampler don't
include it. SaltyShade 3D looks for it automatically in:

1. the `SALTYSHADE_SBSRENDER` environment variable (full path to `sbsrender.exe`),
2. the location you chose earlier with **Locate sbsrender.exe...**,
3. any folder on your `PATH`,
4. `C:\Program Files\Adobe\<any Substance folder>\` and `C:\Program Files\Allegorithmic\<any Substance folder>\`.

**Opening a `.sbsar`:** click **Import**, then **Choose files...**, and pick the `.sbsar`. The top of the
window always shows:

| Item | Meaning |
|---|---|
| Thumbnail | The preview image the material's author packed into the file. |
| Name and **by ...** | The material's name and author, as written in the file. |
| **Author's page** | Opens the author's web page for this material, if the file names one. |
| **Maps:** | The images the material produces (for example `basecolor, normal, roughness, metallic, height`). |

#### If Substance isn't installed

The window says **Rendering .sbsar needs Adobe's Substance engine** and offers:

- **Locate sbsrender.exe...**: point to `sbsrender.exe` yourself if it's installed somewhere unusual.
  SaltyShade 3D remembers it.
- **Import texture maps instead...**: goes back to the file picker. Many sites (ambientCG among them)
  offer the same material as a PNG texture set. Download that version and import it.

#### If Substance is installed

You get the full set of controls:

| Control | What it does |
|---|---|
| **Preset** | Ready-made looks saved by the material's author (for example `1A`, `2B`). Choosing one sets every parameter below to the author's values. **Default** returns to the material's starting values. |
| **Size** | Resolution of the rendered maps: 512, 1K (1024), 2K (2048) or 4K (4096) pixels square. Larger is sharper but slower to render and uses more memory. 2K is a good default. |
| **Seed** | A random-number starting point. Changing it reshuffles random details (where the rust spots, scratches and dents fall) without changing their style. |
| Parameters | The material's own settings, as the author named them. **Sliders** set amounts, **dropdowns** choose between options (Base Color: Iron, Copper, Gold...), **checkboxes** switch features on or off (Enable Rusting), and **colour swatches** open a colour picker. Hover a name to see its internal ID. |
| Groups (▸ Oxidation/Rust, ▸ Scratches...) | Related parameters, folded away to keep the window short. Click to open. |
| **Normal output is DirectX (Substance's default); flip it to OpenGL** | Substance materials normally produce DirectX normal maps; leave this ticked unless the material's author says otherwise. See [normal maps](#directx-and-opengl-normal-maps). |
| **Render and add fill layer** | Sends your settings to `sbsrender`, waits for the maps (the button shows *Rendering...* with a spinner), then adds the fill layer. You can keep tweaking and render again: each render adds a new layer, so you can compare versions. |
| **Reset** | Puts every parameter back to the material's defaults (and the preset back to **Default**). |
| **Engine:** | The `sbsrender.exe` being used. |

Only settings that differ from the material's defaults are sent to `sbsrender`, so an unchanged
material renders exactly as its author intended.

### After importing

- The new fill layer sits **above the layer that was selected**, and becomes the selected layer.
- Each map becomes an image on the **Textures** tab of **Assets** (the picture icon) and is **saved inside your project**, so the
  project still works if the original files are moved or deleted.
- The layer uses **UV projection**. If the model has no good UVs, or the texture looks stretched,
  change **Projection** to **Tri-planar** in the **Properties** panel, and use **Scale / tiling** to make
  the pattern bigger or smaller.
- Everything works like any other fill layer: add a **mask** to paint where the material appears,
  change blending and opacity, or drag it into a folder.
- If a map seems to have no effect, the texture set may not have that channel switched on (Height
  and Emissive often aren't). Turn it on in **Texture Set Settings**.
- **Edit > Undo** (Ctrl+Z) removes an import step by step.

### DirectX and OpenGL normal maps

Normal maps come in two conventions that differ only in the green channel: **OpenGL** (green up,
used by Blender, Unity and SaltyShade 3D) and **DirectX** (green down, used by Unreal and Substance).
A DirectX map used as OpenGL makes bumps look like dents. The checkbox flips the green channel while
importing so the map always ends up as OpenGL. If the bumps on your model look inverted after
importing, import again with the checkbox the other way.

### Troubleshooting

| Problem | Fix |
|---|---|
| **No images found there.** | The folder or zip has no supported images (or they're nested more than two folders deep). Unpack it and pick the folder that contains the maps. |
| A map got the wrong channel | Change its dropdown before clicking **Add as fill layer**. |
| **Choose a channel for at least one map.** | Every file is set to **Skip**. Give at least one a channel. |
| **could not unpack the .sbsar** | The file is damaged or isn't a real `.sbsar`. Download it again. |
| **sbsrender failed: ...** | The message is the last line `sbsrender` printed. Usually the Substance licence has expired or isn't signed in, or the `.sbsar` needs a newer Substance version. |
| Bumps look inside out | Re-import with the DirectX/OpenGL checkbox flipped. |
| Material looks stretched | Set **Projection** to **Tri-planar** in Properties. |

### Limits

- **Packed maps** (one image holding AO, roughness and metallic in its red, green and blue, often
  named `_ARM` or `_ORM`) aren't split up. Use a set with separate maps, or set the packed image to
  **Skip**.
- A `.sbsar` with **several materials** inside shows the first one; `sbsrender` renders the others
  too, and maps with the same output name overwrite each other.
- `.sbsar` **image, text and font inputs** (for example "use your own logo") aren't shown; the
  material uses the default its author set.
- Substance materials can't be imported in the web version of SaltyShade 3D.

### Where import files are kept

| Folder | Contents |
|---|---|
| `%LOCALAPPDATA%\SaltyShade3D\zips\` | Images unpacked from imported `.zip` files. |
| `%LOCALAPPDATA%\SaltyShade3D\sbsar\` | Unpacked `.sbsar` descriptions and thumbnails. |
| `%LOCALAPPDATA%\SaltyShade3D\sbsar-render\` | Maps produced by the last render of each `.sbsar`. |
| `%APPDATA%\SaltyShade3D\sbsrender.txt` | The `sbsrender.exe` location chosen with **Locate**. |

These are working copies (the maps themselves are stored in your project), so the folders are safe
to delete. Each new import of the same material replaces its folder.

The importer's code is `overlay/files/crates/ui-egui/src/saltyshade_import.rs`, hooked into the
menu and Assets panel by the *Import material* rules in `overlay/rules.json`.

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
