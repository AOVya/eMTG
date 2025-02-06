use std::thread;

use crate::scryfall_models::{Card, ImageUris, ScryfallApiClient};
use egui_extras::{Column, TableBuilder};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct CardSearchView {
    card_search_spot: String,
    card_search_result_table: Vec<Card>,
    #[serde(skip)]
    client: ScryfallApiClient,
    #[serde(skip)]
    card_display: Vec<Card>,
}

impl Default for CardSearchView {
    fn default() -> Self {
        Self {
            card_search_spot: "type card name here...".to_string(),
            card_search_result_table: vec![],
            client: ScryfallApiClient::new(),
            card_display: vec![],
        }
    }
}

impl CardSearchView {
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
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
            self.show_cards_table_extras(ui);
        });
    }
    fn show_cards_table_extras(&mut self, ui: &mut egui::Ui) {
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
                        row.col(|ui| if ui.label(&card.name).clicked() {
                            let card_versions = self.client.get_card_versions(&card).expect("Error getting card versions");
                            for card in card_versions {
                                self.card_display.push(card);
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
