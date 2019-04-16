use {
    crate::{request, Address, LandLotAddress, RoadAddress, KAKAO_LOCAL_API_BASE_URL},
    futures::prelude::*,
    serde::{de::DeserializeOwned, Deserialize},
};

#[derive(Debug, Clone)]
pub struct CoordRequest {
    base_url: String,
    app_key: String,
    page: usize,
    longitude: f32,
    latitude: f32,
}

impl CoordRequest {
    pub fn new(app_key: &str, longitude: f32, latitude: f32) -> Self {
        CoordRequest {
            base_url: KAKAO_LOCAL_API_BASE_URL.to_string(),
            app_key: app_key.to_string(),
            page: 1,
            longitude,
            latitude,
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

    fn request<T: DeserializeOwned>(
        &self,
        api_path: &str,
    ) -> impl Future<Item = T, Error = failure::Error> {
        request::<T>(
            &self.base_url,
            api_path,
            &[
                ("page", self.page.to_string()),
                ("x", self.longitude.to_string()),
                ("y", self.latitude.to_string()),
            ],
            &self.app_key,
        )
    }

    pub fn get_region(&self) -> impl Future<Item = Vec<Region>, Error = failure::Error> {
        static API_PATH: &'static str = "/geo/coord2regioncode.json";

        self.request::<Coord2RegionResponse>(API_PATH)
            .map(|resp| resp.documents.into_iter().map(Into::into).collect())
    }

    pub fn get_address(&self) -> impl Future<Item = Vec<Address>, Error = failure::Error> {
        static API_PATH: &'static str = "/geo/coord2address.json";

        self.request::<Coord2AddressResponse>(API_PATH).map(|resp| {
            resp.documents
                .into_iter()
                .map(|document| Address {
                    address: None,
                    land_lot: document.address.map(Into::into),
                    road: document.road_address.map(Into::into),
                })
                .collect()
        })
    }
}

#[derive(Debug, Deserialize)]
struct Coord2AddressResponse {
    documents: Vec<Coord2AddressDocument>,
}

#[derive(Debug, Deserialize)]
struct Coord2RegionResponse {
    documents: Vec<RawRegion>,
}

#[derive(Debug, Deserialize)]
struct Coord2AddressDocument {
    address: Option<RawLandLotAddress>,
    road_address: Option<RawRoadAddress>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Deserialize)]
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
}

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
