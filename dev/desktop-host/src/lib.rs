//! Runs any `kiosk_kit::App` in an 800x480 desktop window, so apps can be
//! started from an IDE or a terminal on a machine that has a desktop session.
//! The glass backdrop is not drawn here: only the egui layer is, so effects
//! that need kiosk-kit's GL backend will look plainer than on the device.

use kiosk_kit::App;

/// Window size of the target kiosk display.
const SIZE: [f32; 2] = [800.0, 480.0];

struct Host<A>(A);

impl<A: App> eframe::App for Host<A> {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.0.ui(ui);
    }
}

/// Opens a window titled `title` and runs `app` until it is closed.
pub fn run<A: App + 'static>(title: &str, app: A) -> anyhow::Result<()> {
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(SIZE),
        ..Default::default()
    };
    eframe::run_native(title, opts, Box::new(|_| Ok(Box::new(Host(app)))))
        .map_err(|e| anyhow::anyhow!("{e}"))
}
