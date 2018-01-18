use super::{request, Address, Element, LandLotAddress, Req, ReqOnce, Response, Result, RoadAddress};
use std::hash::{Hash, Hasher};
use std::cmp::{Eq, PartialEq};
use std::convert::{From, Into};
use serde_json;

#[derive(Deserialize)]
struct Coord2AddressDocument {
    address: Option<RawLandLotAddress>,
    road_address: Option<RawRoadAddress>,
}

#[derive(Deserialize)]
struct Coord2AddressResponse {
    documents: Vec<Coord2AddressDocument>,
}

#[derive(Deserialize)]
struct Coord2RegionResponse {
    documents: Vec<RawRegion>,
}

#[derive(Clone)]
pub struct CoordRequest {
    app_key: String,
    longitude: f32,
    latitude: f32,
    page: usize,
}

#[derive(Deserialize)]
struct RawRegion {
    address_name: String,
    region_1depth_name: String,
    region_2depth_name: String,
    region_3depth_name: String,
    region_4depth_name: String,
    code: String,
    x: f32,
    y: f32,
}

#[derive(Clone)]
pub struct Region {
    pub address: String,
    pub province: String,
    pub city: String,
    pub town: String,
    pub neighborhood: String,
    pub code: Option<usize>,
    pub longitude: Option<f32>,
    pub latitude: Option<f32>,
}

#[derive(Deserialize)]
struct RawLandLotAddress {
    address_name: String,
    region_1depth_name: String,
    region_2depth_name: String,
    region_3depth_name: String,
    mountain_yn: String,
    main_address_no: String,
    sub_address_no: String,
    zip_code: String,
}

#[derive(Deserialize)]
struct RawRoadAddress {
    address_name: String,
    region_1depth_name: String,
    region_2depth_name: String,
    region_3depth_name: String,
    road_name: String,
    underground_yn: String,
    main_building_no: String,
    sub_building_no: String,
    building_name: String,
    zone_no: String,
}

impl CoordRequest {
    pub fn new(app_key: &str, longitude: f32, latitude: f32) -> Self {
        CoordRequest {
            app_key: app_key.to_owned(),
            longitude,
            latitude,
            page: 1,
        }
    }

    fn to_url_with_base(&self, base_url: &str) -> String {
        let mut s = String::from(base_url);
        s = s + "?x=" + &self.longitude.to_string();
        s = s + "&y=" + &self.latitude.to_string();
        s = s + "&page=" + &self.page.to_string();
        s
    }

    pub fn get_region(&self) -> Response<Region, Self> {
        Req::<Region>::get(self)
    }

    pub fn get_address(&self) -> Response<Address, Self> {
        Req::<Address>::get(self)
    }
}

impl ReqOnce<Region> for CoordRequest {
    fn page(&mut self, page: usize) -> &mut Self {
        self.page = page;
        self
    }

    fn request(&self) -> Result<Vec<Region>> {
        request(
            &self.to_url_with_base("https://dapi.kakao.com/v2/local/geo/coord2regioncode.json"),
            &self.app_key,
        ).and_then(|v| serde_json::from_value::<Coord2RegionResponse>(v).map_err(|e| e.into()))
            .map(|r| r.documents.into_iter().map(|r| r.into()).collect())
    }
}

impl ReqOnce<Address> for CoordRequest {
    fn page(&mut self, page: usize) -> &mut Self {
        self.page = page;
        self
    }

    fn request(&self) -> Result<Vec<Address>> {
        request(
            &self.to_url_with_base("https://dapi.kakao.com/v2/local/geo/coord2address.json"),
            &self.app_key,
        ).and_then(|v| serde_json::from_value::<Coord2AddressResponse>(v).map_err(|e| e.into()))
            .map(|r| {
                r.documents
                    .into_iter()
                    .map(|d| Address {
                        land_lot: d.address.map(|r| r.into()),
                        road: d.road_address.map(|r| r.into()),
                    })
                    .collect()
            })
    }
}

impl Req<Region> for CoordRequest {}

impl Req<Address> for CoordRequest {}

impl From<RawRegion> for Region {
    fn from(rreg: RawRegion) -> Self {
        Region {
            address: rreg.address_name,
            province: rreg.region_1depth_name,
            city: rreg.region_2depth_name,
            town: rreg.region_3depth_name,
            neighborhood: rreg.region_4depth_name,
            code: rreg.code.parse::<usize>().ok(),
            longitude: Some(rreg.x),
            latitude: Some(rreg.y),
        }
    }
}

impl Into<LandLotAddress> for RawLandLotAddress {
    fn into(self) -> LandLotAddress {
        LandLotAddress {
            address: self.address_name,
            province: self.region_1depth_name,
            city: self.region_2depth_name,
            town: self.region_3depth_name,
            neighborhood: None,
            h_code: None,
            b_code: None,
            is_mountain: if self.mountain_yn.is_empty() {
                None
            } else {
                Some(self.mountain_yn == "Y")
            },
            main_address_number: self.main_address_no.parse::<usize>().ok(),
            sub_address_number: self.sub_address_no.parse::<usize>().ok(),
            zip_code: self.zip_code.parse::<usize>().ok(),
            longitude: None,
            latitude: None,
        }
    }
}

impl Into<RoadAddress> for RawRoadAddress {
    fn into(self) -> RoadAddress {
        RoadAddress {
            address: self.address_name,
            province: self.region_1depth_name,
            city: self.region_2depth_name,
            town: self.region_3depth_name,
            road_name: self.road_name,
            is_underground: self.underground_yn == "Y",
            main_building_number: self.main_building_no.parse::<usize>().ok(),
            sub_building_number: self.sub_building_no.parse::<usize>().ok(),
            building_name: self.building_name,
            post_code: self.zone_no.parse::<usize>().ok(),
            longitude: None,
            latitude: None,
        }
    }
}

impl Hash for Region {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address.hash(state);
        self.code.hash(state);
    }
}

impl PartialEq for Region {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address && self.code == other.code
    }
}

impl Eq for Region {}

impl Element for Region {}
