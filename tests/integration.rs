#![allow(clippy::unreadable_literal)]
#![allow(clippy::excessive_precision)]

use std::convert::Infallible;

use {
    daummap,
    hyper::{
        header::HeaderValue,
        service::{make_service_fn, service_fn},
        Body, Response, Server,
    },
};

#[tokio::test]
async fn test_address() {
    static RESP: &'static str = r#"{
  "meta": {
    "total_count": 4,
    "pageable_count": 4,
    "is_end": true
  },
  "documents": [
    {
      "address_name": "전북 익산시 부송동 100",
      "y": "35.97664845766847",
      "x": "126.99597295767953",
      "address_type": "REGION_ADDR",
      "address": {
        "address_name": "전북 익산시 부송동 100",
        "region_1depth_name": "전북",
        "region_2depth_name": "익산시",
        "region_3depth_name": "부송동",
        "region_3depth_h_name": "삼성동",
        "h_code": "4514069000",
        "b_code": "4514013400",
        "mountain_yn": "N",
        "main_address_no": "100",
        "sub_address_no": "",
        "zip_code": "570972",
        "x": "126.99597295767953",
        "y": "35.97664845766847"
      },
      "road_address": {
        "address_name": "전북 익산시 망산길 11-17",
        "region_1depth_name": "전북",
        "region_2depth_name": "익산시",
        "region_3depth_name": "부송동",
        "road_name": "망산길",
        "underground_yn": "N",
        "main_building_no": "11",
        "sub_building_no": "17",
        "building_name": "",
        "zone_no": "54547",
        "y": "35.976749396987046",
        "x": "126.99599512792346"
      }
    }
  ]
}"#;

    let (called_sender, called_receiver) = std::sync::mpsc::channel();
    let (shutdown_sender, shutdown_receiver) = tokio::sync::oneshot::channel();

    let service = make_service_fn(move |_| {
        let called_sender = called_sender.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let called_sender = called_sender.clone();
                async move {
                    let uri = req.uri();
                    assert_eq!(uri.path(), "/search/address.json");
                    assert_eq!(uri.query(), Some("query=address&page=2&size=5"));

                    let headers = req.headers();
                    assert_eq!(
                        headers.get("Authorization"),
                        Some(&HeaderValue::from_static("KakaoAK key"))
                    );

                    called_sender.send(()).unwrap();

                    Ok::<_, Infallible>(Response::<Body>::new(RESP.into()))
                }
            }))
        }
    });

    let server = Server::bind(&"127.0.0.1:12121".parse().unwrap())
        .serve(service)
        .with_graceful_shutdown(async { shutdown_receiver.await.unwrap() });

    tokio::spawn(async {
        if let Err(e) = server.await {
            panic!("{}", e);
        }
    });

    let resp = daummap::AddressRequest::new("key", "address")
        .base_url("http://localhost:12121")
        .page(2)
        .size(5)
        .get()
        .await
        .unwrap();

    shutdown_sender.send(()).unwrap();
    called_receiver.try_recv().unwrap();

    assert_eq!(resp.total_count, 4);
    assert_eq!(resp.pageable_count, 4);
    assert_eq!(resp.is_end, true);

    assert_eq!(resp.addresses.len(), 1);

    let address = &resp.addresses[0];

    assert_eq!(address.address, Some("전북 익산시 부송동 100".to_string()));

    assert!(address.land_lot.is_some());
    let land_lot = address.land_lot.as_ref().unwrap();
    assert_eq!(&land_lot.address, "전북 익산시 부송동 100");

    assert!(address.road.is_some());
    let road = address.road.as_ref().unwrap();
    assert_eq!(&road.address, "전북 익산시 망산길 11-17");
}

