//! # daummap
//!
//! Daum Map API wrapper
//!
//! # Examples
//!
//! ## Address Search
//!
//! ```
//! # use futures::prelude::*;
//! # dotenv::dotenv();
//! # let key = std::env::var("APP_KEY").unwrap();
//! # #[allow(non_snake_case)]
//! # let APP_KEY = &key;
//! let resp = daummap::AddressRequest::new(APP_KEY, "전북 삼성동 100").get();
//! for addr in resp.wait() {
//!     println!("{}", addr.unwrap().land_lot.unwrap().address);
//! }
//! ```
//!
//! ## Coord to Region
//!
//! ```
//! # use futures::prelude::*;
//! # dotenv::dotenv();
//! # let key = std::env::var("APP_KEY").unwrap();
//! # #[allow(non_snake_case)]
//! # let APP_KEY = &key;
//! let resp = daummap::CoordRequest::new(APP_KEY, 127.1086228, 37.4012191).get_region();
//! for reg in resp.wait() {
//!     println!("{}", reg.unwrap().address);
//! }
//! ```
//!
//! ## Coord to Address
//!
//! ```
//! # use futures::prelude::*;
//! # dotenv::dotenv();
//! # let key = std::env::var("APP_KEY").unwrap();
//! # #[allow(non_snake_case)]
//! # let APP_KEY = &key;
//! let resp =
//!     daummap::CoordRequest::new(APP_KEY, 127.423084873712, 37.0789561558879).get_address();
//! for addr in resp.wait() {
//!     println!("{}", addr.unwrap().road.unwrap().address);
//! }
//! ```
//!
//! ## Keyword Search
//!
//! ```
//! # use futures::prelude::*;
//! # dotenv::dotenv();
//! # let key = std::env::var("APP_KEY").unwrap();
//! # #[allow(non_snake_case)]
//! # let APP_KEY = &key;
//! let resp = daummap::KeywordRequest::new(APP_KEY, "카카오프렌즈")
//!     .coord(127.06283102249932, 37.514322572335935)
//!     .radius(20000)
//!     .get();
//! for p in resp.wait() {
//!     println!("{}", p.unwrap().name);
//! }
//! ```
//!
//! ## Category Search
//!
//! ```
//! # use futures::prelude::*;
//! # dotenv::dotenv();
//! # let key = std::env::var("APP_KEY").unwrap();
//! # #[allow(non_snake_case)]
//! # let APP_KEY = &key;
//! let resp = daummap::CategoryRequest::rect(
//!     APP_KEY,
//!     daummap::CategoryGroup::Pharmacy,
//!     127.0561466,
//!     37.5058277,
//!     127.0602340,
//!     37.5142554,
//! ).get();
//! for p in resp.wait() {
//!     println!("{}", p.unwrap().name);
//! }
//! ```

pub mod address;
pub mod category;
pub mod coord;
pub mod errors;
pub mod keyword;

pub use crate::{
    address::{Address, AddressRequest, LandLotAddress, RoadAddress},
    category::{CategoryGroup, CategoryRequest},
    coord::{CoordRequest, Region},
    errors::ParseCategoryGroup,
    keyword::{KeywordRequest, Place},
};

use {
    failure::Error,
    futures::{prelude::*, Async, Poll},
    hyper::{Body, Client, Request},
    hyper_tls::HttpsConnector,
    std::{cmp::Eq, collections::HashSet, hash::Hash, marker::Sized},
};

pub(crate) fn encode(s: &str) -> String {
    use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
    utf8_percent_encode(s, DEFAULT_ENCODE_SET).to_string()
}

#[derive(Clone)]
pub enum Sort {
    Distance,
    Accuracy,
}

pub trait Element
where
    Self: Sized + Hash + Eq + Clone + Send,
{
}

pub trait ReqOnce<E>
where
    E: Element + 'static,
{
    fn to_url(&self, page: usize) -> String;

    fn get_app_key(&self) -> &str;

    fn deserialize(value: serde_json::Value) -> Result<Vec<E>, Error>;

    fn request(&self, page: usize) -> Box<dyn Future<Item = Vec<E>, Error = Error> + Send> {
        use futures::future::result;
        Box::new(
            request(&self.to_url(page), &self.get_app_key())
                .and_then(|v| result(Self::deserialize(v))),
        )
    }
}

pub trait Req<E>
where
    E: Element + 'static,
    Self: Sized + ReqOnce<E> + Clone + 'static,
{
    fn get(&self) -> Response<E, Self> {
        Response::new(self.clone())
    }
}

fn request(url: &str, key: &str) -> impl Future<Item = serde_json::Value, Error = Error> {
    use futures::future::result;
    result(
        HttpsConnector::new(2)
            .map_err(Into::<Error>::into)
            .and_then(|https| {
                Request::builder()
                    .method("GET")
                    .uri(url)
                    .header("Authorization", format!("KakaoAK {}", key))
                    .body(Body::empty())
                    .map(|req| (https, req))
                    .map_err(Into::into)
            }),
    )
    .and_then(|(https, req)| {
        let client = Client::builder().build::<_, Body>(https);
        client.request(req).map_err(Into::into)
    })
    .and_then(|resp| resp.into_body().concat2().map_err(Into::into))
    .and_then(|body| result(serde_json::from_slice(&body).map_err(Into::into)))
}

pub struct Response<E, R>
where
    E: Element + 'static,
    R: Req<E>,
{
    req: R,
    set: HashSet<E>,
    buffer: Vec<E>,
    page: usize,
    fut: Option<Box<dyn Future<Item = Vec<E>, Error = Error>>>,
}

impl<E, R> Response<E, R>
where
    E: Element + 'static,
    R: Req<E> + 'static,
{
    fn new(req: R) -> Self {
        Response {
            req,
            set: HashSet::new(),
            buffer: vec![],
            page: 1,
            fut: None,
        }
    }
}

impl<E, R> Stream for Response<E, R>
where
    E: Element + 'static,
    R: Req<E> + 'static,
{
    type Item = E;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        use {
            futures::{
                future::{ok, result},
                sync::oneshot::channel,
            },
            std::thread::spawn,
            tokio::run,
        };

        if self.buffer.is_empty() {
            let req = &self.req;
            let page = self.page;
            let fut = self.fut.get_or_insert_with(|| {
                let (sender, receiver) = channel();
                let fut = req.request(page);
                spawn(|| {
                    run(fut.then(|r| {
                        sender.send(r).ok();
                        ok(())
                    }))
                });

                Box::new(receiver.map_err(Into::into).and_then(result))
            });
            let poll = fut.poll();
            match poll {
                Ok(Async::Ready(v)) => {
                    self.fut = None;
                    for e in v {
                        if !self.set.contains(&e) {
                            self.set.insert(e.clone());
                            self.buffer.push(e);
                        }
                    }
                    self.page += 1;
                    if self.buffer.is_empty() {
                        Ok(Async::Ready(None))
                    } else {
                        Ok(Async::Ready(Some(self.buffer.remove(0))))
                    }
                }
                Ok(Async::NotReady) => Ok(Async::NotReady),
                Err(e) => Err(e),
            }
        } else {
            Ok(Async::Ready(Some(self.buffer.remove(0))))
        }
    }
}
