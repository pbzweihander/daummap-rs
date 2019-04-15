use {
    crate::{request, Meta, Place, Sort, KAKAO_LOCAL_API_BASE_URL},
    failure::{Fail, Fallible},
    futures::prelude::*,
    reqwest::Url,
    serde::Deserialize,
    std::str::FromStr,
};

#[derive(Debug, Clone)]
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

impl FromStr for CategoryGroup {
    type Err = failure::Error;

    fn from_str(s: &str) -> Fallible<Self> {
        use crate::CategoryGroup::*;

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
            _ => return Err(ParseCategoryGroup(s.to_string()).into()),
        })
    }
}

impl CategoryGroup {
    pub fn to_code<'a>(&self) -> &'a str {
        use crate::CategoryGroup::*;
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

#[derive(Debug, Fail)]
#[fail(display = "Cannot parse category group from {}", _0)]
pub struct ParseCategoryGroup(pub String);

#[derive(Debug, Clone)]
pub struct CategoryResponse {
    pub places: Vec<Place>,
    pub total_count: usize,
    pub pageable_count: usize,
    pub is_end: bool,
}

#[derive(Debug, Clone)]
pub struct CategoryRequest {
    base_url: String,
    app_key: String,
    category_group: CategoryGroup,
    longitude: Option<f32>,
    latitude: Option<f32>,
    radius: Option<usize>,
    rect: Option<(f32, f32, f32, f32)>,
    page: usize,
    size: usize,
    sort: Sort,
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
            base_url: KAKAO_LOCAL_API_BASE_URL.to_string(),
            app_key: app_key.to_string(),
            category_group,
            longitude: Some(longitude),
            latitude: Some(latitude),
            radius: Some(radius),
            rect: None,
            page: 1,
            size: 15,
            sort: Sort::Accuracy,
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
            base_url: KAKAO_LOCAL_API_BASE_URL.to_string(),
            app_key: app_key.to_string(),
            category_group,
            longitude: None,
            latitude: None,
            radius: None,
            rect: Some((x1, y1, x2, y2)),
            page: 1,
            size: 15,
            sort: Sort::Accuracy,
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

    pub fn sort(&mut self, sort: Sort) -> &mut Self {
        self.sort = sort;
        self
    }

    pub fn get(&self) -> impl Future<Item = CategoryResponse, Error = failure::Error> {
        static API_PATH: &'static str = "/search/category.json";

        use futures::future::result;

        let app_key = self.app_key.clone();

        let mut params = vec![
            (
                "category_group_code",
                self.category_group.to_code().to_string(),
            ),
            ("page", self.page.to_string()),
            ("size", self.size.to_string()),
            ("sort", self.sort.to_string()),
        ];

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

        result(
            Url::parse(&self.base_url)
                .and_then(|base| base.join(API_PATH))
                .and_then(|url| Url::parse_with_params(url.as_str(), &params))
                .map_err(Into::into),
        )
        .and_then(move |url| request::<RawResponse>(url, &app_key))
        .map(|resp| {
            let places = resp.documents.into_iter().map(Into::into).collect();

            CategoryResponse {
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
