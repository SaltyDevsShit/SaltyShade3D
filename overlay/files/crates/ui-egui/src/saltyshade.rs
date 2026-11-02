//! SaltyShade 3D: the Neon Cyber-Studio look, laid over TextureCraft's own theme.
//!
//! This file is part of the SaltyShade overlay (`overlay/files/`), copied into the upstream tree
//! on every update by `tools/apply_overlay.py`; edit it there, never inside `upstream/`.
//! [`apply`] runs right after `theme::apply`, so whatever Brightness or Accent is chosen in
//! Settings, the interface is deep obsidian with frosted-glass floating panels and electric violet
//! and cyan glow for active tools, masks and viewport gizmos.

use egui::{Color32, CornerRadius, Painter, Pos2, Rect, Stroke, StrokeKind};

use crate::theme::Tokens;

const fn hex(s: u32) -> Color32 {
    Color32::from_rgb((s >> 16) as u8, (s >> 8) as u8, s as u8)
}

/// `rgb` at `alpha` (0-255), premultiplied as egui stores it.
const fn glass(s: u32, alpha: u8) -> Color32 {
    Color32::from_rgba_premultiplied(premul(s >> 16, alpha), premul(s >> 8, alpha), premul(s, alpha), alpha)
}

const fn premul(c: u32, alpha: u8) -> u8 {
    ((c & 0xff) * alpha as u32 / 255) as u8
}

/// Deep obsidian: the window frame and everything behind the glass.
pub const OBSIDIAN: Color32 = hex(0x0b0914);
/// Electric violet: the accent, active wells, selections.
pub const VIOLET: Color32 = hex(0x9d4edd);
/// Cyan: the glow on what is live right now (active tool, shown mask, focused field).
pub const CYAN: Color32 = hex(0x00f5d4);
/// A deeper violet for filled buttons (white lettering reads on it).
pub const VIOLET_DEEP: Color32 = hex(0x7b2cbf);

/// The 3D axis gizmo, neon instead of plain RGB (still three distinct hues).
pub const AXIS_X: Color32 = hex(0xff4d9d);
pub const AXIS_Y: Color32 = CYAN;
pub const AXIS_Z: Color32 = hex(0xb57bff);

fn mix(a: Color32, b: Color32, k: f32) -> Color32 {
    let m = |p: u8, q: u8| (f32::from(p) + (f32::from(q) - f32::from(p)) * k).round() as u8;
    Color32::from_rgb(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()))
}

/// The Neon Cyber-Studio tokens. Fields not named here keep TextureCraft's values, so a token
/// added upstream still has a sensible colour (today every field is named, hence the allow).
#[allow(unused_variables)]
pub fn tokens(base: Tokens) -> Tokens {
    // Dock cards: the violet-smoked glass as it reads over the obsidian frame (opaque, so docked
    // panels never show the OS window behind them; the floating ones below are see-through).
    let card = hex(0x15112a);
    Tokens {
        dark: true,
        chrome: OBSIDIAN,
        panel: card,
        row: hex(0x110d22),
        row_hover: hex(0x261d45),
        row_selected: mix(card, VIOLET, 0.30),
        separator: glass(0x9d4edd, 0x40),
        text: hex(0xebe7f7),
        text_dim: hex(0xa79fc4),
        text_strong: hex(0xffffff),
        accent: VIOLET,
        accent_strong: VIOLET_DEEP,
        accent_dim: mix(card, VIOLET, 0.45),
        field: hex(0x07060e),
        field_border: hex(0x3d2f63),
        button: hex(0x1e1736),
        button_border: hex(0x5a4a86),
        tool_active: mix(card, VIOLET, 0.38),
        danger: hex(0xff4d6d),
        warning: hex(0xffbe0b),
        viewport: hex(0x110d1d),
        viewport_text: hex(0xbdb3dd),
        viewport_accent: CYAN,
        radius: 7,
        ..base
    }
}

