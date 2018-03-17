extern crate daummap;

use std::env::vars;

fn get_key() -> String {
    let v = vars();
    v.filter_map(|(key, value)| if key == "APP_KEY" { Some(value) } else { None })
        .next()
        .unwrap()
}

#[test]
fn check_key() {
    get_key();
}

#[test]
fn address() {
    let mut resp = daummap::AddressRequest::new(&get_key(), "전북 삼성동 100").get();
    let first = resp.next().unwrap().land_lot.unwrap();
    assert_eq!(&first.address, "전북 익산시 부송동 100");
    assert_eq!(first.zip_code, Some(570972));
}

#[test]
fn coord2region() {
    let mut resp = daummap::CoordRequest::new(&get_key(), 127.1086228, 37.4012191).get_region();
    let first = resp.next().unwrap();
    assert_eq!(&first.address, "경기도 성남시 분당구 삼평동");
    assert_eq!(first.code, Some(4113510900));
}

#[test]
fn coord2address() {
    let mut resp =
        daummap::CoordRequest::new(&get_key(), 127.423084873712, 37.0789561558879).get_address();
    let first = resp.next().unwrap().road.unwrap();
    assert_eq!(
        &first.address,
        "경기도 안성시 죽산면 죽산초교길 69-4"
    );
    assert_eq!(first.post_code, Some(17519));
}

#[test]
fn keyword() {
    let mut resp = daummap::KeywordRequest::new(&get_key(), "카카오프렌즈")
        .coord(127.06283102249932, 37.514322572335935)
        .radius(20000)
        .get();
    let first = resp.next().unwrap();
    assert_eq!(&first.name, "카카오프렌즈 코엑스점");
    assert_eq!(first.id, Some(26338954));
}

#[test]
fn category() {
    let mut resp = daummap::CategoryRequest::rect(
        &get_key(),
        daummap::CategoryGroup::Pharmacy,
        127.0561466,
        37.5058277,
        127.0602340,
        37.5142554,
    ).get();
    let first = resp.next().unwrap();
    assert_eq!(&first.name, "장생당약국");
    assert_eq!(first.id, Some(16618597));
}
