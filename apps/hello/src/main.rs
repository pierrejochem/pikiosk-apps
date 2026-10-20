//! The smallest useful pikiosk app. kiosk-kit opens the display and the touch
//! screen; the app draws one frame at a time. Exiting with status 0 hands the
//! display back to kiosk-ui.

use egui::{Align2, Id, Rect, Ui, pos2, vec2};
use kiosk_kit::glass::{BackdropParams, Glass, GlassSystem, Optics, Theme};
use kiosk_kit::widgets::{self as w, Sc, text};
use kiosk_kit::{App, Lay};

struct Hello {
    glass: GlassSystem,
}

impl App for Hello {
    fn ui(&mut self, ui: &mut Ui) {
        let screen = ui.ctx().content_rect();
        let l = Lay::new(screen);
        let time = ui.ctx().input(|i| i.time);
        let painter = ui.painter().clone();

        self.glass.optics = Optics { frost: 1.0, refraction: 1.0 };
        self.glass.backdrop(&painter, BackdropParams { time: time as f32, theme: Theme::Ocean, animate: true });

        let sc = Sc { gs: &self.glass.clone(), u: l.wu, alpha: 1.0, enabled: true };
        let card = Rect::from_center_size(screen.center(), vec2(l.v(520.0, 420.0), l.v(280.0, 220.0)));
        sc.glass(&painter, card, Glass::panel(l.v(34.0, 26.0)));
        text(&painter, pos2(card.center().x, card.top() + l.v(80.0, 62.0)), Align2::CENTER_CENTER, "Hello", l.v(52.0, 38.0), sc.white(1.0));
        text(&painter, pos2(card.center().x, card.top() + l.v(138.0, 104.0)), Align2::CENTER_CENTER, "an app from the pikiosk store", l.v(20.0, 15.0), sc.white(0.75));

        let button = Rect::from_center_size(pos2(card.center().x, card.bottom() - l.v(52.0, 40.0)), vec2(l.v(200.0, 160.0), l.v(52.0, 42.0)));
        if w::button(ui, sc, button, Id::new("back"), "Back", Some(w::ACCENT)).clicked() {
            std::process::exit(0);
        }
        ui.ctx().request_repaint();
    }
}

fn main() -> anyhow::Result<()> {
    kiosk_kit::run(Hello { glass: GlassSystem::default() })
}
