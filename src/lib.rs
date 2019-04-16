//! # daummap
//!
//! Kakao Map API wrapper
//!
//! # Examples
//!
//! ## Address Search
//!
//! ```no_run
//! # use futures::prelude::*;
//! # #[allow(non_snake_case)]
//! # let APP_KEY = "";
//! let resp = daummap::AddressRequest::new(APP_KEY, "전북 삼성동 100")
//!     .get()
//!     .wait()
//!     .unwrap();
//! for addr in resp.addresses {
//!     println!("{}", addr.land_lot.unwrap().address);
//! }
//! ```
//!
//! ## Coord to Region
//!
//! ```no_run
//! # use futures::prelude::*;
//! # #[allow(non_snake_case)]
//! # let APP_KEY = "";
//! let resp = daummap::CoordRequest::new(APP_KEY, 127.1086228, 37.4012191)
//!     .get_region()
//!     .wait()
//!     .unwrap();
//! for reg in resp {
//!     println!("{}", reg.address);
//! }
//! ```
//!
//! ## Coord to Address
//!
//! ```no_run
//! # use futures::prelude::*;
//! # #[allow(non_snake_case)]
//! # let APP_KEY = "";
//! let resp = daummap::CoordRequest::new(APP_KEY, 127.423084873712, 37.0789561558879)
//!     .get_address()
//!     .wait()
//!     .unwrap();
//! for addr in resp {
//!     println!("{}", addr.road.unwrap().address);
//! }
//! ```
//!
//! ## Keyword Search
//!
//! ```no_run
//! # use futures::prelude::*;
//! # #[allow(non_snake_case)]
//! # let APP_KEY = "";
//! let resp = daummap::KeywordRequest::new(APP_KEY, "카카오프렌즈")
//!     .coord(127.06283102249932, 37.514322572335935)
//!     .radius(20000)
//!     .get()
//!     .wait()
//!     .unwrap();
//! for p in resp.places {
//!     println!("{}", p.name);
//! }
//! ```
//!
//! ## Category Search
//!
//! ```no_run
//! # use futures::prelude::*;
//! # #[allow(non_snake_case)]
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
//! .wait()
//! .unwrap();
//! for p in resp.places {
//!     println!("{}", p.name);
//! }
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
    futures::prelude::*,
    reqwest::{
        r#async::{Client, Response},
        Url,
    },
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

pub(crate) fn request<T: DeserializeOwned>(
    base_url: &str,
    path: &str,
    params: &[(&str, String)],
    key: &str,
) -> impl Future<Item = T, Error = failure::Error> {
    use futures::future::result;

    let key = key.to_string();

    result(
        Url::parse(base_url)
            .and_then(|base| base.join(path))
            .and_then(|url| Url::parse_with_params(url.as_str(), params))
            .map_err(Into::into),
    )
    .and_then(move |url| {
        Client::new()
            .get(url)
            .header("Authorization", format!("KakaoAK {}", key))
            .body("")
            .send()
            .and_then(Response::error_for_status)
            .and_then(|mut resp| resp.json())
            .map_err(Into::into)
    })
}

#[cfg(test)]
mod tests {
    use {
        crate::request,
        futures::prelude::*,
        hyper::{
            header::HeaderValue,
            service::{make_service_fn, service_fn_ok},
            Body, Response, Server,
        },
        serde::Deserialize,
        tokio::runtime::Runtime,
    };

    #[derive(Deserialize)]
    struct Foo {
        bar: String,
    }

    #[test]
    fn test_request() {
        let (called_sender, called_receiver) = std::sync::mpsc::channel();
        let (shutdown_sender, shutdown_receiver) = futures::sync::oneshot::channel();

        let mut rt = Runtime::new().unwrap();

        let service = make_service_fn(move |_| {
            let called_sender = called_sender.clone();

            service_fn_ok(move |req| {
                let uri = req.uri();
                assert_eq!(uri.path(), "/foo/bar");
                assert_eq!(uri.query(), Some("baz=bax"));

                let headers = req.headers();
                assert_eq!(
                    headers.get("Authorization"),
                    Some(&HeaderValue::from_static("KakaoAK key"))
                );

                called_sender.send(()).unwrap();

                Response::<Body>::new(r#"{ "bar": "foobar" }"#.into())
            })
        });

        let server = Server::bind(&"0.0.0.0:12121".parse().unwrap())
            .serve(service)
            .with_graceful_shutdown(shutdown_receiver)
            .map_err(|why| panic!("{}", why));

        rt.spawn(server);

        let fut = request::<Foo>(
            "http://localhost:12121",
            "/foo/bar",
            &[("baz", "bax".to_string())],
            "key",
        )
        .inspect(|_| shutdown_sender.send(()).unwrap());

        let resp = rt.block_on_all(fut).unwrap();

        called_receiver.try_recv().unwrap();

        assert_eq!(&resp.bar, "foobar");
    }
}
