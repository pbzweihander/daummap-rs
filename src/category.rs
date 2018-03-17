use std::str::FromStr;
use super::{Error, ErrorKind, Place, Req, ReqOnce, Response, Result, Sort};
use std::convert::Into;
use serde_json;

#[derive(Clone)]
pub enum CategoryGroup {
    Mart,
    ConvStore,
    Kindergarten,
    School,
    Academy,
    Parking,
    Oil,
    Station,
    Bank,
    Culture,
    Agency,
    PubOffice,
    Tour,
    Accommodation,
    Food,
    Cafe,
    Hospital,
    Pharmacy,
}

#[derive(Clone)]
pub struct CategoryRequest {
    app_key: String,
    category_group: CategoryGroup,
    page: usize,
    longitude: Option<f32>,
    latitude: Option<f32>,
    radius: Option<usize>,
    rect: Option<(f32, f32, f32, f32)>,
    sort: Option<Sort>,
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

impl FromStr for CategoryGroup {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        use CategoryGroup::*;
        Ok(match s {
            "MT1" => Mart,
            "CS2" => ConvStore,
            "PS3" => Kindergarten,
            "SC4" => School,
            "AC5" => Academy,
            "PK6" => Parking,
            "OL7" => Oil,
            "SW8" => Station,
            "BK9" => Bank,
            "CT1" => Culture,
            "AG2" => Agency,
            "PO3" => PubOffice,
            "AT4" => Tour,
            "AD5" => Accommodation,
            "FD6" => Food,
            "CE7" => Cafe,
            "HP8" => Hospital,
            "PM9" => Pharmacy,
            _ => bail!(ErrorKind::ParseCategoryGroup(s.to_owned())),
        })
    }
}

impl CategoryGroup {
    pub fn to_code<'a>(&self) -> &'a str {
        use CategoryGroup::*;
        match *self {
            Mart => "MT1",
            ConvStore => "CS2",
            Kindergarten => "PS3",
            School => "SC4",
            Academy => "AC5",
            Parking => "PK6",
            Oil => "OL7",
            Station => "SW8",
            Bank => "BK9",
            Culture => "CT1",
            Agency => "AG2",
            PubOffice => "PO3",
            Tour => "AT4",
            Accommodation => "AD5",
            Food => "FD6",
            Cafe => "CE7",
            Hospital => "HP8",
            Pharmacy => "PM9",
        }
    }
}

impl CategoryRequest {
    pub fn circle(
        app_key: &str,
        category_group: CategoryGroup,
        longitude: f32,
        latitude: f32,
        radius: usize,
    ) -> Self {
        CategoryRequest {
            app_key: app_key.to_owned(),
            category_group,
            page: 1,
            longitude: Some(longitude),
            latitude: Some(latitude),
            radius: Some(radius),
            rect: None,
            sort: None,
        }
    }

    pub fn rect(
        app_key: &str,
        category_group: CategoryGroup,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    ) -> Self {
        CategoryRequest {
            app_key: app_key.to_owned(),
            category_group,
            page: 1,
            longitude: None,
            latitude: None,
            radius: None,
            rect: Some((x1, y1, x2, y2)),
            sort: None,
        }
    }

    pub fn sort(&mut self, sort: Sort) -> &mut Self {
        self.sort = Some(sort);
        self
    }

    pub fn get(&self) -> Response<Place, Self> {
        Req::<Place>::get(self)
    }
}

impl ReqOnce<Place> for CategoryRequest {
    fn to_url(&self) -> String {
        let mut s = String::from("https://dapi.kakao.com/v2/local/search/category.json");
        s = s + "?category_group_code=" + self.category_group.to_code();
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

impl Req<Place> for CategoryRequest {}

impl Into<Place> for RawPlace {
    fn into(self) -> Place {
        Place {
            id: self.id.parse::<usize>().ok(),
            name: self.place_name,
            category: self.category_name,
            category_group: self.category_group_code.parse::<CategoryGroup>().ok(),
            phone: self.phone,
            address: self.address_name,
            road_address: self.road_address_name,
            longitude: self.x.parse::<f32>().ok(),
            latitude: self.y.parse::<f32>().ok(),
            url: self.place_url,
            distance: self.distance.parse::<usize>().ok(),
        }
    }
}
