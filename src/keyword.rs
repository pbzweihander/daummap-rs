use {
    crate::{request, CategoryGroup, Meta, Sort, KAKAO_LOCAL_API_BASE_URL},
    futures::prelude::*,
    serde::Deserialize,
};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct KeywordResponse {
    pub places: Vec<Place>,
    pub total_count: usize,
    pub pageable_count: usize,
    pub is_end: bool,
}

#[derive(Debug, Clone)]
pub struct KeywordRequest {
    base_url: String,
    app_key: String,
    query: String,
    category_group: Option<CategoryGroup>,
    longitude: Option<f32>,
    latitude: Option<f32>,
    radius: Option<usize>,
    rect: Option<(f32, f32, f32, f32)>,
    page: usize,
    size: usize,
    sort: Sort,
}

impl KeywordRequest {
    pub fn new(app_key: &str, query: &str) -> Self {
        KeywordRequest {
            base_url: KAKAO_LOCAL_API_BASE_URL.to_string(),
            app_key: app_key.to_string(),
            query: query.to_string(),
            category_group: None,
            longitude: None,
            latitude: None,
            radius: None,
            rect: None,
            page: 1,
            size: 15,
            sort: Sort::Accuracy,
        }
    }

    pub fn base_url(&mut self, base_url: &str) -> &mut Self {
        self.base_url = base_url.to_string();
        self
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

    pub fn page(&mut self, page: usize) -> &mut Self {
        self.page = page;
        self
    }

    pub fn size(&mut self, size: usize) -> &mut Self {
        self.size = size;
        self
    }

    pub fn sort(&mut self, sort: Sort) -> &mut Self {
        self.sort = sort;
        self
    }

    pub fn get(&self) -> impl Future<Item = KeywordResponse, Error = failure::Error> {
        static API_PATH: &'static str = "/search/keyword.json";

        let mut params = vec![
            ("query", self.query.clone()),
            ("page", self.page.to_string()),
            ("size", self.size.to_string()),
            ("sort", self.sort.to_string()),
        ];

        if let Some(ref category_group) = self.category_group {
            params.push(("category_group_code", category_group.to_code().to_string()));
        }
        if let Some(x) = self.longitude {
            params.push(("x", x.to_string()));
        }
        if let Some(y) = self.latitude {
            params.push(("y", y.to_string()));
        }
        if let Some(radius) = self.radius {
            params.push(("radius", radius.to_string()));
        }
        if let Some((x1, y1, x2, y2)) = self.rect {
            params.push(("rect", format!("{},{},{},{}", x1, y1, x2, y2)));
        }

        request::<RawResponse>(&self.base_url, API_PATH, &params, &self.app_key).map(|resp| {
            let places = resp.documents.into_iter().map(Into::into).collect();

            KeywordResponse {
                places,
                total_count: resp.meta.total_count,
                pageable_count: resp.meta.pageable_count,
                is_end: resp.meta.is_end,
            }
        })
    }
}

#[derive(Debug, Deserialize)]
struct RawResponse {
    documents: Vec<RawPlace>,
    meta: Meta,
}

#[derive(Debug, Deserialize)]
struct RawPlace {
    id: String,
    place_name: String,
    category_group_name: String,
    category_group_code: String,
    phone: String,
    address_name: String,
    road_address_name: String,
    x: String,
    y: String,
    place_url: String,
    distance: String,
}

impl From<RawPlace> for Place {
    fn from(raddr: RawPlace) -> Self {
        Place {
            id: raddr.id.parse::<usize>().ok(),
            name: raddr.place_name,
            category: raddr.category_group_name,
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
