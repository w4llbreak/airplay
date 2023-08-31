use airplay::{look_for, rtsp::{self}};
use plist::Dictionary;

#[tokio::main]
async fn main() {
    let meta = look_for("Nappali (2)".to_string()).await.expect("Device not found");

    let mut client = rtsp::Client::connect((meta.ip_addresses.iter().find_map(|x| match x {
        std::net::IpAddr::V4(x) => Some(x),
        std::net::IpAddr::V6(_) => None,
    }).expect("No IPv4 address found for device").to_owned(), meta.port)).await.expect("Failed to connect to RTSP");

    let mut info_req = Dictionary::new();
    info_req.insert("qualifier".to_string(), plist::Value::Array(vec! [
        plist::Value::String("txtAirPlay".to_string()),
    ]));
    let info_req = plist::Value::Dictionary(info_req);

    let mut req = rtsp::Request::new(rtsp::Method::GET, "/info");
    req.body = rtsp::Body::Plist(info_req);

    let req = client.request(req).await.unwrap();
    let res = req.await.unwrap();

    println!("{:#?}", res);
}
