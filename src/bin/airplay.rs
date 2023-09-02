use airplay::{mdns, rtsp::{self, ops::SetupInfoRequest}};
use plist::Data;

#[tokio::main]
async fn main() {
    let meta = mdns::look_for("Nappali (2)".to_string()).await.expect("Device not found");

    let mut client = rtsp::Client::connect((meta.ip_addresses.iter().find_map(|x| match x {
        std::net::IpAddr::V4(x) => Some(x),
        std::net::IpAddr::V6(_) => None,
    }).expect("No IPv4 address found for device").to_owned(), meta.port)).await.expect("Failed to connect to RTSP");

    let info = client.fetch_info().await.expect("Failed to fetch info");
    println!("{:#?}", info);

    let setup_info = client.setup_info(SetupInfoRequest {
        device_id: "00:00:00:00:00:00".to_string(),
        eiv: Data::new(vec![]),
        ekey: Data::new(vec![]),
        et: 0,
        group_contains_group_leader: false,
        group_uuid: "67EAD1FA-7EAB-4810-82F7-A9132FD2D0BB".to_string(),
        is_multi_select_airplay: true,
        mac_address: "00:00:00:00:00:00".to_string(),
        model: "iPhone10,6".to_string(),
        name: "crystal".to_string(),
        os_build_version: "17B111".to_string(),
        os_name: "iPhone OS".to_string(),
        os_version: "13.2.3".to_string(),
        sender_supports_relay: false,
        session_uuid: "3195C737-1E6E-4487-BECB-4D287B7C7626".to_string(),
        source_version: "409.16".to_string(),
        timing_peer_info: vec![],
        timing_peer_list: vec![],
        timing_protocol: "PTP".to_string()
    }).await.expect("Failed to setup");
    println!("{:#?}", setup_info);
}
