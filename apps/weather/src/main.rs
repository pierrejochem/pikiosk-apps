//! A weather card for the pikiosk display: the current conditions, a
//! three-hourly chart (temperature, precipitation or wind) and an 8-day row.
//! Data comes from Open-Meteo; the location is `$KIOSK_DATA/config.toml`.
//! Exiting with status 0 hands the display back to kiosk-ui.

mod api;
mod chart;
mod config;
mod fetch;
mod icons;
mod model;
mod wmo;

use std::os::fd::RawFd;
use std::time::{SystemTime, UNIX_EPOCH};

use egui::{Align2, Color32, Id, Rect, Sense, Ui, pos2, vec2};
use kiosk_kit::glass::{BackdropParams, Glass, GlassSystem, Optics, Theme};
use kiosk_kit::widgets::{self as w, Sc, text, text_width};
use kiosk_kit::App;

use chart::Tab;
use fetch::Fetcher;
use model::{Forecast, WEEKDAYS_LONG, WEEKDAYS_SHORT, local_from_unix};

/// The layout is drawn for 800x480 and scaled to fit.
const DESIGN: egui::Vec2 = vec2(800.0, 480.0);

struct Weather {
    glass: GlassSystem,
    config: config::Config,
    note: Option<String>,
    fetcher: Fetcher,
    forecast: Option<Forecast>,
    error: Option<String>,
    tab: usize,
    day: usize,
}

impl Weather {
    fn new() -> Self {
        let (config, note) = config::load(&config::data_dir());
        let fetcher = fetch::spawn(config.clone());
        Weather { glass: GlassSystem::default(), config, note, fetcher, forecast: None, error: None, tab: 0, day: 0 }
    }

    /// Takes what the network thread has delivered since the last frame.
    fn drain(&mut self) {
        let mut buf = [0u8; 8];
        // SAFETY: reads at most 8 bytes into a live buffer from our own nonblocking eventfd.
        unsafe { libc::read(self.fetcher.wake, buf.as_mut_ptr().cast(), buf.len()) };
        while let Ok(result) = self.fetcher.rx.try_recv() {
            match result {
                Ok(f) => {
                    self.day = self.day.min(f.days.len().saturating_sub(1));
                    self.forecast = Some(f);
                    self.error = None;
                }
                Err(e) => self.error = Some(e),
            }
        }
    }
}

fn unix_now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64)
}

