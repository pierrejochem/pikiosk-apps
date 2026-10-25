//! Weather icons drawn with egui shapes, so they scale with the layout.

use egui::{Color32, Painter, Pos2, Shape, Stroke, pos2, vec2};

use crate::wmo::IconKind;

const SUN: Color32 = Color32::from_rgb(250, 170, 20);
const SUN_RAY: Color32 = Color32::from_rgb(255, 200, 70);
const CLOUD: Color32 = Color32::from_rgb(190, 196, 206);
const CLOUD_DARK: Color32 = Color32::from_rgb(150, 156, 168);
const DROP: Color32 = Color32::from_rgb(20, 110, 235);
const SNOW: Color32 = Color32::from_rgb(235, 240, 255);
const BOLT: Color32 = Color32::from_rgb(255, 210, 40);

/// Draws `kind` centred on `c`, fitting a square of side `size`.
pub fn draw(p: &Painter, kind: IconKind, c: Pos2, size: f32, alpha: f32) {
    let a = |col: Color32| col.gamma_multiply(alpha);
    let s = size;
    match kind {
        IconKind::Sun => sun(p, c, s * 0.5, alpha),
        IconKind::PartlyCloudy => {
            sun(p, c + vec2(-s * 0.14, -s * 0.14), s * 0.34, alpha);
            cloud(p, c + vec2(s * 0.06, s * 0.1), s * 0.8, a(CLOUD));
        }
        IconKind::Cloud => cloud(p, c, s * 0.9, a(CLOUD)),
        IconKind::Fog => {
            cloud(p, c + vec2(0.0, -s * 0.1), s * 0.8, a(CLOUD));
            for i in 0..2 {
                let y = c.y + s * (0.28 + 0.12 * i as f32);
                p.line_segment([pos2(c.x - s * 0.3, y), pos2(c.x + s * 0.3, y)], Stroke::new(s * 0.05, a(CLOUD_DARK)));
            }
        }
        IconKind::Rain => {
            cloud(p, c + vec2(0.0, -s * 0.1), s * 0.8, a(CLOUD_DARK));
            for dx in [-0.2f32, 0.0, 0.2] {
                drop(p, c + vec2(s * dx, s * 0.32), s * 0.07, a(DROP));
            }
        }
        IconKind::Snow => {
            cloud(p, c + vec2(0.0, -s * 0.1), s * 0.8, a(CLOUD_DARK));
            for dx in [-0.2f32, 0.0, 0.2] {
                p.circle_filled(c + vec2(s * dx, s * 0.32), s * 0.05, a(SNOW));
            }
        }
        IconKind::Thunder => {
            cloud(p, c + vec2(0.0, -s * 0.1), s * 0.8, a(CLOUD_DARK));
            let t = c + vec2(0.0, s * 0.3);
            let bolt = [t + vec2(s * 0.04, -s * 0.16), t + vec2(-s * 0.08, s * 0.02), t + vec2(0.0, s * 0.02), t + vec2(-s * 0.04, s * 0.18), t + vec2(s * 0.1, -s * 0.04), t + vec2(s * 0.02, -s * 0.04)];
            p.add(Shape::convex_polygon(bolt.to_vec(), a(BOLT), Stroke::NONE));
        }
    }
}

fn sun(p: &Painter, c: Pos2, r: f32, alpha: f32) {
    let core = r * 0.62;
    for i in 0..12 {
        let ang = i as f32 * std::f32::consts::TAU / 12.0;
        let d = vec2(ang.cos(), ang.sin());
        p.line_segment([c + d * (core * 1.2), c + d * r], Stroke::new(r * 0.12, SUN_RAY.gamma_multiply(alpha)));
    }
    p.circle_filled(c, core, SUN.gamma_multiply(alpha));
}

/// A flat-bottomed cloud made of three discs and a base bar.
fn cloud(p: &Painter, c: Pos2, w: f32, col: Color32) {
    let r = w * 0.22;
    let base = c.y + w * 0.14;
    p.circle_filled(pos2(c.x - w * 0.22, base - r * 0.55), r * 0.8, col);
    p.circle_filled(pos2(c.x + w * 0.02, base - r * 1.05), r * 1.1, col);
    p.circle_filled(pos2(c.x + w * 0.24, base - r * 0.5), r * 0.85, col);
    p.rect_filled(egui::Rect::from_min_max(pos2(c.x - w * 0.34, base - r * 0.7), pos2(c.x + w * 0.34, base + r * 0.25)), r * 0.55, col);
}

fn drop(p: &Painter, c: Pos2, r: f32, col: Color32) {
    p.circle_filled(c + vec2(0.0, r * 0.4), r, col);
    p.add(Shape::convex_polygon(vec![c + vec2(0.0, -r * 1.6), c + vec2(-r * 0.9, r * 0.1), c + vec2(r * 0.9, r * 0.1)], col, Stroke::NONE));
}
