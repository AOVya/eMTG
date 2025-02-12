use bytes::Bytes;
use egui::load::Result;
use reqwest::{
    blocking::Client,
    header::{ACCEPT, ACCEPT_ENCODING, USER_AGENT},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{error::Error, time::Duration};

#[derive(Deserialize, Debug)]
pub struct ScryfallSearchResponse {
    /// Typically "list" for a list response.
    pub data: Vec<Card>,
    #[serde(flatten)]
    pub _extra: Value,
    //    pub object: String,
    //    pub has_more: bool,
}

impl Default for ScryfallSearchResponse {
    fn default() -> Self {
        Self {
            data: vec![],
            _extra: Value::default(),
            //            object: "".to_owned(),
            //            has_more: false,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
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
    #[serde(flatten)]
    pub _extra: Value,
    /*
       pub object: String,
       #[serde(rename = "oracle_id")]
       pub oracle_id: String,
       /// Some cards have no multiverse IDs so default to empty.
       #[serde(default)]
       pub multiverse_ids: Vec<u32>,
       #[serde(default)]
       pub ScryfallSearchResponsemtgo_id: Option<u32>,
       #[serde(default)]
       pub mtgo_foil_id: Option<u32>,
       #[serde(default)]
       pub tcgplayer_id: Option<u32>,
       #[serde(default)]
       pub cardmarket_id: Option<u32>,
       pub lang: String,
       pub released_at: String,
       pub uri: String,
       pub scryfall_uri: String,
       pub layout: String,
       pub highres_image: bool,
       pub image_status: String,
    Some cards include an image_uris object; others (like double‐faced cards) may not.
       /// For spells and creatures, the mana cost is a string such as "{1}{G}".
       #[serde(default)]
       pub mana_cost: Option<String>,
       #[serde(default)]
       pub cmc: Option<f64>,
       #[serde(default)]
       pub oracle_text: Option<String>,
       #[serde(default)]
       pub power: Option<String>,
       #[serde(default)]
       pub toughness: Option<String>,
       #[serde(default)]
       pub colors: Vec<String>,
       #[serde(default)]
       pub color_identity: Vec<String>,
    There are many additional fields (legalities, prices, related_uris, etc.)
    */
}

#[derive(Serialize, Deserialize, Debug)]
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
    pub fn get_card_versions(&self, card: &Card) -> Result<Vec<Card>, reqwest::Error> {
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
        Ok(response_object.data)
    }

    /// Given a card image Uri return the raw pixels in bytes (ready to load into egui image)
    pub fn download_card_image(&self, card_image_uri: &String) -> Result<Bytes, Box<dyn Error>> {
        let response = self
            .client
            .get(card_image_uri)
            .header(
                USER_AGENT,
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            )
            .header(ACCEPT, "application/json")
            .header(ACCEPT_ENCODING, "gzip, deflate, br, zstd")
            .timeout(Duration::from_secs(3))
            .send()?;

        // println!("{:?}", String::from(response.text()?));
        let image_bytes = response.bytes()?;

        Ok(image_bytes)
    }
}
