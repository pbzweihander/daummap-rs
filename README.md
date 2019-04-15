# daummap-rs

[![circleci badge]][circleci]
[![crates.io badge]][crates.io]
[![docs.rs badge]][docs.rs]
[![MIT License badge]](LICENSE-MIT)
[![Apache License badge]](LICENSE-APACHE)

[Kakao Map API](https://developers.kakao.com/docs/restapi/local) wrapper written in Rust.

```rust
use daummap;
use futures::prelude::*;

let resp = daummap::KeywordRequest::new(APP_KEY, "카카오프렌즈")
    .coord(127.06283102249932, 37.514322572335935)
    .radius(20000)
    .get()
    .wait()
    .unwrap();
for place in resp.addresses {
    println!("{}", place.name);
}
```

[circleci]: https://circleci.com/gh/pbzweihander/daummap-rs
[circleci badge]: https://circleci.com/gh/pbzweihander/daummap-rs.svg?style=shield
[crates.io]: https://crates.io/crates/daummap
[crates.io badge]: https://badgen.net/crates/v/daummap
[docs.rs]: https://docs.rs/daummap
[docs.rs badge]: https://docs.rs/daummap/badge.svg
[MIT License badge]: https://badgen.net/badge/license/MIT/blue
[Apache License badge]: https://badgen.net/badge/license/Apache-2.0/blue
