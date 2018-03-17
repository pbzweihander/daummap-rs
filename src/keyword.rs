use super::{CategoryGroup, Element, Req, ReqOnce, Response, Result, Sort};
use std::hash::{Hash, Hasher};
use std::cmp::{Eq, PartialEq};
use std::convert::From;
use serde_json;

#[derive(Clone)]
pub struct KeywordRequest {
    app_key: String,
    query: String,
    page: usize,
    category_group: Option<CategoryGroup>,
    longitude: Option<f32>,
    latitude: Option<f32>,
    radius: Option<usize>,
    rect: Option<(f32, f32, f32, f32)>,
    sort: Option<Sort>,
}

#[derive(Clone)]
pub struct Place {
    pub id: Option<usize>,
    pub name: String,
    pub category: String,
    pub category_group: Option<CategoryGroup>,
    pub phone: String,
    pub address: String,
    pub road_address: String,
    pub longitude: Option<f32>,
    pub latitude: Option<f32>,
    pub url: String,
    pub distance: Option<usize>,
}

#[derive(Deserialize)]
struct RawResponse {
    documents: Vec<RawPlace>,
}

#[derive(Deserialize)]
struct RawPlace {
    id: String,
    place_name: String,
    category_name: String,
    category_group_code: String,
    phone: String,
    address_name: String,
    road_address_name: String,
    x: String,
    y: String,
    place_url: String,
    distance: String,
}

impl KeywordRequest {
    pub fn new(app_key: &str, query: &str) -> Self {
        KeywordRequest {
            app_key: app_key.to_owned(),
            query: query.to_owned(),
            page: 1,
            category_group: None,
            longitude: None,
            latitude: None,
            radius: None,
            rect: None,
            sort: None,
        }
    }

    pub fn category_group(&mut self, group: CategoryGroup) -> &mut Self {
        self.category_group = Some(group);
        self
    }

    pub fn longitude(&mut self, x: f32) -> &mut Self {
        self.longitude = Some(x);
        self
    }

    pub fn latitude(&mut self, y: f32) -> &mut Self {
        self.latitude = Some(y);
        self
    }

    pub fn coord(&mut self, longitude: f32, latitude: f32) -> &mut Self {
        self.longitude = Some(longitude);
        self.latitude = Some(latitude);
        self
    }

    pub fn radius(&mut self, r: usize) -> &mut Self {
        self.radius = Some(r);
        self
    }

    pub fn rect(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) -> &mut Self {
        self.rect = Some((x1, y1, x2, y2));
        self
    }

    pub fn sort(&mut self, sort: Sort) -> &mut Self {
        self.sort = Some(sort);
        self
    }

    pub fn get(&self) -> Response<Place, Self> {
        Req::<Place>::get(self)
    }
}

impl ReqOnce<Place> for KeywordRequest {
    fn to_url(&self) -> String {
        let mut s = String::from("https://dapi.kakao.com/v2/local/search/keyword.json");
        s = s + "?query=" + &self.query;
        s = s + "&page=" + &self.page.to_string();
        if let Some(ref c) = self.category_group {
            s = s + "&category_group_code=" + c.to_code();
        }
        if let Some(x) = self.longitude {
            s = s + "&x=" + &x.to_string();
        }
        if let Some(y) = self.latitude {
            s = s + "&y=" + &y.to_string();
        }
        if let Some(r) = self.radius {
            s = s + "&radius=" + &r.to_string();
        }
        if let Some((x1, y1, x2, y2)) = self.rect {
            s = s + &format!("&rect={},{},{},{}", x1, y1, x2, y2);
        }
        if let Some(ref ss) = self.sort {
            use self::Sort::*;
            s = s + "&sort=" + match *ss {
                Accuracy => "accuracy",
                Distance => "distance",
            }
        }
        s
    }

    fn get_app_key(&self) -> &str {
        &self.app_key
    }

    fn page(&mut self, page: usize) -> &mut Self {
        self.page = page;
        self
    }

    fn deserialize(value: serde_json::Value) -> Result<Vec<Place>> {
        serde_json::from_value::<RawResponse>(value)
            .map_err(|e| e.into())
            .map(|r| r.documents.into_iter().map(|r| r.into()).collect())
    }
}

impl Req<Place> for KeywordRequest {}

impl Hash for Place {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
        self.address.hash(state);
    }
}

impl PartialEq for Place {
    fn eq(&self, other: &Self) -> bool {
        let b: bool = if self.id.is_some() && other.id.is_some() {
            self.id.unwrap() == other.id.unwrap()
        } else {
            true
        };
        b && self.name == other.name
    }
}

impl Eq for Place {}

impl Element for Place {}

impl From<RawPlace> for Place {
    fn from(raddr: RawPlace) -> Self {
        Place {
            id: raddr.id.parse::<usize>().ok(),
            name: raddr.place_name,
            category: raddr.category_name,
            category_group: raddr.category_group_code.parse::<CategoryGroup>().ok(),
            phone: raddr.phone,
            address: raddr.address_name,
            road_address: raddr.road_address_name,
            longitude: raddr.x.parse::<f32>().ok(),
            latitude: raddr.y.parse::<f32>().ok(),
            url: raddr.place_url,
            distance: raddr.distance.parse::<usize>().ok(),
        }
    }
}
