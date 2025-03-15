use bytes::Bytes;
use egui::load::Result;
use egui::TextureHandle;
use reqwest::{
    blocking::Client,
    header::{ACCEPT, ACCEPT_ENCODING, USER_AGENT},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

#[derive(Deserialize)]
pub struct ScryfallSearchResponse {
    /// Typically "list" for a list response.
    pub data: Vec<Card>,
    pub total_cards: Option<u32>,
    #[serde(flatten)]
    pub _extra: Value,
    //    pub object: String,
    //    pub has_more: bool,
}

impl Default for ScryfallSearchResponse {
    fn default() -> Self {
        Self {
            data: vec![],
            total_cards: None,
            _extra: Value::default(),
            //            object: "".to_owned(),
            //            has_more: false,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Card {
    pub set: String,
    pub name: String,
    pub id: String,
    #[serde(default)]
    pub image_uris: Option<ImageUris>,
    #[serde(default)]
    pub prints_search_uri: String,
    #[serde(default)]
    pub type_line: Option<String>,
    pub oracle_text: Option<String>,
    #[serde(default, skip)]
    pub image_texture: Option<TextureHandle>,
    #[serde(flatten)]
    pub _extra: Value,
}
impl Clone for Card {
    fn clone(&self) -> Self {
        Card {
            set: self.set.clone(),
            name: self.name.clone(),
            id: self.id.clone(),
            image_uris: self.image_uris.clone(),
            prints_search_uri: self.prints_search_uri.clone(),
            type_line: self.type_line.clone(),
            oracle_text: self.oracle_text.clone(),
            image_texture: self.image_texture.clone(),
            _extra: self._extra.clone(),
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
#[derive(Clone)]
pub struct ImageUris {
    pub small: String,
    pub normal: String,
    pub large: String,
    pub png: String,
    pub art_crop: String,
    pub border_crop: String,
}

pub struct ScryfallApiClient {
    client: Client,
}

impl Default for ScryfallApiClient {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl ScryfallApiClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Given a string, perform a serach on scryfalls database. NEED TO IMPROVE OPTIONS.
    pub fn search(&self, card_name: String) -> Result<ScryfallSearchResponse, reqwest::Error> {
        let url = format!("https://api.scryfall.com/cards/search?&q={}", card_name);
        println!("url of the request: {}", url);
        let response = self
            .client
            .get(&url)
            .header(
                USER_AGENT,
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            )
            .header(ACCEPT, "application/json")
            .header(ACCEPT_ENCODING, "gzip, deflate, br, zstd")
            .timeout(Duration::from_secs(3))
            .send()?;

        let body_text = response.text()?;
        let body_json: ScryfallSearchResponse =
            serde_json::from_str(&body_text).unwrap_or_else(|err| {
                print!(
                    "There was an error parsing the text string into json: {}",
                    err
                );
                ScryfallSearchResponse::default()
            });
        //eprintln!("{:#?}", body_json);
        Ok(body_json)
    }

    /// Given a Card struct return a vector with all its card variations
    pub fn get_card_versions(
        &self,
        tx: mpsc::Sender<(Card, Bytes)>,
        card: &Card,
    ) -> Result<u32, reqwest::Error> {
        println!("Getting card versions wirh uri {}", card.prints_search_uri);
        let response = self
            .client
            .get(&card.prints_search_uri)
            .header(
                USER_AGENT,
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            )
            .header(ACCEPT, "application/json")
            .header(ACCEPT_ENCODING, "gzip, deflate, br, zstd")
            .timeout(Duration::from_secs(3))
            .send()?;

        let response_object: ScryfallSearchResponse =
            response.json().expect("Deserializign card prints");
        println!("Found {} versions of the card", response_object.data.len());
        let card_n = response_object.total_cards.unwrap();

        thread::spawn(move || {
            // Recreate the client to satisfy the borrow checker.
            let tmp_client = Client::new();
            let mut last_request_time = Instant::now() - Duration::from_millis(100);

            for card in response_object.data {
                let elapsed = last_request_time.elapsed();
                if elapsed < Duration::from_millis(100) {
                    thread::sleep(Duration::from_millis(100) - elapsed);
                }

                if let Some(uri) = card.image_uris.as_ref() {
                    let response = tmp_client
                        .get(uri.normal.as_str())
                        .header(
                            USER_AGENT,
                            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
                        )
                        .header(ACCEPT, "application/json")
                        .header(ACCEPT_ENCODING, "gzip, deflate, br, zstd")
                        .timeout(Duration::from_secs(3))
                        .send()
                        .unwrap_or_else(|e| panic!("Error downloading card image: {}", e));
                    last_request_time = Instant::now();
                    if let Ok(img) = response.bytes() {
                        tx.send((card, img))
                            .unwrap_or_else(|e| panic!("Error sending card image: {}", e));
                    }
                }
            }
        });

        Ok(card_n)
    }
}
