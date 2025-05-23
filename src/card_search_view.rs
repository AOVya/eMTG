use crate::scryfall_models::{Card, ScryfallApiClient};
use bytes::Bytes;
use egui::{Image, TextureHandle};
use egui::{ImageButton, Response, Sense};
use egui_extras::{Column, TableBuilder};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

pub const CELL_WIDTH: f32 = 250.;

pub struct CardSearchView {
    card_search_spot: String,
    single_card_view: SingleCardView,
    selected_card_in_table: Option<String>,
    are_cards_loading: bool,
    card_search_result: Vec<Card>,
    cards_in_display: u16,
    client: ScryfallApiClient,
    card_display: Vec<Card>,
    rx: Option<Receiver<(Card, Bytes)>>,
    tx: Option<Sender<(Card, Bytes)>>,
}

impl Default for CardSearchView {
    fn default() -> Self {
        Self {
            card_search_spot: "angel".to_string(),
            single_card_view: SingleCardView::default(),
            selected_card_in_table: None,
            are_cards_loading: false,
            card_search_result: vec![],
            client: ScryfallApiClient::new(),
            card_display: vec![],
            cards_in_display: 0,
            rx: None,
            tx: None,
        }
    }
}

impl CardSearchView {
    pub fn draw(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.show_search_bar(ui);
        ui.separator();
        ui.with_layout(
            egui::Layout::left_to_right(egui::Align::Min).with_cross_justify(true),
            |ui| {
                self.show_card_list(ui);
                if self.single_card_view.is_loaded() {
                    self.single_card_view.draw(ui);
                } else {
                    self.show_card_versions(ui, ctx);
                }
            },
        );
    }

    fn img_bytes_to_texture(&self, img_bytes: Bytes, ctx: &egui::Context, id: String) -> TextureHandle {
        let dyn_image = image::load_from_memory(&img_bytes).unwrap();
        let size = [dyn_image.width() as usize, dyn_image.height() as usize];
        let image_buffer = dyn_image.to_rgba8(); // Convert to RGBA8 format.
        let pixels = image_buffer.into_raw();
        let egui_cpu_image =
            egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
        // This sends the image to the gpu for faster render and extra
        // memory
        ctx.load_texture(
            format!("{}", id),
            egui_cpu_image,
            Default::default(),
        )
    }

    fn show_search_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // let mut line_text = "type card name here";
            ui.text_edit_singleline(&mut self.card_search_spot);
            if ui.button("Search").clicked() {
                let result = self.client.search(self.card_search_spot.to_string());
                match result {
                    Ok(info) => {
                        self.card_search_result = info.data;
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
        // Calculate how many cells we can fit in the available width.
        let available_width = ui.available_size().x;
        let mut num_columns = (available_width / CELL_WIDTH).floor() as usize;
        if num_columns == 0 {
            num_columns = 1;
        }

        // Below here is code for receiving card img bytes from channel and painting them. For the
        // card info view this needs to be inside a match for a selected card.
        ui.vertical(|ui| {
            if let Some(rx) = self.rx.as_ref() {
                if let Ok((mut card, img_bytes)) = rx.try_recv() {
                    let texture = self.img_bytes_to_texture(img_bytes, ctx, card.id.clone());
                    card.image_texture = Some(texture);
                    self.card_display.push(card);
                }
            }
            ui.horizontal(|ui| {
                if !self.card_display.is_empty() {
                    ui.heading("Card Versions");
                }
                let progress =
                    self.card_display.len() as f32 / self.cards_in_display as f32;
                if self.are_cards_loading && progress < 1.0 {
                    ui.add(
                        egui::ProgressBar::new(progress)
                            .show_percentage()
                            .animate(true),
                    );
                }
                if progress == 1.0 {
                    self.are_cards_loading = false;
                    self.rx = None;
                    self.tx = None;
                }
            });
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("cards_grid")
                    .num_columns(num_columns)
                    .spacing([10.0, 10.0])
                    .show(ui, |ui| {
                        for (i, card) in self.card_display.iter().enumerate() {
                            if let Some(img_texture) = &card.image_texture {
                                let response: Response;
                                response = ui.add(
                                    ImageButton::new(
                                        Image::new(img_texture)
                                            .rounding(15.0)
                                            .max_width(CELL_WIDTH)
                                            .maintain_aspect_ratio(true)
                                            .fit_to_original_size(1.0)
                                            .bg_fill(egui::Color32::WHITE),
                                    )
                                        .frame(true)
                                        .sense(Sense::click()),
                                );
                                if response.clicked() {
                                    self.single_card_view.load(card.clone());
                                }
                            };
                            // End the row after filling a row with the computed number of columns.
                            if (i + 1) % (num_columns) == 0 {
                                ui.end_row();
                            }
                        }
                        ui.end_row();
                    })
            });
        });
    }
    fn show_card_list(&mut self, ui: &mut egui::Ui) {
        if self.card_search_result.is_empty() {
            return;
        }
        ui.vertical(|ui| {
            TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Max))
                .columns(Column::auto(), 3)
                .sense(Sense::click())
                .header(18.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Name");
                    });
                    header.col(|ui| {
                        ui.strong("Type");
                    });
                    header.col(|ui| {
                        ui.strong("Set");
                    });
                })
                .body(|mut body| {
                    for card in &self.card_search_result {
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
                            match &self.selected_card_in_table {
                                Some(selected_card) => {
                                    if *selected_card == card.name {
                                        row.set_selected(true)
                                    }
                                }
                                _ => {
                                    row.set_selected(false);
                                }
                            }
                            if row.response().clicked() {
                                let (tx, rx) = mpsc::channel();
                                self.tx = Some(tx);
                                self.rx = Some(rx);
                                self.card_display.clear();
                                self.cards_in_display = self
                                    .client
                                    .get_card_versions(self.tx.clone().unwrap(), card)
                                    .expect("Error getting card versions")
                                    as u16;
                                self.are_cards_loading = true;
                                self.selected_card_in_table = Some(card.name.clone());
                                self.single_card_view.clear();
                            }
                        });
                    }
                });
        });
    }
}

struct SingleCardView {
    card: Option<Card>,
}

impl Default for SingleCardView {
    fn default() -> Self {
        SingleCardView { card: None }
    }
}

impl SingleCardView {
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        if let Some(card) = &self.card {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    if let Some(txtr_ref) = &card.image_texture {
                        ui.add(Image::new(txtr_ref)
                            .rounding(15.0)
                            .max_width(CELL_WIDTH)
                            .maintain_aspect_ratio(true)
                            .fit_to_original_size(1.0)
                            .bg_fill(egui::Color32::WHITE));
                    }
                });
                ui.vertical(|ui| {
                    ui.heading("Name:".to_string());
                    ui.label(&card.name);
                    if let Some(card_type) = &card.type_line {
                        ui.heading("Type:".to_string());
                        ui.label(card_type);
                    }
                    if let Some(oracle_text) = &card.oracle_text {
                        ui.heading("Oracle Text:".to_string());
                        ui.label(oracle_text);
                    }
                });
            });

        }
    }
    
    pub fn is_loaded(&self) -> bool {
        self.card.is_some()
    }
    
    pub fn load(&mut self, card: Card) {
        self.card = Some(card);
    }
    
    pub fn clear(&mut self) {
        self.card = None;
    }
}
