#![feature(str_escape)]
extern crate reqwest;
extern crate serde;
extern crate regex;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

pub mod types;

use std::str::FromStr;

use types::media::Media;

use regex::Regex;

/// Types of search for Qwant
pub enum SearchType {
    Web,
    News,
    Images,
    Videos,
    Shopping,
    Music
}

#[derive(Deserialize,Clone)]
/// Type that holds the status of the query
/// and all the data from it. A failed query
/// will return null for Data.
pub struct APIResponse {
    pub status: String,
    pub data: Option<Data>,
    #[serde(skip_deserializing)]
    pub search_str: Option<String>,
}

#[derive(Deserialize,Clone)]
/// The Data type changes based on whether or not
/// an error is received. When all goes well, you
/// get query, cache, and result. Otherwise, you
/// receive a result with empty items, and an
/// error code.
pub struct Data {
    pub query: Option<Query>,
    pub cache: Option<Cache>,
    pub result: QwantResult,
    pub error_code: Option<u32>
}

#[derive(Deserialize,Clone)]
pub struct Cache {
    pub key: String,
    pub created: u64,
    pub expiration: u64,
    pub status: String,
    pub age: u64,
}

#[derive(Deserialize,Clone)]
/// Struct representing the "query" field of Data.
pub struct Query {
    /// A valid locale string, e.g. "en_US"
    pub locale: String,
    /// String to search for
    pub query: String,
    /// Offset of search results
    pub offset: u32,
}

#[derive(Deserialize,Clone)]
pub struct QwantResult {
    pub items: Vec<Item>,
    pub filters: Filters,
    pub version: String,
    pub domain: Option<String>,
    pub last: Option<bool>,
}

#[derive(Deserialize,Clone)]
// mutable so that html can be stripped
pub struct Item {
    pub title: String,
    pub _id: String,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub favicon: Option<String>,
    pub url: String,
    pub source: Option<String>,
    pub desc: String,
    pub desc_short: Option<String>,
    pub position: Option<u64>,
    pub duration: Option<u64>,
    pub thumbnail: Option<String>,
    pub thumb_height: Option<u64>,
    pub thumb_width: Option<u64>,
    pub thumb_type: Option<String>,
    pub width: Option<String>,
    pub height: Option<String>,
    pub size: Option<String>,
    pub b_id: Option<String>,
    pub media_fullsize: Option<String>,
    pub count: Option<u64>,
    pub domain: Option<String>,
    pub date: Option<u64>,
    pub media: Option<String>,
    pub media_: Option<Vec<Media>>
}

#[derive(Deserialize,Clone)]
pub struct Filters {
    pub freshness: Freshness,
    pub size: Option<Size>,
    pub license: Option<License>,
}

#[derive(Deserialize,Clone)]
pub struct Size {
    pub label: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub selected: String,
    pub values: Vec<Values>
}

#[derive(Deserialize,Clone)]
pub struct License {
    pub label: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub selected: String,
    pub values: Vec<Values>
}

#[derive(Deserialize,Clone)]
pub struct Freshness {
    pub label: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub selected: String,
    pub values: Vec<Values>
}

#[derive(Deserialize,Clone)]
pub struct Values {
    pub value: String,
    pub label: String,
    pub translate: bool,
}

impl APIResponse {
    // TODO: Make this return a Result type rather than assuming nothing fails.
    /// Returns the Response struct from the search. Takes a valid locale string,
    /// like "en_US", an app id string, the string to search for, and a boolean
    /// value to determine seaarch result safety.
    pub fn new(query: &str, type_: &SearchType, safe: bool, locale: &str, id: &str) -> Option<APIResponse> {
        let type_str: &str = match *type_ {
            SearchType::Web => "web",
            SearchType::Images => "images",
            SearchType::Music => "music",
            SearchType::Shopping => "shopping",
            SearchType::Videos => "videos",
            SearchType::News => "news"
        };

        let safe_search = if safe {1} else {0};

        let search_str =
            format!(
                "https://api.qwant.com/api/search/{}?/count=10&device=desktop&extensionDisabled=true&safesearch={}&locale={}&q={}&t={}",
                    type_str,
                    safe_search,
                    locale,
                    query,
                    id);

        let req = match reqwest::get(search_str.as_str()) {
            Ok(req) => Some(req),
            Err(e) => {
                println!("Failed to request JSON from API");
                println!("{:?}", e);
                None
            }
        };

        if req.is_none() {
            return None
        }

        let mut req = req.unwrap();
        // TODO: Figure out forwarding the error rather than
        // using unwrap().
        let mut resp: APIResponse = serde_json::from_str(&req.text().unwrap()).unwrap();
        resp.search_str = Some(search_str);
        Some(resp)
    }

    pub fn next_page(self) -> Self {
        let s = self.clone();
        let offset = &s.data.unwrap().query.unwrap().offset;
        let search_str = s.search_str.unwrap();
        let re = Regex::new(r"(?:&offset=^\d$)").unwrap();

        let new_search = {
            re.replace_all(&search_str, "").escape_default();
            search_str.clone() + &String::from_str(&format!("&offset={}", (offset + 10))).unwrap()
        };

        let mut req = reqwest::get(new_search.clone().as_str()).expect("request JSON from API");
        let mut resp: APIResponse = serde_json::from_str(&req.text().unwrap()).unwrap();
        resp.data = Some(Data {
            cache: resp.clone().data.unwrap().cache,
            error_code: resp.clone().data.unwrap().error_code,
            query: Some(Query {
                locale: resp.clone().data.unwrap().query.unwrap().locale,
                offset: offset + 10,
                query: resp.clone().data.unwrap().query.unwrap().query,
            }),
            result: resp.clone().data.unwrap().result,
        });
        resp.search_str = Some(new_search);
        resp
    }

}

impl Item {
    /// Uses regex to strip <b></b> tags
    pub fn strip_html(&mut self) {
        let re = Regex::new(r"<(.|\n)*?>").unwrap();
        self.title = re.replace_all(&self.title, "").escape_default();
        self.desc = re.replace_all(&self.desc, "").escape_default();
        if self.desc_short.is_some() {
            self.desc_short = Some(re.replace_all(&self.clone().desc_short.unwrap(), "").escape_default());
        }
    }
}