/// Lay the Neon Cyber-Studio look over the style `theme::apply` just installed.
pub fn apply(ctx: &egui::Context) {
    let t = tokens(Tokens::get(ctx));
    ctx.data_mut(|d| d.insert_temp(egui::Id::NULL, t));
    ctx.all_styles_mut(|s| {
        let v = &mut s.visuals;
        v.dark_mode = true;
        v.override_text_color = None;
        v.panel_fill = t.panel;
        // Floating windows, menus and popups are frosted glass: the scene shows through.
        v.window_fill = glass(0x140f28, 0xd2);
        v.extreme_bg_color = t.field;
        v.text_edit_bg_color = Some(t.field);
        v.faint_bg_color = t.row;
        v.code_bg_color = t.field;
        v.window_stroke = Stroke::new(1.0, glass(0x9d4edd, 0x8c));
        v.window_corner_radius = CornerRadius::same(12);
        v.menu_corner_radius = CornerRadius::same(10);
        // Their shadow is a violet halo instead of a drop shadow.
        v.window_shadow = egui::epaint::Shadow { offset: [0, 0], blur: 30, spread: 1, color: glass(0x9d4edd, 0x50) };
        v.popup_shadow = egui::epaint::Shadow { offset: [0, 0], blur: 18, spread: 0, color: glass(0x00f5d4, 0x30) };
        v.selection.bg_fill = t.accent_dim;
        v.selection.stroke = Stroke::new(1.0, t.text_strong);
        v.hyperlink_color = CYAN;
        v.text_cursor.stroke = Stroke::new(2.0, CYAN);
        let w = &mut v.widgets;
        w.noninteractive.bg_fill = t.panel;
        w.noninteractive.bg_stroke = Stroke::new(1.0, t.separator);
        w.noninteractive.fg_stroke = Stroke::new(1.0, t.text);
        w.inactive.bg_fill = t.button;
        w.inactive.weak_bg_fill = t.button;
        w.inactive.bg_stroke = Stroke::new(1.0, glass(0x9d4edd, 0x30));
        w.inactive.fg_stroke = Stroke::new(1.0, t.text);
        w.hovered.bg_fill = t.row_hover;
        w.hovered.weak_bg_fill = t.row_hover;
        w.hovered.bg_stroke = Stroke::new(1.0, glass(0x00f5d4, 0x99));
        w.hovered.fg_stroke = Stroke::new(1.5, t.text_strong);
        // Pressed / active: the violet well with a cyan rim.
        w.active.bg_fill = t.tool_active;
        w.active.weak_bg_fill = t.tool_active;
        w.active.bg_stroke = Stroke::new(1.5, CYAN);
        w.active.fg_stroke = Stroke::new(1.5, t.text_strong);
        w.open.bg_fill = t.row_hover;
        w.open.weak_bg_fill = t.row_hover;
        w.open.bg_stroke = Stroke::new(1.0, VIOLET);
        for wv in [&mut w.noninteractive, &mut w.inactive, &mut w.hovered, &mut w.active, &mut w.open] {
            wv.corner_radius = t.cr();
        }
    });
}

/// A soft neon halo around `rect`: a few widening, fading outlines.
pub fn glow_rect(p: &Painter, rect: Rect, radius: f32, color: Color32) {
    for (grow, alpha) in [(1.0, 0.55), (2.5, 0.28), (4.5, 0.12)] {
        p.rect_stroke(rect.expand(grow), radius + grow, Stroke::new(1.5, color.gamma_multiply(alpha)), StrokeKind::Outside);
    }
}

/// A line with a neon bloom under it (the viewport gizmo's axes).
pub fn glow_line(p: &Painter, pts: [Pos2; 2], color: Color32) {
    p.line_segment(pts, Stroke::new(6.0, color.gamma_multiply(0.15)));
    p.line_segment(pts, Stroke::new(3.5, color.gamma_multiply(0.3)));
    p.line_segment(pts, Stroke::new(1.75, color));
}

/// The SaltyShade 3D logo, `size` points square, centred in the current layout (About window).
pub fn logo(ui: &mut egui::Ui, size: f32) {
    const PNG: &[u8] = include_bytes!("saltyshade_logo.png");
    let id = egui::Id::unique("saltyshade-logo");
    let tex = ui.ctx().data(|d| d.get_temp::<egui::TextureHandle>(id)).or_else(|| {
        let img = image::load_from_memory(PNG).ok()?.to_rgba8();
        let ci = egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw());
        let tex = ui.ctx().load_texture("saltyshade-logo", ci, egui::TextureOptions::LINEAR);
        ui.ctx().data_mut(|d| d.insert_temp(id, tex.clone()));
        Some(tex)
    });
    if let Some(tex) = tex {
        ui.add(egui::Image::new((tex.id(), egui::vec2(size, size))));
    }
}
