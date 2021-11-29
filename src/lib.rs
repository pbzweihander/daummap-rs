//! # daummap
//!
//! Kakao Map API wrapper
//!
//! # Examples
//!
//! ## Address Search
//!
//! ```no_run
//! # #[allow(non_snake_case)]
//! # async fn foo() {
//! # let APP_KEY = "";
//! let resp = daummap::AddressRequest::new(APP_KEY, "전북 삼성동 100")
//!     .get()
//!     .await
//!     .unwrap();
//! for addr in resp.addresses {
//!     println!("{}", addr.land_lot.unwrap().address);
//! }
//! # }
//! ```
//!
//! ## Coord to Region
//!
//! ```no_run
//! # #[allow(non_snake_case)]
//! # async fn foo() {
//! # let APP_KEY = "";
//! let resp = daummap::CoordRequest::new(APP_KEY, 127.1086228, 37.4012191)
//!     .get_region()
//!     .await
//!     .unwrap();
//! for reg in resp {
//!     println!("{}", reg.address);
//! }
//! # }
//! ```
//!
//! ## Coord to Address
//!
//! ```no_run
//! # #[allow(non_snake_case)]
//! # async fn foo() {
//! # let APP_KEY = "";
//! let resp = daummap::CoordRequest::new(APP_KEY, 127.423084873712, 37.0789561558879)
//!     .get_address()
//!     .await
//!     .unwrap();
//! for addr in resp {
//!     println!("{}", addr.road.unwrap().address);
//! }
//! # }
//! ```
//!
//! ## Keyword Search
//!
//! ```no_run
//! # #[allow(non_snake_case)]
//! # async fn foo() {
//! # let APP_KEY = "";
//! let resp = daummap::KeywordRequest::new(APP_KEY, "카카오프렌즈")
//!     .coord(127.06283102249932, 37.514322572335935)
//!     .radius(20000)
//!     .get()
//!     .await
//!     .unwrap();
//! for p in resp.places {
//!     println!("{}", p.name);
//! }
//! # }
//! ```
//!
//! ## Category Search
//!
//! ```no_run
//! # #[allow(non_snake_case)]
//! # async fn foo() {
//! # let APP_KEY = "";
//! let resp = daummap::CategoryRequest::rect(
//!     APP_KEY,
//!     daummap::CategoryGroup::Pharmacy,
//!     127.0561466,
//!     37.5058277,
//!     127.0602340,
//!     37.5142554,
//! )
//! .get()
//! .await
//! .unwrap();
//! for p in resp.places {
//!     println!("{}", p.name);
//! }
//! # }
//! ```

pub mod address;
pub mod category;
pub mod coord;
pub mod keyword;

pub use crate::{
    address::{Address, AddressRequest, AddressResponse, LandLotAddress, RoadAddress},
    category::{CategoryGroup, CategoryRequest, CategoryResponse},
    coord::{CoordRequest, Region},
    keyword::{KeywordRequest, KeywordResponse, Place},
};

use {
    reqwest::{Client, Url},
    serde::{de::DeserializeOwned, Deserialize},
};

pub(crate) static KAKAO_LOCAL_API_BASE_URL: &'static str = "https://dapi.kakao.com/v2/local";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sort {
    Distance,
    Accuracy,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Meta {
    total_count: usize,
    pageable_count: usize,
    is_end: bool,
}

impl ToString for Sort {
    fn to_string(&self) -> String {
        match self {
            Sort::Distance => "distance".to_string(),
            Sort::Accuracy => "accuracy".to_string(),
        }
    }
}

pub(crate) async fn request<T: DeserializeOwned>(
    base_url: &str,
    path: &str,
    params: &[(&str, String)],
    key: &str,
) -> Result<T, failure::Error> {
    let base_url = if base_url.ends_with('/') {
        base_url.to_string()
    } else {
        base_url.to_string() + "/"
    };
    let path = if path.starts_with('/') {
        path.trim_start_matches('/').to_string()
    } else {
        path.to_string()
    };

    let key = key.to_string();

    let url = Url::parse(&base_url)
        .and_then(|base| base.join(&path))
        .and_then(|url| Url::parse_with_params(url.as_str(), params))?;
    let resp = Client::new()
        .get(url)
        .header("Authorization", format!("KakaoAK {}", key))
        .body("")
        .send()
        .await?;
    let resp = resp.error_for_status()?;
    Ok(resp.json().await?)
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use {
        crate::request,
        hyper::{
            header::HeaderValue,
            service::{make_service_fn, service_fn},
            Body, Response, Server,
        },
        serde::Deserialize,
    };

    #[derive(Deserialize)]
    struct Foo {
        bar: String,
    }

    #[tokio::test]
    async fn test_request() {
        let (called_sender, mut called_receiver) = tokio::sync::mpsc::channel(2);
        let (shutdown_sender, shutdown_receiver) = tokio::sync::oneshot::channel();

        let service = make_service_fn(move |_| {
            let called_sender = called_sender.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    let called_sender = called_sender.clone();
                    async move {
                        let uri = req.uri();
                        assert_eq!(uri.path(), "/api/foo/bar");
                        assert_eq!(uri.query(), Some("baz=bax"));

                        let headers = req.headers();
                        assert_eq!(
                            headers.get("Authorization"),
                            Some(&HeaderValue::from_static("KakaoAK key"))
                        );

                        called_sender.send(()).await.unwrap();

                        Ok::<_, Infallible>(Response::<Body>::new(r#"{ "bar": "foobar" }"#.into()))
                    }
                }))
            }
        });

        let server = Server::bind(&"0.0.0.0:12121".parse().unwrap())
            .serve(service)
            .with_graceful_shutdown(async {
                shutdown_receiver.await.unwrap();
            });

        tokio::spawn(async {
            if let Err(e) = server.await {
                panic!("{}", e)
            }
        });

        let resp = request::<Foo>(
            "http://localhost:12121/api",
            "/foo/bar",
            &[("baz", "bax".to_string())],
            "key",
        )
        .await
        .unwrap();

        shutdown_sender.send(()).unwrap();
        called_receiver.try_recv().unwrap();

        assert_eq!(&resp.bar, "foobar");
    }
}
