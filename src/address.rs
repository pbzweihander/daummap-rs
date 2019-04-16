use {
    crate::{request, Meta, KAKAO_LOCAL_API_BASE_URL},
    futures::prelude::*,
    serde::Deserialize,
};

#[derive(Debug, Clone)]
pub struct Address {
    pub address: Option<String>,
    pub land_lot: Option<LandLotAddress>,
    pub road: Option<RoadAddress>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct AddressResponse {
    pub addresses: Vec<Address>,
    pub total_count: usize,
    pub pageable_count: usize,
    pub is_end: bool,
}

#[derive(Debug, Clone)]
pub struct AddressRequest {
    base_url: String,
    app_key: String,
    query: String,
    page: usize,
    size: usize,
}

impl AddressRequest {
    pub fn new(app_key: &str, query: &str) -> Self {
        AddressRequest {
            base_url: KAKAO_LOCAL_API_BASE_URL.to_string(),
            app_key: app_key.to_string(),
            query: query.to_string(),
            page: 1,
            size: 15,
        }
    }

    pub fn base_url(&mut self, base_url: &str) -> &mut Self {
        self.base_url = base_url.to_string();
        self
    }

    pub fn page(&mut self, page: usize) -> &mut Self {
        self.page = page;
        self
    }

    pub fn size(&mut self, size: usize) -> &mut Self {
        self.size = size;
        self
    }

    pub fn get(&self) -> impl Future<Item = AddressResponse, Error = failure::Error> {
        static API_PATH: &'static str = "/search/address.json";

        request::<RawResponse>(
            &self.base_url,
            API_PATH,
            &[
                ("query", self.query.clone()),
                ("page", self.page.to_string()),
                ("size", self.size.to_string()),
            ],
            &self.app_key,
        )
        .map(|resp| {
            let addresses = resp
                .documents
                .into_iter()
                .map(|document| Address {
                    address: document.address_name,
                    land_lot: document.address.map(Into::into),
                    road: document.road_address.map(Into::into),
                })
                .filter(|addr| addr.land_lot.is_some() || addr.road.is_some())
                .collect();

            AddressResponse {
                addresses,
                total_count: resp.meta.total_count,
                pageable_count: resp.meta.pageable_count,
                is_end: resp.meta.is_end,
            }
        })
    }
}

#[derive(Debug, Deserialize)]
struct RawResponse {
    documents: Vec<Document>,
    meta: Meta,
}

#[derive(Debug, Deserialize)]
struct Document {
    address_name: Option<String>,
    address: Option<RawLandLotAddress>,
    road_address: Option<RawRoadAddress>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
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
    x: String,
    y: String,
}

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
            is_underground: raddr.underground_yn == "Y",
            main_building_number: raddr.main_building_no.parse::<usize>().ok(),
            sub_building_number: raddr.sub_building_no.parse::<usize>().ok(),
            building_name: raddr.building_name,
            post_code: raddr.zone_no.parse::<usize>().ok(),
            longitude: raddr.x.parse::<f32>().ok(),
            latitude: raddr.y.parse::<f32>().ok(),
        }
    }
}
