use super::{request, Element, Req, ReqOnce, Response, Result};
use std::hash::{Hash, Hasher};
use std::cmp::{Eq, PartialEq};
use std::convert::From;
use serde_json;

#[derive(Clone)]
pub struct AddressRequest {
    app_key: String,
    query: String,
    page: usize,
}

#[derive(Clone)]
pub struct Address {
    pub land_lot: Option<LandLotAddress>,
    pub road: Option<RoadAddress>,
}

#[derive(Clone)]
pub struct LandLotAddress {
    pub address: String,
    pub province: String,
    pub city: String,
    pub town: String,
    pub neighborhood: Option<String>,
    pub h_code: Option<usize>,
    pub b_code: Option<usize>,
    pub is_mountain: Option<bool>,
    pub main_address_number: Option<usize>,
    pub sub_address_number: Option<usize>,
    pub zip_code: Option<usize>,
    pub longitude: Option<f32>,
    pub latitude: Option<f32>,
}

#[derive(Clone)]
pub struct RoadAddress {
    pub address: String,
    pub province: String,
    pub city: String,
    pub town: String,
    pub road_name: String,
    pub is_underground: bool,
    pub main_building_number: Option<usize>,
    pub sub_building_number: Option<usize>,
    pub building_name: String,
    pub post_code: Option<usize>,
    pub longitude: Option<f32>,
    pub latitude: Option<f32>,
}

#[derive(Deserialize)]
struct RawResponse {
    documents: Vec<Document>,
}

#[derive(Deserialize)]
struct Document {
    address: Option<RawLandLotAddress>,
    road_address: Option<RawRoadAddress>,
}

#[derive(Deserialize)]
struct RawLandLotAddress {
    address_name: String,
    region_1depth_name: String,
    region_2depth_name: String,
    region_3depth_name: String,
    region_3depth_h_name: String,
    h_code: String,
    b_code: String,
    mountain_yn: String,
    main_address_no: String,
    sub_address_no: String,
    zip_code: String,
    x: String,
    y: String,
}

#[derive(Deserialize)]
struct RawRoadAddress {
    address_name: String,
    region_1depth_name: String,
    region_2depth_name: String,
    region_3depth_name: String,
    road_name: String,
    undergroun_yn: String,
    main_building_no: String,
    sub_building_no: String,
    building_name: String,
    zone_no: String,
    x: String,
    y: String,
}

impl AddressRequest {
    pub fn new(app_key: &str, query: &str) -> Self {
        AddressRequest {
            app_key: app_key.to_owned(),
            query: query.to_owned(),
            page: 1,
        }
    }

    fn to_url(&self) -> String {
        let mut s = String::from("https://dapi.kakao.com/v2/local/search/address.json?query=")
            + &self.query;
        s = s + "&page=" + &self.page.to_string();
        s
    }

    pub fn get(&self) -> Response<Address, Self> {
        Req::<Address>::get(self)
    }
}

impl ReqOnce<Address> for AddressRequest {
    fn page(&mut self, page: usize) -> &mut Self {
        self.page = page;
        self
    }

    fn request(&self) -> Result<Vec<Address>> {
        request(&self.to_url(), &self.app_key)
            .and_then(|v| serde_json::from_value::<RawResponse>(v).map_err(|e| e.into()))
            .map(|r| {
                r.documents
                    .into_iter()
                    .map(|d| Address {
                        land_lot: d.address.map(|r| r.into()),
                        road: d.road_address.map(|r| r.into()),
                    })
                    .filter(|a| a.land_lot.is_some() || a.road.is_some())
                    .collect()
            })
    }
}

impl Req<Address> for AddressRequest {}

impl From<RawLandLotAddress> for LandLotAddress {
    fn from(raddr: RawLandLotAddress) -> Self {
        LandLotAddress {
            address: raddr.address_name,
            province: raddr.region_1depth_name,
            city: raddr.region_2depth_name,
            town: raddr.region_3depth_name,
            neighborhood: if raddr.region_3depth_h_name.is_empty() {
                None
            } else {
                Some(raddr.region_3depth_h_name)
            },
            h_code: raddr.h_code.parse::<usize>().ok(),
            b_code: raddr.b_code.parse::<usize>().ok(),
            is_mountain: if raddr.mountain_yn.is_empty() {
                None
            } else {
                Some(raddr.mountain_yn == "Y")
            },
            main_address_number: raddr.main_address_no.parse::<usize>().ok(),
            sub_address_number: raddr.sub_address_no.parse::<usize>().ok(),
            zip_code: raddr.zip_code.parse::<usize>().ok(),
            longitude: raddr.x.parse::<f32>().ok(),
            latitude: raddr.y.parse::<f32>().ok(),
        }
    }
}

impl From<RawRoadAddress> for RoadAddress {
    fn from(raddr: RawRoadAddress) -> Self {
        RoadAddress {
            address: raddr.address_name,
            province: raddr.region_1depth_name,
            city: raddr.region_2depth_name,
            town: raddr.region_3depth_name,
            road_name: raddr.road_name,
            is_underground: raddr.undergroun_yn == "Y",
            main_building_number: raddr.main_building_no.parse::<usize>().ok(),
            sub_building_number: raddr.sub_building_no.parse::<usize>().ok(),
            building_name: raddr.building_name,
            post_code: raddr.zone_no.parse::<usize>().ok(),
            longitude: raddr.x.parse::<f32>().ok(),
            latitude: raddr.y.parse::<f32>().ok(),
        }
    }
}

impl Hash for Address {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.land_lot.hash(state);
        self.road.hash(state);
    }
}

impl Hash for LandLotAddress {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address.hash(state);
        self.zip_code.hash(state);
    }
}

impl Hash for RoadAddress {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address.hash(state);
        self.post_code.hash(state);
    }
}

impl PartialEq for Address {
    fn eq(&self, other: &Self) -> bool {
        self.land_lot == other.land_lot || self.road == other.road
    }
}

impl PartialEq for LandLotAddress {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address && self.zip_code == other.zip_code
    }
}

impl PartialEq for RoadAddress {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address && self.post_code == other.post_code
    }
}

impl Eq for Address {}

impl Element for Address {}
