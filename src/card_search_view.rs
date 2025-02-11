use egui::TextureHandle;
use std::collections::HashMap;

use crate::scryfall_models::{Card, ScryfallApiClient};
use egui_extras::{Column, TableBuilder};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct CardSearchView {
    card_search_spot: String,
    card_search_result_table: Vec<Card>,
    #[serde(skip)]
    client: ScryfallApiClient,
    #[serde(skip)]
    card_display: HashMap<String, TextureHandle>,
}

impl Default for CardSearchView {
    fn default() -> Self {
        Self {
            card_search_spot: "type card name here...".to_string(),
            card_search_result_table: vec![],
            client: ScryfallApiClient::new(),
            card_display: HashMap::new(),
        }
    }
}

impl CardSearchView {
    pub fn draw(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                self.show_search_bar(ui);
                self.show_cards_table_extras(ui, ctx);
            });
            self.show_card_versions(ui);
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

    fn show_card_versions(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("version_gallery")
            .spacing([10.0, 10.0])
            .show(ui, |ui| {
                let columns = 4;
                let mut i = 0;
                for (id, texture) in self.card_display.iter() {
                    ui.image(texture);
                    if (i + 1) % columns == 0 {
                        ui.end_row();
                    }
                    i += 1;
                }
                if self.card_display.len() % columns != 0 {
                    ui.end_row();
                }
            });
    }

    fn show_cards_table_extras(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Create a new TableBuilder on the provided UI.
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Max))
            .columns(Column::auto(), 3)
            .header(3.0, |mut header| {
                // Header for the first column: Name
                header.col(|ui| {
                    ui.label("Name");
                });
                // Header for the second column: Type
                header.col(|ui| {
                    ui.label("Type");
                });
                // Header for the third column: Set
                header.col(|ui| {
                    ui.label("Set");
                });
            })
            // Build the table body:
            .body(|mut body| {
                for card in &self.card_search_result_table {
                    // You can adjust the row height as needed; here we use 30.0.
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            if ui.label(&card.name).clicked() {
                                let card_versions = self
                                    .client
                                    .get_card_versions(&card)
                                    .expect("Error getting card versions");
                                for card in card_versions {
                                    if let Ok(image) = self.client.download_card_image(
                                        &card
                                            .image_uris
                                            .expect("no image uris found for image")
                                            .normal,
                                    ) {
                                        let dyn_image = image::load_from_memory(&image).unwrap();
                                        let size = [
                                            dyn_image.width() as usize,
                                            dyn_image.height() as usize,
                                        ];
                                        let image_buffer = dyn_image.to_rgba8(); // Convert to RGBA8 format.
                                        let pixels = image_buffer.into_raw();
                                        let egui_cpu_image =
                                            egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                                        let texture = ctx.load_texture(
                                            format!("{}", card.id),
                                            egui_cpu_image,
                                            Default::default(),
                                        );
                                        self.card_display.insert(card.id, texture);
                                    }
                                }
                            }
                        });
                        row.col(|ui| {
                            ui.label(card.type_line.as_deref().unwrap_or("Unknown"));
                        });
                        row.col(|ui| {
                            ui.label(&card.set.to_ascii_uppercase());
                        });
                    });
                }
            });
    }
}
