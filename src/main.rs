pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let (tx, rx) = std::sync::mpsc::channel::<audio::synth::Control>();

    std::thread::spawn(move || {
        audio::synth::init(rx).unwrap();
    });

    // egui application
    {
        let tx = tx.clone();
        eframe::run_native(
            "audio",
            eframe::NativeOptions::default(),
            Box::new(|cc| Box::new(App::new(tx, cc))),
        )
        .unwrap();
    }

    tx.send(audio::synth::Control::Exit).unwrap();

    Ok(())
}

use audio::synth::Control;

pub struct App {
    state: State,
    synth: std::sync::mpsc::Sender<Control>,
}

// allow app state to be persisted on shutdown
#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(default)] // if new fields are added, use default values when deserializing
pub struct State {
    heading: String,
    clicked: bool,
}

impl App {
    pub fn new(tx: std::sync::mpsc::Sender<Control>, cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return Self {
                synth: tx,
                state: eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default(),
            };
        }

        Self {
            synth: tx,
            state: Default::default(),
        }
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        {
            use egui::FontFamily::Proportional;
            use egui::FontId;
            use egui::TextStyle::*;

            let mut style = (*ctx.style()).clone();
            *style.text_styles.get_mut(&Heading).unwrap() = FontId::new(14.0, Proportional);
            ctx.set_style(style);
        }

        egui::TopBottomPanel::top("top_panel")
            .frame(
                egui::Frame::side_top_panel(ctx.style().as_ref())
                    .inner_margin(egui::Margin::symmetric(8.0, 6.0)),
            )
            .show(ctx, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
            });

        let clicked = self.state.clicked;
        egui::SidePanel::left("side_panel")
            .frame(
                egui::Frame::side_top_panel(ctx.style().as_ref())
                    .inner_margin(egui::Margin::same(8.0)),
            )
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::default().with_cross_justify(true), |ui| {
                    if ui.button("click me!").clicked() {
                        self.state.clicked = !clicked;
                    }

                    if ui.button("play").clicked() {
                        self.synth.send(Control::Play).unwrap();
                    }

                    if ui.button("pause").clicked() {
                        self.synth.send(Control::Pause).unwrap();
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |_ui| {
            if self.state.clicked {
                egui::Window::new("toggle_window")
                    .open(&mut self.state.clicked)
                    .show(ctx, |ui| {
                        if clicked {
                            ui.label("clicked!");
                        }
                    });
            }
        });
    }
}
