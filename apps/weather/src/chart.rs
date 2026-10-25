//! The three chart tabs. All draw into `rect`, one column per point, so the
//! hour labels line up with the day row below.

use egui::{Align2, Color32, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};
use kiosk_kit::widgets::{ACCENT, text};

use crate::model::Point;

const TEMP: Color32 = Color32::from_rgb(255, 196, 40);
const LABEL: f32 = 15.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Temperature,
    Precipitation,
    Wind,
}

impl Tab {
    pub const ALL: [Tab; 3] = [Tab::Temperature, Tab::Precipitation, Tab::Wind];

    pub fn label(self) -> &'static str {
        match self {
            Tab::Temperature => "Temperatur",
            Tab::Precipitation => "Niederschlag",
            Tab::Wind => "Wind",
        }
    }
}

/// The x centre of column `i` of `n` across `rect`.
fn col_x(rect: Rect, i: usize, n: usize) -> f32 {
    rect.left() + rect.width() * (i as f32 + 0.5) / n as f32
}

/// `rect` is the chart; hour labels go on a strip at its bottom.
pub fn draw(p: &Painter, tab: Tab, rect: Rect, pts: &[Point], u: f32, alpha: f32) {
    if pts.is_empty() {
        return;
    }
    let label_h = 24.0 * u;
    let plot = Rect::from_min_max(rect.min, pos2(rect.right(), rect.bottom() - label_h));
    let n = pts.len();
    for (i, pt) in pts.iter().enumerate() {
        text(p, pos2(col_x(rect, i, n), rect.bottom() - label_h * 0.4), Align2::CENTER_CENTER, &format!("{:02}:00", pt.at.h), LABEL * u, Color32::from_white_alpha((150.0 * alpha) as u8));
    }
    match tab {
        Tab::Temperature => temperature(p, plot, pts, u, alpha),
        Tab::Precipitation => precipitation(p, plot, pts, u, alpha),
        Tab::Wind => wind(p, plot, pts, u, alpha),
    }
}

fn temperature(p: &Painter, plot: Rect, pts: &[Point], u: f32, alpha: f32) {
    let n = pts.len();
    let (lo, hi) = pts.iter().fold((f32::MAX, f32::MIN), |(l, h), q| (l.min(q.temp), h.max(q.temp)));
    let span = (hi - lo).max(4.0);
    let top = plot.top() + 26.0 * u;
    let line_y = |t: f32| plot.bottom() - 6.0 * u - (t - lo) / span * (plot.bottom() - top - 6.0 * u);
    let xy: Vec<Pos2> = pts.iter().enumerate().map(|(i, q)| pos2(col_x(plot, i, n), line_y(q.temp))).collect();

    // Area under the line, one quad per segment so it stays convex.
    let fill = TEMP.gamma_multiply(0.22 * alpha);
    for w in xy.windows(2) {
        let quad = vec![w[0], w[1], pos2(w[1].x, plot.bottom()), pos2(w[0].x, plot.bottom())];
        p.add(Shape::convex_polygon(quad, fill, Stroke::NONE));
    }
    p.add(Shape::line(xy.clone(), Stroke::new(2.5 * u, TEMP.gamma_multiply(alpha))));
    for (i, (pt, q)) in pts.iter().zip(&xy).enumerate() {
        let bold = i == 0;
        let col = if bold { Color32::from_white_alpha((255.0 * alpha) as u8) } else { Color32::from_white_alpha((170.0 * alpha) as u8) };
        text(p, *q - vec2(0.0, 14.0 * u), Align2::CENTER_CENTER, &format!("{:.0}", pt.temp), LABEL * u, col);
    }
}

fn precipitation(p: &Painter, plot: Rect, pts: &[Point], u: f32, alpha: f32) {
    let n = pts.len();
    let top = plot.top() + 24.0 * u;
    let h = plot.bottom() - top;
    let step = plot.width() / n as f32;
    for (i, pt) in pts.iter().enumerate() {
        let x = col_x(plot, i, n);
        let bar_h = (pt.precip / 100.0).clamp(0.0, 1.0) * h;
        let bar = Rect::from_min_max(pos2(x - step * 0.5, plot.bottom() - bar_h.max(2.0 * u)), pos2(x + step * 0.5, plot.bottom()));
        p.rect_filled(bar, 0.0, ACCENT.gamma_multiply(0.30 * alpha));
        p.line_segment([bar.left_top(), bar.right_top()], Stroke::new(2.0 * u, ACCENT.gamma_multiply(alpha)));
        text(p, pos2(x, top - 10.0 * u), Align2::CENTER_CENTER, &format!("{:.0}%", pt.precip), LABEL * u, Color32::from_rgb(120, 180, 255).gamma_multiply(alpha));
    }
}

fn wind(p: &Painter, plot: Rect, pts: &[Point], u: f32, alpha: f32) {
    let n = pts.len();
    let arrow_c = plot.center().y + 10.0 * u;
    for (i, pt) in pts.iter().enumerate() {
        let x = col_x(plot, i, n);
        text(p, pos2(x, plot.top() + 12.0 * u), Align2::CENTER_CENTER, &format!("{:.0} km/h", pt.wind), LABEL * u, Color32::from_white_alpha((200.0 * alpha) as u8));
        arrow(p, pos2(x, arrow_c), pt.wind_dir, 26.0 * u, Color32::from_rgb(160, 180, 210).gamma_multiply(alpha), u);
    }
}

/// An arrow pointing where the wind blows to: `from_deg` is where it comes from.
fn arrow(p: &Painter, c: Pos2, from_deg: f32, len: f32, col: Color32, u: f32) {
    let to = (from_deg + 180.0).to_radians();
    let dir = vec2(to.sin(), -to.cos());
    let side = vec2(-dir.y, dir.x);
    let tail = c - dir * len * 0.5;
    let tip = c + dir * len * 0.5;
    p.line_segment([tail, tip - dir * len * 0.18], Stroke::new(3.0 * u, col));
    let head = vec![tip, tip - dir * len * 0.34 + side * len * 0.2, tip - dir * len * 0.34 - side * len * 0.2];
    p.add(Shape::convex_polygon(head, col, Stroke::NONE));
}