#[tokio::test]
async fn test_coord2region() {
    static RESP: &'static str = r#"{
  "meta": {
    "total_count": 2
  },
  "documents": [
    {
      "region_type": "B",
      "address_name": "경기도 성남시 분당구 삼평동",
      "region_1depth_name": "경기도",
      "region_2depth_name": "성남시 분당구",
      "region_3depth_name": "삼평동",
      "region_4depth_name": "",
      "code": "4113510900",
      "x": 127.10459896729914,
      "y": 37.40269721785548
    },
    {
      "region_type": "H",
      "address_name": "경기도 성남시 분당구 삼평동",
      "region_1depth_name": "경기도",
      "region_2depth_name": "성남시 분당구",
      "region_3depth_name": "삼평동",
      "region_4depth_name": "",
      "code": "4113565500",
      "x": 127.1163593869371,
      "y": 37.40612091848614
    }
  ]
}"#;

    let (called_sender, called_receiver) = std::sync::mpsc::channel();
    let (shutdown_sender, shutdown_receiver) = tokio::sync::oneshot::channel();

    let service = make_service_fn(move |_| {
        let called_sender = called_sender.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let called_sender = called_sender.clone();
                async move {
                    let uri = req.uri();
                    assert_eq!(uri.path(), "/geo/coord2regioncode.json");
                    assert_eq!(uri.query(), Some("page=2&x=123.123&y=456.456"));

                    let headers = req.headers();
                    assert_eq!(
                        headers.get("Authorization"),
                        Some(&HeaderValue::from_static("KakaoAK key"))
                    );

                    called_sender.send(()).unwrap();

                    Ok::<_, Infallible>(Response::<Body>::new(RESP.into()))
                }
            }))
        }
    });

    let server = Server::bind(&"127.0.0.1:12122".parse().unwrap())
        .serve(service)
        .with_graceful_shutdown(async { shutdown_receiver.await.unwrap() });

    tokio::spawn(async {
        if let Err(e) = server.await {
            panic!("{}", e);
        }
    });

    let resp = daummap::CoordRequest::new("key", 123.123, 456.456)
        .base_url("http://localhost:12122")
        .page(2)
        .get_region()
        .await
        .unwrap();

    shutdown_sender.send(()).unwrap();
    called_receiver.try_recv().unwrap();

    assert_eq!(resp.len(), 2);
    assert_eq!(&resp[0].address, "경기도 성남시 분당구 삼평동");
    assert_eq!(resp[0].code, Some(4113510900));
    assert_eq!(&resp[1].address, "경기도 성남시 분당구 삼평동");
    assert_eq!(resp[1].code, Some(4113565500));
}

#[tokio::test]
async fn test_coord2address() {
    static RESP: &'static str = r#"{
  "meta": {
    "total_count": 1
  },
  "documents": [
    {
      "road_address": {
        "address_name": "경기도 안성시 죽산면 죽산초교길 69-4",
        "region_1depth_name": "경기",
        "region_2depth_name": "안성시",
        "region_3depth_name": "죽산면",
        "road_name": "죽산초교길",
        "underground_yn": "N",
        "main_building_no": "69",
        "sub_building_no": "4",
        "building_name": "무지개아파트",
        "zone_no": "17519"
      },
      "address": {
        "address_name": "경기 안성시 죽산면 죽산리 343-1",
        "region_1depth_name": "경기",
        "region_2depth_name": "안성시",
        "region_3depth_name": "죽산면 죽산리",
        "mountain_yn": "N",
        "main_address_no": "343",
        "sub_address_no": "1",
        "zip_code": "456894"
      }
    }
  ]
}"#;

    let (called_sender, called_receiver) = std::sync::mpsc::channel();
    let (shutdown_sender, shutdown_receiver) = tokio::sync::oneshot::channel();

    let service = make_service_fn(move |_| {
        let called_sender = called_sender.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let called_sender = called_sender.clone();
                async move {
                    let uri = req.uri();
                    assert_eq!(uri.path(), "/geo/coord2address.json");
                    assert_eq!(uri.query(), Some("page=2&x=123.123&y=456.456"));

                    let headers = req.headers();
                    assert_eq!(
                        headers.get("Authorization"),
                        Some(&HeaderValue::from_static("KakaoAK key"))
                    );

                    called_sender.send(()).unwrap();

                    Ok::<_, Infallible>(Response::<Body>::new(RESP.into()))
                }
            }))
        }
    });

    let server = Server::bind(&"127.0.0.1:12123".parse().unwrap())
        .serve(service)
        .with_graceful_shutdown(async { shutdown_receiver.await.unwrap() });

    tokio::spawn(async {
        if let Err(e) = server.await {
            panic!("{}", e);
        }
    });

    let resp = daummap::CoordRequest::new("key", 123.123, 456.456)
        .base_url("http://localhost:12123")
        .page(2)
        .get_address()
        .await
        .unwrap();

    shutdown_sender.send(()).unwrap();
    called_receiver.try_recv().unwrap();

    assert_eq!(resp.len(), 1);

    let address = &resp[0];

    assert!(address.land_lot.is_some());
    let land_lot = address.land_lot.as_ref().unwrap();
    assert_eq!(&land_lot.address, "경기 안성시 죽산면 죽산리 343-1");

    assert!(address.road.is_some());
    let road = address.road.as_ref().unwrap();
    assert_eq!(&road.address, "경기도 안성시 죽산면 죽산초교길 69-4");
}

