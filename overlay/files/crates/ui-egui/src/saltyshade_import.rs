//! SaltyShade 3D: Import material (File menu and the Materials header in Assets).
//!
//! Two kinds of material become one fill layer with an image per channel:
//! - PBR texture sets: a folder, a .zip or loose images (ambientCG, Poly Haven, maps exported
//!   from Substance and the like). Maps are recognised by their file names.
//! - Substance materials (.sbsar): the archive's descriptor gives the name, thumbnail, presets
//!   and parameters; Adobe's `sbsrender` (Substance 3D Designer or the Automation Toolkit)
//!   renders the maps, which are then imported like a texture set. Without it the window says
//!   what is needed. The .sbsar format's compiled graphs only run in Adobe's engine.
//!
//! Part of the SaltyShade overlay (`overlay/files/`); edit it there, not in `upstream/`.

use crate::TextureCraftApp;

#[cfg(target_arch = "wasm32")]
pub fn open(_app: &mut TextureCraftApp) {}

#[cfg(target_arch = "wasm32")]
pub fn show(_app: &mut TextureCraftApp, _ctx: &egui::Context) {}

#[cfg(not(target_arch = "wasm32"))]
pub use native::{open, show};

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::Mutex;
    use std::sync::mpsc::{Receiver, channel};

    use egui::{RichText, vec2};
    use serde_json::json;

    use super::TextureCraftApp;
    use crate::theme::Tokens;

    /// Channels a map can feed, with their labels ("" id: not imported).
    const CHANNELS: [(&str, &str); 8] = [
        ("baseColor", "Base color"),
        ("normal", "Normal"),
        ("roughness", "Roughness"),
        ("metallic", "Metallic"),
        ("height", "Height"),
        ("ambientOcclusion", "Ambient occlusion"),
        ("opacity", "Opacity"),
        ("emissive", "Emissive"),
    ];
    const IMAGE_EXTS: [&str; 10] = ["png", "jpg", "jpeg", "tga", "tif", "tiff", "exr", "webp", "bmp", "hdr"];
    const SIZES: [(u32, &str); 4] = [(9, "512"), (10, "1K"), (11, "2K"), (12, "4K")];

    static STATE: Mutex<Option<Importer>> = Mutex::new(None);

    struct Importer {
        open: bool,
        stage: Stage,
        status: Option<(String, bool)>,
        render: Option<Receiver<Result<Vec<PathBuf>, String>>>,
    }

    enum Stage {
        Pick,
        Maps(MapSet),
        Sbsar(Box<Sbsar>),
    }

    #[derive(Clone)]
    struct Map {
        path: PathBuf,
        /// Index into [`CHANNELS`]; None: skipped.
        channel: Option<usize>,
        /// Glossiness (imported inverted, as roughness).
        gloss: bool,
    }

    #[derive(Clone, Default)]
    struct MapSet {
        name: String,
        maps: Vec<Map>,
        normal_dx: bool,
    }

    struct Sbsar {
        file: PathBuf,
        label: String,
        author: String,
        author_url: String,
        thumb: Option<egui::TextureHandle>,
        outputs: Vec<String>,
        inputs: Vec<Input>,
        presets: Vec<(String, Vec<(String, String)>)>,
        preset: Option<usize>,
        size: usize,
        seed: i32,
        normal_dx: bool,
        engine: Option<PathBuf>,
    }

    struct Input {
        id: String,
        label: String,
        group: String,
        ty: u32,
        widget: String,
        min: f32,
        max: f32,
        items: Vec<(i32, String)>,
        value: Vec<f32>,
        default: Vec<f32>,
    }

    fn lock() -> std::sync::MutexGuard<'static, Option<Importer>> {
        STATE.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Open the Import material window.
    pub fn open(_app: &mut TextureCraftApp) {
        *lock() = Some(Importer { open: true, stage: Stage::Pick, status: None, render: None });
    }

    /// Draw the window (every frame; nothing when closed).
    pub fn show(app: &mut TextureCraftApp, ctx: &egui::Context) {
        let mut guard = lock();
        let Some(st) = guard.as_mut() else { return };
        if !st.open {
            *guard = None;
            return;
        }
        // A finished render becomes a texture set import.
        if let Some(rx) = &st.render {
            match rx.try_recv() {
                Ok(Ok(files)) => {
                    st.render = None;
                    if let Stage::Sbsar(sb) = &st.stage {
                        let mut set = classify(&sb.label, &files);
                        set.normal_dx = sb.normal_dx;
                        st.status = Some(add_layer(app, &set));
                    }
                }
                Ok(Err(e)) => {
                    st.render = None;
                    st.status = Some((e, true));
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => ctx.request_repaint_after(std::time::Duration::from_millis(200)),
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    st.render = None;
                    st.status = Some(("The render stopped unexpectedly.".into(), true));
                }
            }
        }
        let tk = Tokens::get(ctx);
        let mut open = st.open;
        egui::Window::new("Import material")
            .id(egui::Id::unique("saltyshade-import"))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(ctx.content_rect().center())
            .default_width(460.0)
            .show(ctx, |ui| {
                let mut next = None;
                match &mut st.stage {
                    Stage::Pick => next = pick_ui(ui, ctx, &tk),
                    Stage::Maps(set) => {
                        if let Some(r) = maps_ui(ui, &tk, set) {
                            if r {
                                st.status = Some(add_layer(app, set));
                            } else {
                                next = Some(Stage::Pick);
                            }
                        }
                    }
                    Stage::Sbsar(sb) => {
                        let busy = st.render.is_some();
                        match sbsar_ui(ui, ctx, &tk, sb, busy) {
                            Some(SbsarAction::Render) => {
                                st.status = Some(("Rendering with Adobe's Substance engine...".into(), false));
                                st.render = Some(start_render(sb));
                            }
                            Some(SbsarAction::Maps) => next = Some(Stage::Pick),
                            None => {}
                        }
                    }
                }
                if let Some((msg, error)) = &st.status {
                    ui.add_space(6.0);
                    ui.label(RichText::new(msg).color(if *error { tk.danger } else { crate::saltyshade::CYAN }));
                }
                if let Some(s) = next {
                    st.stage = s;
                    st.status = None;
                }
            });
        st.open = open;
    }

    // ---------------------------------------------------------------------------------------
    // Picking

    fn pick_ui(ui: &mut egui::Ui, ctx: &egui::Context, tk: &Tokens) -> Option<Stage> {
        ui.label("Bring in a material as a fill layer with an image per channel.");
        ui.add_space(6.0);
        let mut picked: Option<Vec<PathBuf>> = None;
        ui.horizontal(|ui| {
            let b = crate::widgets::primary_button(ui, "Choose files...");
            if ui.add(b).on_hover_text(".sbsar, .zip, or the map images").clicked() {
                picked = rfd::FileDialog::new()
                    .add_filter("Materials", &["sbsar", "zip", "png", "jpg", "jpeg", "tga", "tif", "tiff", "exr", "webp", "bmp", "hdr"])
                    .pick_files();
            }
            if ui.button("Choose folder...").on_hover_text("A folder of map images").clicked() {
                picked = rfd::FileDialog::new().pick_folder().map(|d| vec![d]);
            }
        });
        ui.add_space(4.0);
        ui.label(RichText::new("Texture sets: base color, normal, roughness, metallic, height, AO, opacity and emissive maps are recognised by name (ambientCG, Poly Haven, Substance exports...).").size(11.0).color(tk.text_dim));
        ui.label(RichText::new("Substance .sbsar: rendered with Adobe's sbsrender (Substance 3D Designer or Automation Toolkit).").size(11.0).color(tk.text_dim));
        let files = picked?;
        Some(load(ctx, files))
    }

    fn load(ctx: &egui::Context, files: Vec<PathBuf>) -> Stage {
        let ext = |p: &Path| p.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
        if let Some(sb) = files.iter().find(|f| ext(f) == "sbsar") {
            return match open_sbsar(ctx, sb) {
                Ok(s) => Stage::Sbsar(Box::new(s)),
                Err(e) => Stage::Maps(MapSet { name: format!("Error: {e}"), ..Default::default() }),
            };
        }
        let mut images = Vec::new();
        for f in &files {
            if f.is_dir() {
                collect_images(f, &mut images, 2);
            } else if ext(f) == "zip" {
                match unzip(f) {
                    Ok(dir) => collect_images(&dir, &mut images, 3),
                    Err(e) => return Stage::Maps(MapSet { name: format!("Error: {e}"), ..Default::default() }),
                }
            } else if IMAGE_EXTS.contains(&ext(f).as_str()) {
                images.push(f.clone());
            }
        }
        let name = files.first().map(|f| set_name(f)).unwrap_or_else(|| "Material".into());
        Stage::Maps(classify(&name, &images))
    }

    fn collect_images(dir: &Path, out: &mut Vec<PathBuf>, depth: u32) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        let mut entries: Vec<_> = rd.flatten().map(|e| e.path()).collect();
        entries.sort();
        for p in entries {
            if p.is_dir() && depth > 0 {
                collect_images(&p, out, depth - 1);
            } else if p.extension().and_then(|e| e.to_str()).is_some_and(|e| IMAGE_EXTS.contains(&e.to_ascii_lowercase().as_str())) {
                out.push(p);
            }
        }
    }

    /// A readable material name from a file or folder name ("Metal009_2K-PNG" → "Metal009").
    fn set_name(p: &Path) -> String {
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("Material");
        let cut = stem.split(['_', '-', ' ']).take_while(|t| !is_noise(&t.to_ascii_lowercase())).collect::<Vec<_>>().join(" ");
        if cut.is_empty() { stem.to_string() } else { cut }
    }

    /// Resolution and format tokens in map names.
    fn is_noise(t: &str) -> bool {
        let res = t.len() >= 2 && t.ends_with('k') && t[..t.len() - 1].chars().all(|c| c.is_ascii_digit());
        res || matches!(t, "png" | "jpg" | "jpeg" | "exr" | "tif" | "tiff" | "tga" | "8bit" | "16bit" | "32bit" | "compiled")
    }

    /// Recognise each map's channel from its file name: the last meaningful word decides.
    fn classify(name: &str, files: &[PathBuf]) -> MapSet {
        let mut set = MapSet { name: name.to_string(), ..Default::default() };
        let mut taken = [false; CHANNELS.len()];
        // OpenGL normals first: with both in a set, the one that needs no flip wins.
        let mut files = files.to_vec();
        files.sort_by_key(|f| {
            let s = f.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase();
            s.ends_with("dx") || s.ends_with("directx")
        });
        for f in &files {
            let stem = f.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase();
            let mut toks: Vec<&str> = stem.split(|c: char| !c.is_ascii_alphanumeric()).filter(|t| !t.is_empty() && !is_noise(t)).collect();
            let mut dx = false;
            if let Some(&last) = toks.last() {
                if matches!(last, "dx" | "directx") {
                    dx = true;
                    toks.pop();
                } else if matches!(last, "gl" | "opengl") {
                    toks.pop();
                }
            }
            let last = toks.last().copied().unwrap_or("");
            let (ch, gloss, dx2) = match last {
                "basecolor" | "albedo" | "diffuse" | "diff" | "color" | "colour" | "col" | "base" => (Some(0), false, false),
                "normal" | "normalgl" | "nrm" | "nor" | "norm" | "normals" => (Some(1), false, false),
                "normaldx" => (Some(1), false, true),
                "roughness" | "rough" | "rgh" => (Some(2), false, false),
                "glossiness" | "gloss" => (Some(2), true, false),
                "metallic" | "metalness" | "metal" | "mtl" => (Some(3), false, false),
                "height" | "displacement" | "disp" | "bump" | "heightmap" => (Some(4), false, false),
                "ao" | "ambientocclusion" | "occlusion" | "ambient" => (Some(5), false, false),
                "opacity" | "alpha" | "transparency" => (Some(6), false, false),
                "emissive" | "emission" | "emit" | "glow" => (Some(7), false, false),
                _ => (None, false, false),
            };
            // The first map per channel wins (of NormalDX and NormalGL, whichever sorts first);
            // the normal that is used decides whether it needs flipping.
            let ch = ch.filter(|&c| !taken[c]);
            if let Some(c) = ch {
                taken[c] = true;
                if c == 1 {
                    set.normal_dx = dx || dx2;
                }
            }
            set.maps.push(Map { path: f.clone(), channel: ch, gloss });
        }
        set
    }

    fn maps_ui(ui: &mut egui::Ui, tk: &Tokens, set: &mut MapSet) -> Option<bool> {
        if set.name.starts_with("Error: ") {
            ui.label(RichText::new(&set.name).color(tk.danger));
            return ui.button("Back").clicked().then_some(false);
        }
        if set.maps.is_empty() {
            ui.label(RichText::new("No images found there.").color(tk.danger));
            return ui.button("Back").clicked().then_some(false);
        }
        ui.horizontal(|ui| {
            ui.label("Layer name");
            ui.text_edit_singleline(&mut set.name);
        });
        ui.add_space(4.0);
        egui::Grid::new("saltyshade-maps").num_columns(2).striped(true).spacing(vec2(10.0, 4.0)).show(ui, |ui| {
            for (i, m) in set.maps.iter_mut().enumerate() {
                let file = m.path.file_name().and_then(|n| n.to_str()).unwrap_or("?").to_string();
                ui.label(RichText::new(file).size(11.5));
                let text = match m.channel {
                    Some(c) if m.gloss => format!("{} (from glossiness)", CHANNELS[c].1),
                    Some(c) => CHANNELS[c].1.to_string(),
                    None => "Skip".to_string(),
                };
                egui::ComboBox::from_id_salt(("saltyshade-map", i)).selected_text(text).width(170.0).show_ui(ui, |ui| {
                    ui.selectable_value(&mut m.channel, None, "Skip");
                    for (c, (_, label)) in CHANNELS.iter().enumerate() {
                        ui.selectable_value(&mut m.channel, Some(c), *label);
                    }
                });
                ui.end_row();
            }
        });
        ui.add_space(4.0);
        ui.checkbox(&mut set.normal_dx, "Normal map is DirectX (green down); flip it to OpenGL");
        ui.add_space(6.0);
        let mut out = None;
        ui.horizontal(|ui| {
            let b = crate::widgets::primary_button(ui, "Add as fill layer");
            if ui.add(b).clicked() {
                out = Some(true);
            }
            if ui.button("Back").clicked() {
                out = Some(false);
            }
        });
        out
    }

    /// Import the maps as resources and add a fill layer that uses them.
    fn add_layer(app: &mut TextureCraftApp, set: &MapSet) -> (String, bool) {
        let mut sources = Vec::new();
        for m in &set.maps {
            let Some(c) = m.channel else { continue };
            let id = CHANNELS[c].0;
            let file = m.path.file_name().and_then(|n| n.to_str()).unwrap_or("map.png").to_string();
            let bytes = match std::fs::read(&m.path) {
                Ok(b) => b,
                Err(e) => return (format!("Cannot read {file}: {e}"), true),
            };
            let bytes = if m.gloss {
                transform(&bytes, |px| px.iter_mut().take(3).for_each(|v| *v = 1.0 - *v))
            } else if id == "normal" && set.normal_dx {
                transform(&bytes, |px| px[1] = 1.0 - px[1])
            } else {
                Ok(bytes)
            };
            let bytes = match bytes {
                Ok(b) => b,
                Err(e) => return (format!("{file}: {e}"), true),
            };
            match app.session.import_resource_bytes(&file, bytes) {
                Ok(name) => sources.push((id, name)),
                Err(e) => return (format!("{file}: {e}"), true),
            }
        }
        if sources.is_empty() {
            return ("Choose a channel for at least one map.".into(), true);
        }
        let Some(layer) = app.run("layer.add.fill", json!({ "name": set.name })).and_then(|v| v["layer"].as_u64()) else {
            return ("Could not add the fill layer.".into(), true);
        };
        for (channel, image) in &sources {
            app.run("layer.fill.set", json!({ "layer": layer, "channel": channel, "image": image }));
        }
        let msg = format!("Added \"{}\" with {} map(s).", set.name, sources.len());
        app.toast(msg.clone(), false);
        (msg, false)
    }

    /// Re-encode an image after changing its pixels (0..1 RGBA), keeping 16-bit precision.
    fn transform(bytes: &[u8], f: impl Fn(&mut [f32])) -> Result<Vec<u8>, String> {
        let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
        let out = if img.color().bytes_per_pixel() > img.color().channel_count() {
            let mut b = img.to_rgba16();
            for p in b.pixels_mut() {
                let mut v = p.0.map(|c| f32::from(c) / 65535.0);
                f(&mut v);
                p.0 = v.map(|c| (c.clamp(0.0, 1.0) * 65535.0).round() as u16);
            }
            image::DynamicImage::ImageRgba16(b)
        } else {
            let mut b = img.to_rgba8();
            for p in b.pixels_mut() {
                let mut v = p.0.map(|c| f32::from(c) / 255.0);
                f(&mut v);
                p.0 = v.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8);
            }
            image::DynamicImage::ImageRgba8(b)
        };
        let mut buf = std::io::Cursor::new(Vec::new());
        out.write_to(&mut buf, image::ImageFormat::Png).map_err(|e| e.to_string())?;
        Ok(buf.into_inner())
    }

    // ---------------------------------------------------------------------------------------
    // Files

    fn work_dir(kind: &str, name: &str) -> PathBuf {
        let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
        let safe: String = name.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).take(40).collect();
        let dir = base.join("SaltyShade3D").join(kind).join(safe);
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    fn unzip(path: &Path) -> Result<PathBuf, String> {
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("zip");
        let dir = work_dir("zips", name);
        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
            let Some(rel) = entry.enclosed_name() else { continue };
            let is_image = rel.extension().and_then(|e| e.to_str()).is_some_and(|e| IMAGE_EXTS.contains(&e.to_ascii_lowercase().as_str()));
            if entry.is_dir() || !is_image {
                continue;
            }
            let out = dir.join(rel);
            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut f = std::fs::File::create(&out).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut f).map_err(|e| e.to_string())?;
        }
        Ok(dir)
    }

    fn quiet(cmd: &mut Command) -> &mut Command {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x0800_0000);
        }
        cmd
    }

    // ---------------------------------------------------------------------------------------
    // Substance .sbsar

    /// Unpack the archive (7-Zip; Windows' own tar reads it) and read its descriptor.
    fn open_sbsar(ctx: &egui::Context, file: &Path) -> Result<Sbsar, String> {
        let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("substance").to_string();
        let dir = work_dir("sbsar", &stem);
        let tar = std::env::var_os("SystemRoot").map(|r| PathBuf::from(r).join("System32").join("tar.exe")).filter(|p| p.exists()).unwrap_or_else(|| "tar".into());
        let ok = quiet(Command::new(tar).arg("-xf").arg(file).arg("-C").arg(&dir)).status().map(|s| s.success()).unwrap_or(false);
        if !ok {
            return Err("could not unpack the .sbsar".into());
        }
        let mut xmls = Vec::new();
        find_files(&dir, "xml", &mut xmls, 4);
        let xml = xmls.first().ok_or("the .sbsar has no descriptor")?;
        let text = std::fs::read_to_string(xml).map_err(|e| e.to_string())?;
        let mut sb = parse_descriptor(&text);
        sb.file = file.to_path_buf();
        if sb.label.is_empty() {
            sb.label = stem;
        }
        let mut pngs = Vec::new();
        find_files(&dir, "png", &mut pngs, 4);
        if let Some(t) = pngs.iter().find(|p| p.file_name().is_some_and(|n| n == "thumbnail.png")).or(pngs.first()) {
            if let Ok(img) = image::open(t) {
                let img = img.thumbnail(256, 256).to_rgba8();
                let ci = egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw());
                sb.thumb = Some(ctx.load_texture("saltyshade-sbsar-thumb", ci, egui::TextureOptions::LINEAR));
            }
        }
        sb.engine = find_sbsrender();
        Ok(sb)
    }

    fn find_files(dir: &Path, ext: &str, out: &mut Vec<PathBuf>, depth: u32) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() && depth > 0 {
                find_files(&p, ext, out, depth - 1);
            } else if p.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case(ext)) {
                out.push(p);
            }
        }
    }

    fn unescape(s: &str) -> String {
        s.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'").replace("&amp;", "&")
    }

    /// The value of `name="..."` in one tag.
    fn attr(tag: &str, name: &str) -> String {
        let key = format!(" {name}=\"");
        tag.find(&key).and_then(|i| tag[i + key.len()..].split('"').next()).map(unescape).unwrap_or_default()
    }

    /// Every `<tag ...>...</tag>` (or self-closed) element of `xml`, whole.
    fn elements<'a>(xml: &'a str, tag: &str) -> Vec<&'a str> {
        let open = format!("<{tag} ");
        let close = format!("</{tag}>");
        let mut out = Vec::new();
        let mut rest = xml;
        while let Some(i) = rest.find(&open) {
            let from = &rest[i..];
            let head_end = from.find('>').unwrap_or(from.len());
            let end = if from[..head_end].ends_with('/') { head_end + 1 } else { from.find(&close).map(|j| j + close.len()).unwrap_or(head_end + 1) };
            out.push(&from[..end]);
            rest = &from[end.min(from.len())..];
        }
        out
    }

    fn numbers(s: &str) -> Vec<f32> {
        s.split(',').filter_map(|v| v.trim().parse().ok()).collect()
    }

    fn parse_descriptor(xml: &str) -> Sbsar {
        let graph = elements(xml, "graph").into_iter().next().unwrap_or("");
        let head = &graph[..graph.find('>').unwrap_or(graph.len())];
        let outputs = elements(graph, "output").iter().map(|o| attr(o, "identifier")).collect();
        let mut inputs = Vec::new();
        for el in elements(graph, "input") {
            let id = attr(el, "identifier");
            let gui = elements(el, "inputgui").into_iter().next();
            // Engine inputs ($outputsize, $randomseed) have their own controls.
            let Some(gui) = gui.filter(|_| !id.starts_with('$')) else { continue };
            let ty: u32 = attr(el, "type").parse().unwrap_or(0);
            // Image, text and font inputs (types 5-7) can't be set from here; the graph keeps its own.
            if (5..=7).contains(&ty) {
                continue;
            }
            let default = numbers(&attr(el, "default"));
            let slider = elements(gui, "guislider").into_iter().next().unwrap_or("");
            let items = elements(gui, "guicomboboxitem").iter().map(|i| (attr(i, "value").parse().unwrap_or(0), attr(i, "text"))).collect();
            inputs.push(Input {
                label: { let l = attr(gui, "label"); if l.is_empty() { id.clone() } else { l } },
                group: attr(gui, "group"),
                widget: attr(gui, "widget"),
                min: attr(slider, "min").parse().unwrap_or(0.0),
                max: attr(slider, "max").parse().unwrap_or(1.0),
                value: default.clone(),
                default,
                items,
                ty,
                id,
            });
        }
        let presets = elements(graph, "sbspreset")
            .iter()
            .map(|p| (attr(p, "label"), elements(p, "presetinput").iter().map(|i| (attr(i, "identifier"), attr(i, "value"))).collect()))
            .collect();
        Sbsar {
            file: PathBuf::new(),
            label: attr(head, "label"),
            author: attr(head, "author"),
            author_url: attr(head, "authorurl"),
            thumb: None,
            outputs,
            inputs,
            presets,
            preset: None,
            size: 2,
            seed: 0,
            normal_dx: true,
            engine: None,
        }
    }

    /// Adobe's sbsrender: SALTYSHADE_SBSRENDER, the path chosen with Locate, PATH, or the usual
    /// install folders of Substance 3D Designer and the Automation Toolkit.
    fn find_sbsrender() -> Option<PathBuf> {
        let mut c: Vec<PathBuf> = Vec::new();
        if let Some(p) = std::env::var_os("SALTYSHADE_SBSRENDER") {
            c.push(p.into());
        }
        if let Ok(p) = std::fs::read_to_string(engine_setting()) {
            c.push(p.trim().into());
        }
        if let Some(path) = std::env::var_os("PATH") {
            c.extend(std::env::split_paths(&path).map(|d| d.join("sbsrender.exe")));
        }
        for root in [std::env::var_os("ProgramFiles"), std::env::var_os("ProgramW6432")].into_iter().flatten() {
            let root = PathBuf::from(root);
            for vendor in ["Adobe", "Allegorithmic"] {
                if let Ok(rd) = std::fs::read_dir(root.join(vendor)) {
                    for e in rd.flatten() {
                        if e.file_name().to_string_lossy().to_ascii_lowercase().contains("substance") {
                            c.push(e.path().join("sbsrender.exe"));
                        }
                    }
                }
            }
        }
        c.into_iter().find(|p| p.is_file())
    }

    fn engine_setting() -> PathBuf {
        let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
        base.join("SaltyShade3D").join("sbsrender.txt")
    }

    enum SbsarAction {
        Render,
        Maps,
    }

    fn sbsar_ui(ui: &mut egui::Ui, ctx: &egui::Context, tk: &Tokens, sb: &mut Sbsar, busy: bool) -> Option<SbsarAction> {
        let mut action = None;
        ui.horizontal(|ui| {
            if let Some(t) = &sb.thumb {
                ui.add(egui::Image::new((t.id(), vec2(96.0, 96.0))).corner_radius(6));
            }
            ui.vertical(|ui| {
                ui.label(RichText::new(&sb.label).size(16.0).strong().color(tk.text_strong));
                if !sb.author.is_empty() {
                    ui.label(RichText::new(format!("by {}", sb.author)).color(tk.text_dim));
                }
                if !sb.author_url.is_empty() {
                    ui.hyperlink_to("Author's page", &sb.author_url);
                }
                ui.label(RichText::new(format!("Maps: {}", sb.outputs.join(", "))).size(11.0).color(tk.text_dim));
            });
        });
        ui.separator();
        let Some(engine) = sb.engine.clone() else {
            ui.label(RichText::new("Rendering .sbsar needs Adobe's Substance engine").strong().color(tk.warning));
            ui.label("A .sbsar holds a compiled Substance graph that only Adobe's engine can run. Install Substance 3D Designer or the Substance 3D Automation Toolkit (they include sbsrender.exe) and SaltyShade 3D will find it, or use Locate.");
            ui.label(RichText::new("No Substance licence? Download the same material as texture maps (many sites offer PNG sets) and import those.").color(tk.text_dim));
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button("Locate sbsrender.exe...").clicked() {
                    if let Some(p) = rfd::FileDialog::new().add_filter("sbsrender", &["exe"]).pick_file() {
                        let setting = engine_setting();
                        let _ = std::fs::create_dir_all(setting.parent().unwrap_or(Path::new(".")));
                        let _ = std::fs::write(&setting, p.to_string_lossy().as_bytes());
                        sb.engine = Some(p);
                    }
                }
                if ui.button("Import texture maps instead...").clicked() {
                    action = Some(SbsarAction::Maps);
                }
            });
            return action;
        };
        ui.horizontal(|ui| {
            ui.label("Preset");
            let text = sb.preset.and_then(|i| sb.presets.get(i)).map(|p| p.0.clone()).unwrap_or_else(|| "Default".into());
            let mut chosen = None;
            egui::ComboBox::from_id_salt("saltyshade-preset").selected_text(text).height(320.0).show_ui(ui, |ui| {
                if ui.selectable_label(sb.preset.is_none(), "Default").clicked() {
                    chosen = Some(None);
                }
                for (i, (label, _)) in sb.presets.iter().enumerate() {
                    if ui.selectable_label(sb.preset == Some(i), label).clicked() {
                        chosen = Some(Some(i));
                    }
                }
            });
            if let Some(c) = chosen {
                apply_preset(sb, c);
            }
            ui.label("Size");
            egui::ComboBox::from_id_salt("saltyshade-size").selected_text(SIZES[sb.size].1).width(60.0).show_ui(ui, |ui| {
                for (i, (_, label)) in SIZES.iter().enumerate() {
                    ui.selectable_value(&mut sb.size, i, *label);
                }
            });
            ui.label("Seed");
            ui.add(egui::DragValue::new(&mut sb.seed).range(0..=9999));
        });
        ui.add_space(4.0);
        egui::ScrollArea::vertical().max_height(360.0).auto_shrink([false, true]).show(ui, |ui| {
            let mut groups: Vec<String> = Vec::new();
            for i in &sb.inputs {
                if !groups.contains(&i.group) {
                    groups.push(i.group.clone());
                }
            }
            for g in groups {
                let mut body = |ui: &mut egui::Ui| {
                    egui::Grid::new(("saltyshade-params", &g)).num_columns(2).spacing(vec2(10.0, 4.0)).show(ui, |ui| {
                        for input in sb.inputs.iter_mut().filter(|i| i.group == g) {
                            input_ui(ui, input);
                            ui.end_row();
                        }
                    });
                };
                if g.is_empty() {
                    body(ui);
                } else {
                    egui::CollapsingHeader::new(&g).id_salt(("saltyshade-group", &g)).show(ui, body);
                }
            }
        });
        ui.add_space(4.0);
        ui.checkbox(&mut sb.normal_dx, "Normal output is DirectX (Substance's default); flip it to OpenGL");
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let label = if busy { "Rendering..." } else { "Render and add fill layer" };
            let b = crate::widgets::primary_button(ui, label);
            if ui.add_enabled(!busy, b).clicked() {
                action = Some(SbsarAction::Render);
            }
            if busy {
                ui.spinner();
                ctx.request_repaint_after(std::time::Duration::from_millis(200));
            }
            if ui.button("Reset").clicked() {
                apply_preset(sb, None);
            }
        });
        ui.label(RichText::new(format!("Engine: {}", engine.display())).size(10.5).color(tk.text_dim));
        action
    }

    fn input_ui(ui: &mut egui::Ui, input: &mut Input) {
        // Substance value types: 0-3 float1-4, 4 int, 8-10 int2-4.
        let n = match input.ty {
            1 | 8 => 2,
            2 | 9 => 3,
            3 | 10 => 4,
            _ => 1,
        };
        input.value.resize(n, 0.0);
        let int = input.ty == 4 || input.ty >= 8;
        ui.label(&input.label).on_hover_text(&input.id);
        ui.horizontal(|ui| {
            ui.spacing_mut().slider_width = 170.0;
            match input.widget.as_str() {
                "combobox" if !input.items.is_empty() => {
                    let cur = input.value[0] as i32;
                    let text = input.items.iter().find(|(v, _)| *v == cur).map(|i| i.1.clone()).unwrap_or_else(|| cur.to_string());
                    egui::ComboBox::from_id_salt(("saltyshade-in", &input.id)).selected_text(text).show_ui(ui, |ui| {
                        for (v, t) in &input.items {
                            if ui.selectable_label(cur == *v, t).clicked() {
                                input.value[0] = *v as f32;
                            }
                        }
                    });
                }
                "togglebutton" => {
                    let mut on = input.value[0] != 0.0;
                    if ui.checkbox(&mut on, "").changed() {
                        input.value[0] = if on { 1.0 } else { 0.0 };
                    }
                }
                "color" if n >= 3 => {
                    let mut rgb = [input.value[0], input.value[1], input.value[2]];
                    if ui.color_edit_button_rgb(&mut rgb).changed() {
                        input.value[..3].copy_from_slice(&rgb);
                    }
                }
                _ => {
                    let (lo, hi) = if input.max > input.min { (input.min, input.max) } else { (0.0, 1.0) };
                    if n == 1 {
                        if int {
                            let mut v = input.value[0] as i32;
                            if ui.add(egui::Slider::new(&mut v, lo as i32..=hi as i32)).changed() {
                                input.value[0] = v as f32;
                            }
                        } else {
                            ui.add(egui::Slider::new(&mut input.value[0], lo..=hi));
                        }
                    } else {
                        for v in input.value.iter_mut() {
                            ui.add(egui::DragValue::new(v).speed((hi - lo) / 200.0).range(lo..=hi));
                        }
                    }
                }
            }
        });
    }

    fn apply_preset(sb: &mut Sbsar, preset: Option<usize>) {
        for i in &mut sb.inputs {
            i.value = i.default.clone();
        }
        sb.preset = preset;
        let Some(values) = preset.and_then(|p| sb.presets.get(p)).map(|p| p.1.clone()) else { return };
        for (id, v) in values {
            if let Some(i) = sb.inputs.iter_mut().find(|i| i.id == id) {
                let n = numbers(&v);
                if !n.is_empty() {
                    i.value = n;
                }
            }
        }
    }

    /// Run sbsrender in the background: every output as a PNG, with the chosen parameters.
    fn start_render(sb: &Sbsar) -> Receiver<Result<Vec<PathBuf>, String>> {
        let (tx, rx) = channel();
        let engine = sb.engine.clone().unwrap_or_default();
        let out = work_dir("sbsar-render", &sb.label);
        let log2 = SIZES[sb.size].0;
        let mut args: Vec<String> = vec![
            "render".into(),
            "--input".into(),
            sb.file.to_string_lossy().into_owned(),
            "--output-path".into(),
            out.to_string_lossy().into_owned(),
            "--output-name".into(),
            "{outputNodeName}".into(),
            "--output-format".into(),
            "png".into(),
            "--set-value".into(),
            format!("$outputsize@{log2},{log2}"),
            "--set-value".into(),
            format!("$randomseed@{}", sb.seed),
        ];
        for i in &sb.inputs {
            if i.value != i.default {
                let int = i.ty == 4 || i.ty >= 8;
                let v: Vec<String> = i.value.iter().map(|x| if int { format!("{}", x.round() as i64) } else { format!("{x}") }).collect();
                args.push("--set-value".into());
                args.push(format!("{}@{}", i.id, v.join(",")));
            }
        }
        std::thread::spawn(move || {
            let result = quiet(Command::new(&engine).args(&args)).output();
            let r = match result {
                Err(e) => Err(format!("Could not start sbsrender: {e}")),
                Ok(o) if !o.status.success() => {
                    let err = String::from_utf8_lossy(&o.stderr);
                    Err(format!("sbsrender failed: {}", err.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("unknown error")))
                }
                Ok(_) => {
                    let mut files = Vec::new();
                    find_files(&out, "png", &mut files, 1);
                    if files.is_empty() { Err("sbsrender produced no maps.".into()) } else { Ok(files) }
                }
            };
            let _ = tx.send(r);
        });
        rx
    }
}
