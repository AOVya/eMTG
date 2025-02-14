use crate::scryfall_models::{Card, ScryfallApiClient};
use egui::{vec2, Color32, Frame, Image, Margin, TextureHandle};
use egui_extras::{Column, Size, TableBuilder};
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use bytes::Bytes;
use eframe::wgpu::Color;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct CardSearchView {
    card_search_spot: String,
    #[serde(skip)]
    are_cards_loading: bool,
    #[serde(skip)]
    card_search_result_table: Vec<Card>,
    #[serde(skip)]
    cards_in_display: u16,
    #[serde(skip)]
    client: ScryfallApiClient,
    #[serde(skip)]
    card_display: HashMap<String, TextureHandle>,
    #[serde(skip)]
    rx: Option<mpsc::Receiver<(String, Bytes)>>,
    #[serde(skip)]
    tx: Option<mpsc::Sender<(String, Bytes)>>,
}

impl Default for CardSearchView {
    fn default() -> Self {
        let (tx, rx): (Sender<(String, Bytes)>, Receiver<(String, Bytes)>) = mpsc::channel();
        Self {
            card_search_spot: "angel".to_string(),
            are_cards_loading: false,
            card_search_result_table: vec![],
            client: ScryfallApiClient::new(),
            card_display: HashMap::new(),
            cards_in_display: 0,
            rx: Some(rx),
            tx: Some(tx),
        }
    }
}

impl CardSearchView {
    pub fn draw(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.show_search_bar(ui);
        ui.separator();
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min).with_cross_justify(true), |ui| {
            self.show_cards_table_extras(ui);
            self.show_card_versions(ui, ctx);
        });
    }

    fn show_search_bar(&mut self, ui: &mut egui::Ui) {
       ui.horizontal(|ui| {
            // let mut line_text = "type card name here";
            ui.text_edit_singleline(&mut self.card_search_spot);
            if ui.button("Search").clicked() {
                let result = self.client.search(self.card_search_spot.to_string());
                match result {
                    Ok(info) => {
                        self.card_search_result_table = info.data;
                    }
                    Err(e) => {
                        panic!("Error with the search reqwest: {}", e)
                    }
                }
            }
        });
    }

    fn show_card_versions(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Define the desired minimum cell width.
        let cell_width: f32 = 250.0;
        // Calculate how many cells we can fit in the available width.
        let available_width = ui.available_size().x;
        let mut num_columns = (available_width / cell_width).floor() as usize;
        if num_columns == 0 {
            num_columns = 1;
        }
        ui.vertical(|ui| {
            if let Ok((id, image)) = self.rx.as_ref().unwrap().try_recv() {
                let dyn_image = image::load_from_memory(&image).unwrap();
                let size = [
                    dyn_image.width() as usize,
                    dyn_image.height() as usize,
                ];
                let image_buffer = dyn_image.to_rgba8(); // Convert to RGBA8 format.
                let pixels = image_buffer.into_raw();
                let egui_cpu_image =
                    egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                // This sends the image to the gpu for faster render and extra
                // memory
                let texture = ctx.load_texture(
                    format!("{}", id),
                    egui_cpu_image,
                    Default::default(),
                );
                self.card_display.insert(id, texture);
            }
            ui.horizontal(|ui| {
                if !self.card_display.is_empty() {
                    ui.heading("Card Versions");
                }
                let progress = self.card_display.len() as f32 / self.cards_in_display as f32;
                if self.are_cards_loading && progress < 1.0 {
                    ui.add(egui::ProgressBar::new(progress).show_percentage().animate(true));
                }
                if progress == 1.0 {
                    self.are_cards_loading = false;
                }
            });
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("cards_grid")
                    .num_columns(num_columns)
                    .spacing([10.0, 10.0])
                    .show(ui, |ui| {
                        for (i, card) in self.card_display.iter().enumerate() {
                            // End the row after filling a row with the computed number of columns.
                            ui.add(
                                Image::new(card.1)
                                    .rounding(15.0)
                                    .max_width(cell_width)
                                    .maintain_aspect_ratio(true)
                                    .fit_to_original_size(1.0)
                                    .bg_fill(Color32::WHITE)
                            );
                            if (i + 1) % (num_columns) == 0 {
                                ui.end_row();
                            }
                        }
                        ui.end_row();
                    })
            });
        });
    }

    fn show_cards_table_extras(&mut self, ui: &mut egui::Ui) {
        // Create a new TableBuilder on the provided UI.
        if self.card_search_result_table.is_empty() {
            return;
        }
        ui.vertical(|ui| {
            TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Max))
                .columns(Column::auto(), 3)
                .sense(egui::Sense::click())
                .header(18.0, |mut header| {
                    // Header for the first column: Name
                    header.col(|ui| {
                        ui.strong("Name");
                    });
                    // Header for the second column: Type
                    header.col(|ui| {
                        ui.strong("Type");
                    });
                    // Header for the third column: Set
                    header.col(|ui| {
                        ui.strong("Set");
                    });
                })
                // Build the table body:
                .body(|mut body| {
                    for card in &self.card_search_result_table {
                        // You can adjust the row height as needed; here we use 30.0.
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label(&card.name);
                            });
                            row.col(|ui| {
                                ui.label(card.type_line.as_deref().unwrap_or("Unknown"));
                            });
                            row.col(|ui| {
                                ui.label(&card.set.to_ascii_uppercase());
                            });
                            if row.response().clicked() {
                                self.card_display.clear();
                                self.cards_in_display = self
                                    .client
                                    .get_card_versions(self.tx.clone().unwrap(), card)
                                    .expect("Error getting card versions") as u16;
                                self.are_cards_loading = true;
                            }
                        });
                    }
                });
        });
    }
}