#[tokio::test]
async fn test_keyword() {
    static RESP: &'static str = r#"{
  "meta": {
    "same_name": {
      "region": [],
      "keyword": "카카오프렌즈",
      "selected_region": ""
    },
    "pageable_count": 14,
    "total_count": 14,
    "is_end": true
  },
  "documents": [
    {
      "place_name": "카카오프렌즈 코엑스점",
      "distance": "418",
      "place_url": "http://place.map.daum.net/26338954",
      "category_name": "가정,생활 > 문구,사무용품 > 디자인문구 > 카카오프렌즈",
      "address_name": "서울 강남구 삼성동 159",
      "road_address_name": "서울 강남구 영동대로 513",
      "id": "26338954",
      "phone": "02-6002-1880",
      "category_group_code": "",
      "category_group_name": "",
      "x": "127.05902969025047",
      "y": "37.51207412593136"
    }
  ]
}"#;

    let (called_sender, called_receiver) = std::sync::mpsc::channel();
    let (shutdown_sender, shutdown_receiver) = tokio::sync::oneshot::channel();

    let service = make_service_fn(move |_| {
        let called_sender = called_sender.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let called_sender = called_sender.clone();
                async move {
                    let uri = req.uri();
                    assert_eq!(uri.path(), "/search/keyword.json");
                    assert_eq!(
                uri.query(),
                Some("query=keyword&page=2&size=5&sort=accuracy&x=123.123&y=456.456&radius=1234")
            );

                    let headers = req.headers();
                    assert_eq!(
                        headers.get("Authorization"),
                        Some(&HeaderValue::from_static("KakaoAK key"))
                    );

                    called_sender.send(()).unwrap();

                    Ok::<_, Infallible>(Response::<Body>::new(RESP.into()))
                }
            }))
        }
    });

    let server = Server::bind(&"127.0.0.1:12124".parse().unwrap())
        .serve(service)
        .with_graceful_shutdown(async { shutdown_receiver.await.unwrap() });

    tokio::spawn(async {
        if let Err(e) = server.await {
            panic!("{}", e);
        }
    });

    let resp = daummap::KeywordRequest::new("key", "keyword")
        .base_url("http://localhost:12124")
        .coord(123.123, 456.456)
        .radius(1234)
        .page(2)
        .size(5)
        .get()
        .await
        .unwrap();

    shutdown_sender.send(()).unwrap();
    called_receiver.try_recv().unwrap();

    assert_eq!(resp.total_count, 14);
    assert_eq!(resp.pageable_count, 14);
    assert_eq!(resp.is_end, true);

    assert_eq!(resp.places.len(), 1);

    let place = &resp.places[0];
    assert_eq!(&place.name, "카카오프렌즈 코엑스점");
}

#[tokio::test]
async fn test_category() {
    static RESP: &'static str = r#"{
  "meta": {
    "same_name": null,
    "pageable_count": 11,
    "total_count": 11,
    "is_end": true
  },
  "documents": [
    {
      "place_name": "장생당약국",
      "distance": "",
      "place_url": "http://place.map.daum.net/16618597",
      "category_name": "의료,건강 > 약국",
      "address_name": "서울 강남구 대치동 943-16",
      "road_address_name": "서울 강남구 테헤란로84길 17",
      "id": "16618597",
      "phone": "02-558-5476",
      "category_group_code": "PM9",
      "category_group_name": "약국",
      "x": "127.05897078335246",
      "y": "37.506051888130386"
    }
  ]
}"#;

    let (called_sender, called_receiver) = std::sync::mpsc::channel();
    let (shutdown_sender, shutdown_receiver) = tokio::sync::oneshot::channel();

    let service = make_service_fn(move |_| {
        let called_sender = called_sender.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let called_sender = called_sender.clone();
                async move {
                    let uri = req.uri();
                    assert_eq!(uri.path(), "/search/category.json");
                    assert_eq!(
                uri.query(),
                Some("category_group_code=PM9&page=2&size=5&sort=accuracy&rect=123.123%2C456.456%2C321.321%2C654.654")
            );

                    let headers = req.headers();
                    assert_eq!(
                        headers.get("Authorization"),
                        Some(&HeaderValue::from_static("KakaoAK key"))
                    );

                    called_sender.send(()).unwrap();

                    Ok::<_, Infallible>(Response::<Body>::new(RESP.into()))
                }
            }))
        }
    });

    let server = Server::bind(&"127.0.0.1:12125".parse().unwrap())
        .serve(service)
        .with_graceful_shutdown(async { shutdown_receiver.await.unwrap() });

    tokio::spawn(async {
        if let Err(e) = server.await {
            panic!("{}", e);
        }
    });

    let resp = daummap::CategoryRequest::rect(
        "key",
        daummap::CategoryGroup::Pharmacy,
        123.123,
        456.456,
        321.321,
        654.654,
    )
    .base_url("http://localhost:12125")
    .page(2)
    .size(5)
    .get()
    .await
    .unwrap();

    shutdown_sender.send(()).unwrap();
    called_receiver.try_recv().unwrap();

    assert_eq!(resp.total_count, 11);
    assert_eq!(resp.pageable_count, 11);
    assert_eq!(resp.is_end, true);

    assert_eq!(resp.places.len(), 1);

    let place = &resp.places[0];
    assert_eq!(&place.name, "장생당약국");
}
