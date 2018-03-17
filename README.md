# daummap-rs

[![circleci](https://circleci.com/gh/pbzweihander/daummap-rs.svg?style=shield)](https://circleci.com/gh/pbzweihander/daumdic-rs)
[![crate.io](https://img.shields.io/crates/v/daummap.svg)](https://crates.io/crates/daummap)
[![docs.rs](https://docs.rs/daummap/badge.svg)](https://docs.rs/daummap)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE-MIT)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE-APACHE)

[Daum Map API](https://developers.kakao.com/docs/restapi/local) wrapper written in Rust.

```rust
extern crate daummap;

fn main() {
    let resp = daummap::KeywordRequest::new(&get_key(), "카카오프렌즈")
        .coord(127.06283102249932, 37.514322572335935)
        .radius(20000)
        .get();
    for place in resp {
        println!("{}", place.name);
    }
}
```
