use egui::{Button, Color32, Response, RichText};
use log::{log, Level};

use crate::card_search_view::CardSearchView;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    main_panel: String,

    #[serde(skip)]
    value: f32,
    #[serde(skip)]
    card_search_view: CardSearchView,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            value: 2.7,
            main_panel: "none".to_owned(),
            card_search_view: CardSearchView::default(),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::SidePanel::left("test_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.add_space(16.0);
                let button_builder =
                    Button::new(RichText::new("Card searcher").color(Color32::WHITE));
                let button: Response;
                if self.main_panel == "card_searcher" {
                    button = ui.add(button_builder.fill(Color32::from_rgb(0, 120, 215)));
                } else {
                    button = ui.add(button_builder);
                }
                if button.clicked() {
                    if self.main_panel != "card_searcher" {
                        self.main_panel = "card_searcher".to_string();
                        log!(Level::Info, "changed main to card_searcher");
                    } else {
                        self.main_panel = "none".to_string();
                    }
                };
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            match self.main_panel.as_str() {
                "card_searcher" => {
                    self.card_search_view.draw(ui, ctx);
                }
                _ => {
                    ui.heading("Welcome to eMTG");
                }
            };

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}
