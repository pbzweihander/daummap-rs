//! # daummap
//!
//! Daum Map API wrapper
//!
//! # Examples
//!
//! ## Address Search
//!
//! ```
//! let resp = daummap::AddressRequest::new("APP_KEY", "전북 삼성동 100").get();
//! for addr in resp {
//!     println!("{}", addr.land_lot.unwrap().address);
//! }
//! ```
//!
//! ## Coord to Region
//!
//! ```
//! let resp = daummap::CoordRequest::new("APP_KEY", 127.1086228, 37.4012191).get_region();
//! for reg in resp {
//!     println!("{}", reg.address);
//! }
//! ```
//!
//! ## Coord to Address
//!
//! ```
//! let resp =
//!     daummap::CoordRequest::new("APP_KEY", 127.423084873712, 37.0789561558879).get_address();
//! for addr in resp {
//!     println!("{}", addr.road.unwrap().address);
//! }
//! ```
//!
//! ## Keyword Search
//!
//! ```
//! let resp = daummap::KeywordRequest::new("APP_KEY", "카카오프렌즈")
//!     .coord(127.06283102249932, 37.514322572335935)
//!     .radius(20000)
//!     .get();
//! for p in resp {
//!     println!("{}", p.name);
//! }
//! ```
//!
//! ## Category Search
//!
//! ```
//! let resp = daummap::CategoryRequest::rect(
//!     "APP_KEY",
//!     daummap::CategoryGroup::Pharmacy,
//!     127.0561466,
//!     37.5058277,
//!     127.0602340,
//!     37.5142554,
//! ).get();
//! for p in resp {
//!     println!("{}", p.name);
//! }
//! ```

#[macro_use]
extern crate error_chain;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

pub mod errors;
pub use errors::{Error, ErrorKind, Result};

pub mod address;
pub use address::{Address, AddressRequest, LandLotAddress, RoadAddress};

pub mod coord;
pub use coord::{CoordRequest, Region};

pub mod keyword;
pub use keyword::{KeywordRequest, Place};

pub mod category;
pub use category::{CategoryGroup, CategoryRequest};

use std::collections::HashSet;
use std::hash::Hash;
use std::cmp::Eq;
use std::marker::Sized;

#[derive(Clone)]
pub enum Sort {
    Distance,
    Accuracy,
}

pub trait Element
where
    Self: Sized + Hash + Eq + Clone,
{
}

pub trait ReqOnce<E>
where
    E: Element,
{
    fn to_url(&self) -> String;

    fn get_app_key(&self) -> &str;

    fn page(&mut self, page: usize) -> &mut Self;

    fn deserialize(value: serde_json::Value) -> Result<Vec<E>>;

    fn request(&self) -> Result<Vec<E>> {
        request(&self.to_url(), &self.get_app_key()).and_then(|v| Self::deserialize(v))
    }
}

pub trait Req<E>
where
    E: Element,
    Self: Sized + ReqOnce<E> + Clone,
{
    fn get(&self) -> Response<E, Self> {
        Response::new(self.clone())
    }
}

fn request(url: &str, key: &str) -> Result<serde_json::Value> {
    let resp = reqwest::Client::new()
        .get(url)
        .header(reqwest::header::Authorization(format!("KakaoAK {}", key)))
        .send()?;

    serde_json::from_reader(resp).map_err(|e| e.into())
}

pub struct Response<E, R>
where
    E: Element,
    R: Req<E>,
{
    req: R,
    set: HashSet<E>,
    buffer: Vec<E>,
    page: usize,
}

impl<E, R> Response<E, R>
where
    E: Element,
    R: Req<E>,
{
    fn new(req: R) -> Self {
        let mut i = Response {
            req,
            set: HashSet::new(),
            buffer: vec![],
            page: 1,
        };
        i.refresh();
        i
    }

    fn refresh(&mut self) {
        if let Ok(v) = self.req.page(self.page).request() {
            let mut buf: Vec<E> = vec![];
            for e in v {
                if !self.set.contains(&e) {
                    self.set.insert(e.clone());
                    buf.push(e);
                }
            }
            self.buffer = buf;
            self.page += 1;
        }
    }
}

impl<E, R> Iterator for Response<E, R>
where
    E: Element,
    R: Req<E>,
{
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            self.refresh();
            if self.buffer.is_empty() {
                None
            } else {
                Some(self.buffer.remove(0))
            }
        } else {
            Some(self.buffer.remove(0))
        }
    }
}