impl App for Weather {
    fn ui(&mut self, ui: &mut Ui) {
        self.drain();
        let screen = ui.ctx().content_rect();
        let time = ui.ctx().input(|i| i.time);
        let painter = ui.painter().clone();

        self.glass.optics = Optics { frost: 1.0, refraction: 1.0 };
        self.glass.backdrop(&painter, BackdropParams { time: time as f32, theme: Theme::Ocean, animate: true });

        // Design space 800x480, scaled to fit and centred.
        let u = (screen.width() / DESIGN.x).min(screen.height() / DESIGN.y);
        let origin = screen.center() - DESIGN * u * 0.5;
        let at = |x: f32, y: f32| pos2(origin.x + x * u, origin.y + y * u);
        let area = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(at(x0, y0), at(x1, y1));

        let gs = self.glass.clone();
        let sc = Sc { gs: &gs, u: u * 0.86, alpha: 1.0, enabled: true };
        sc.glass(&painter, area(12.0, 12.0, 788.0, 468.0), Glass::panel(30.0 * u));

        // Top row: back and the location.
        if w::button(ui, sc, area(28.0, 24.0, 108.0, 58.0), Id::unique("back"), "Zurück", None).clicked() {
            std::process::exit(0);
        }
        let name_w = text_width(&painter, &self.config.name, 17.0 * u) / u + 40.0;
        let name_box = area(120.0, 24.0, 120.0 + name_w.min(520.0), 58.0);
        sc.glass(&painter, name_box, Glass::control(name_box.height() * 0.5).tinted(Color32::WHITE, 0.08).no_shadow());
        text(&painter, name_box.center(), Align2::CENTER_CENTER, &self.config.name, 17.0 * u, sc.white(1.0));

        let Some(f) = &self.forecast else {
            let msg = match &self.error {
                Some(e) => format!("Keine Verbindung: {e}"),
                None => "Wetter wird geladen …".to_owned(),
            };
            text(&painter, at(400.0, 250.0), Align2::CENTER_CENTER, &msg, 20.0 * u, sc.white(0.8));
            return repaint(ui);
        };

        let (now, minute) = local_from_unix(unix_now(), f.utc_offset);
        let (cond, kind) = wmo::describe(f.code);

        // Current conditions.
        icons::draw(&painter, kind, at(72.0, 120.0), 84.0 * u, 1.0);
        text(&painter, at(128.0, 116.0), Align2::LEFT_CENTER, &format!("{:.0}", f.temp), 70.0 * u, sc.white(1.0));
        let tw = text_width(&painter, &format!("{:.0}", f.temp), 70.0 * u) / u;
        text(&painter, at(128.0 + tw + 6.0, 102.0), Align2::LEFT_CENTER, "°C", 22.0 * u, sc.white(0.8));
        for (i, line) in [format!("Niederschlag: {:.0}%", f.precip_now(now)), format!("Luftfeuchte: {:.0}%", f.humidity), format!("Wind: {:.0} km/h", f.wind)].iter().enumerate() {
            text(&painter, at(290.0, 94.0 + 22.0 * i as f32), Align2::LEFT_CENTER, line, 15.0 * u, sc.white(0.72));
        }
        text(&painter, at(772.0, 84.0), Align2::RIGHT_CENTER, "Wetter", 26.0 * u, sc.white(1.0));
        text(&painter, at(772.0, 112.0), Align2::RIGHT_CENTER, &format!("{}, {:02}:{:02}", WEEKDAYS_LONG[now.weekday()], now.h, minute), 17.0 * u, sc.white(0.72));
        text(&painter, at(772.0, 136.0), Align2::RIGHT_CENTER, cond, 17.0 * u, sc.white(0.72));

        // Tabs and status.
        let labels = Tab::ALL.map(Tab::label);
        w::segmented(ui, sc, area(28.0, 160.0, 388.0, 194.0), Id::unique("tabs"), &labels, &mut self.tab);
        let status = self.error.as_ref().map(|_| "offline").or(self.note.as_deref());
        if let Some(s) = status {
            w::pill(ui, sc, at(772.0, 177.0), s, Color32::from_rgb(200, 120, 20));
        }

        // Chart for the selected day.
        let pts = f.chart_points(self.day, now);
        chart::draw(&painter, Tab::ALL[self.tab], area(28.0, 204.0, 772.0, 334.0), &pts, u, 1.0);

        // Day row.
        let cell_w = (772.0 - 28.0) / 8.0;
        for (i, d) in f.days.iter().take(8).enumerate() {
            let x0 = 28.0 + cell_w * i as f32;
            let cell = area(x0 + 2.0, 342.0, x0 + cell_w - 2.0, 458.0);
            let hit = ui.interact(cell, Id::unique(("day", i)), Sense::click());
            if hit.clicked() {
                self.day = i;
            }
            if i == self.day {
                sc.glass(&painter, cell, Glass::control(16.0 * u).tinted(Color32::WHITE, 0.18).no_shadow());
            }
            let cx = x0 + cell_w * 0.5;
            text(&painter, at(cx, 366.0), Align2::CENTER_CENTER, WEEKDAYS_SHORT[d.date.weekday()], 19.0 * u, sc.white(1.0));
            icons::draw(&painter, d.icon(), at(cx, 405.0), 46.0 * u, 1.0);
            text(&painter, at(cx - 14.0, 440.0), Align2::CENTER_CENTER, &format!("{:.0}°", d.max), 15.0 * u, sc.white(1.0));
            text(&painter, at(cx + 16.0, 440.0), Align2::CENTER_CENTER, &format!("{:.0}°", d.min), 15.0 * u, sc.white(0.6));
        }
        repaint(ui)
    }

    #[cfg(unix)]
    fn wake_fd(&self) -> Option<RawFd> {
        Some(self.fetcher.wake)
    }
}

fn repaint(ui: &Ui) {
    ui.ctx().request_repaint();
}

fn main() -> anyhow::Result<()> {
    kiosk_kit::run(Weather::new())
}
