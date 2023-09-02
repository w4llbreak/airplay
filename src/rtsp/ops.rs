use plist::Data;
use serde::{Serialize, Deserialize};

use super::{Client, Response, Request, Body, Method};

type Result<T> = std::result::Result<T, Response>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TimingPeer {
    pub addresses: Vec<String>,

    #[serde(rename = "ID")]
    pub id: String,

    pub supports_clock_port_matching_override: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupInfoRequest {
    #[serde(rename = "deviceID")]
    pub device_id: String,

    pub eiv: Data,
    pub ekey: Data,
    pub et: i32,

    pub group_contains_group_leader: bool,
    pub group_uuid: String,

    #[serde(rename = "isMultiSelectAirPlay")]
    pub is_multi_select_airplay: bool,

    pub mac_address: String,
    pub model: String,
    pub name: String,
    pub os_build_version: String,
    pub os_name: String,
    pub os_version: String,
    pub sender_supports_relay: bool,

    #[serde(rename = "sessionUUID")]
    pub session_uuid: String,

    pub source_version: String,
    pub timing_peer_info: Vec<TimingPeer>,
    pub timing_peer_list: Vec<TimingPeer>,
    pub timing_protocol: String,
}

impl Client {
    pub async fn fetch_info(&mut self) -> Result<Response> {
        let req = self.request(
            Request::new(Method::GET, "/info")
        ).await.unwrap();
        let res = req.await.unwrap();

        if res.status == 200 {
            Ok(res)
        } else {
            Err(res)
        }
    }

    pub async fn setup_info(&mut self, body: SetupInfoRequest) -> Result<Response> {
        let req = self.request(
            Request::new_body(
                Method::SETUP,
                format!("rtsp://{}/666", self.peer.ip().to_string()),
                Body::PList(plist::to_value(&body).unwrap()),
            )
        ).await.unwrap();
        let res = req.await.unwrap();

        if res.status == 200 {
            Ok(res)
        } else {
            Err(res)
        }
    }
}