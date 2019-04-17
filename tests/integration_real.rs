#![recursion_limit = "128"]
#![allow(clippy::unreadable_literal, clippy::excessive_precision)]

use {dotenv::dotenv, failure::Fallible, std::env::var, tokio::runtime::Runtime};

fn get_key() -> Fallible<String> {
    dotenv().ok();
    let key = var("DAUMMAP_APP_KEY")?;
    Ok(key)
}

macro_rules! get_key {
    () => {
        match get_key() {
            Ok(key) => key,
            Err(_) => return Ok(()),
        }
    };
}

#[test]
fn test_address_real() -> Fallible<()> {
    let mut rt = Runtime::new()?;
    let resp =
        rt.block_on(daummap::AddressRequest::new(&get_key!(), "전북 삼성동 100").get())?;
    let addr = resp.addresses[0].land_lot.as_ref().unwrap();
    assert_eq!(&addr.address, "전북 익산시 부송동 100",);
    assert_eq!(addr.zip_code, Some(570972));
    Ok(())
}

#[test]
fn test_coord2region_real() -> Fallible<()> {
    let mut rt = Runtime::new()?;
    let resp =
        rt.block_on(daummap::CoordRequest::new(&get_key!(), 127.1086228, 37.4012191).get_region())?;
    let addr = &resp[0];
    assert_eq!(&addr.address, "경기도 성남시 분당구 삼평동");
    assert_eq!(addr.code, Some(4113510900));
    Ok(())
}

#[test]
fn test_coord2address_real() -> Fallible<()> {
    let mut rt = Runtime::new()?;
    let resp = rt.block_on(
        daummap::CoordRequest::new(&get_key!(), 127.423084873712, 37.0789561558879).get_address(),
    )?;
    let addr = resp[0].road.as_ref().unwrap();
    assert_eq!(
        &addr.address,
        "경기도 안성시 죽산면 죽산초교길 69-4"
    );
    assert_eq!(addr.post_code, Some(17519));
    Ok(())
}

#[test]
fn test_keyword_real() -> Fallible<()> {
    let mut rt = Runtime::new()?;
    let resp = rt.block_on(
        daummap::KeywordRequest::new(&get_key!(), "카카오프렌즈")
            .coord(127.06283102249932, 37.514322572335935)
            .radius(20000)
            .get(),
    )?;
    let place = &resp.places[0];
    assert_eq!(
        &place.name,
        "카카오프렌즈 스타필드 코엑스몰점"
    );
    assert_eq!(place.id, Some(26338954));
    Ok(())
}

#[test]
fn test_category_real() -> Fallible<()> {
    let mut rt = Runtime::new()?;
    let resp = rt.block_on(
        daummap::CategoryRequest::rect(
            &get_key!(),
            daummap::CategoryGroup::Pharmacy,
            127.0561466,
            37.5058277,
            127.0602340,
            37.5142554,
        )
        .get(),
    )?;
    let place = &resp.places[0];
    assert_eq!(&place.name, "삼성수약국");
    assert_eq!(place.id, Some(16059439));
    Ok(())
}
