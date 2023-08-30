use std::{time::Duration, net::IpAddr, collections::HashMap, io::Write};

use futures_util::{StreamExt, pin_mut};
use mdns::{Response, RecordKind};

#[derive(Debug)]
pub struct AirPlayFeatures {
    pub supports_video_v1: bool,
    pub supports_video_v2: bool,
    pub supports_photo: bool,
    pub supports_slideshow: bool,
    pub supports_screen: bool,
    pub supports_audio: bool,
    pub audio_redundant: bool,
    pub rsa_auth: bool,
    pub fairplay_auth: bool,
    pub mfi_auth: bool,
    pub send_artwork: bool,
    pub send_track_progress: bool,
    pub send_daap_nowplaying: bool,
    pub send_bplist_nowplaying: bool,
    pub supports_alac: bool,
    pub supports_aac: bool,
    pub supports_aac_eld: bool,
    pub supports_opus: bool,
    pub supports_legacy_pairing: bool,
    pub has_unified_advertiser_info: bool,
    pub is_carplay: bool,
    pub supports_volume: bool,
    pub supports_airplay_video_queue: bool,
    pub supports_airplay_from_cloud: bool,
    pub supports_tls_psk: bool,
    pub supports_unified_media_control: bool,
    pub supports_buffered_audio: bool,
    pub supports_ptp: bool,
    pub supports_screen_multi_codec: bool,
    pub supports_system_pairing: bool,
    pub is_ap_valeria_screen_sender: bool,
    pub supports_homekit: bool,
    pub supports_coreutils: bool,
    pub supports_unified_pair_mfi: bool,
    pub supports_setpeers_extended_message: bool,
    pub supports_ap_sync: bool,
    pub supports_wol: bool,
    pub supports_hangdog: bool,
    pub supports_audio_stream_connection_setup: bool,
    pub supports_audio_media_data_control: bool,
    pub supports_rfc2198_redundancy: bool,
}

fn is_bit_set(v: u64, b: u8) -> bool {
    (v & (1 << b)) != 0
}

impl From<u64> for AirPlayFeatures {
    fn from(v: u64) -> Self {
        AirPlayFeatures {
            supports_video_v1: is_bit_set(v, 0),
            supports_video_v2: is_bit_set(v, 49),
            supports_photo: is_bit_set(v, 1),
            supports_slideshow: is_bit_set(v, 5),
            supports_screen: is_bit_set(v, 7),
            supports_audio: is_bit_set(v, 9),
            audio_redundant: is_bit_set(v, 11),
            fairplay_auth: is_bit_set(v, 14),
            rsa_auth: is_bit_set(v, 23),
            mfi_auth: is_bit_set(v, 26) || is_bit_set(v, 51),
            send_artwork: is_bit_set(v, 15),
            send_track_progress: is_bit_set(v, 16),
            send_daap_nowplaying: is_bit_set(v, 17),
            send_bplist_nowplaying: is_bit_set(v, 50),
            supports_alac: is_bit_set(v, 18),
            supports_aac: is_bit_set(v, 19),
            supports_aac_eld: is_bit_set(v, 20),
            supports_opus: is_bit_set(v, 21),
            supports_legacy_pairing: is_bit_set(v, 27),
            has_unified_advertiser_info: is_bit_set(v, 30),
            is_carplay: is_bit_set(v, 32),
            supports_volume: !is_bit_set(v, 32),
            supports_airplay_video_queue: is_bit_set(v, 33),
            supports_airplay_from_cloud: is_bit_set(v, 34),
            supports_tls_psk: is_bit_set(v, 35),
            supports_unified_media_control: is_bit_set(v, 38),
            supports_buffered_audio: is_bit_set(v, 40),
            supports_ptp: is_bit_set(v, 41),
            supports_screen_multi_codec: is_bit_set(v, 42),
            supports_system_pairing: is_bit_set(v, 43),
            is_ap_valeria_screen_sender: is_bit_set(v, 44),
            supports_homekit: is_bit_set(v, 46),
            supports_coreutils: is_bit_set(v, 38) || is_bit_set(v, 43) || is_bit_set(v, 46) || is_bit_set(v, 48),
            supports_unified_pair_mfi: is_bit_set(v, 51),
            supports_setpeers_extended_message: is_bit_set(v, 52),
            supports_ap_sync: is_bit_set(v, 54),
            supports_wol: is_bit_set(v, 55) || is_bit_set(v, 56),
            supports_hangdog: is_bit_set(v, 58), // TODO: more complex
            supports_audio_stream_connection_setup: is_bit_set(v, 59),
            supports_audio_media_data_control: is_bit_set(v, 60),
            supports_rfc2198_redundancy: is_bit_set(v, 61),
        }
    }
}

impl From<(u32, u32)> for AirPlayFeatures {
    fn from(v: (u32, u32)) -> Self {
        let x = u64::from(v.1) << 32 | u64::from(v.0);
        x.into()
    }
}

#[derive(Debug)]
pub struct AirPlayReceiverMediaRemoteMeta {
    pub model_name: String,
    pub allow_pairing: bool,
    pub bluetooth_address: Vec<u8>,
    pub mac_address: String,
    pub name: String,
    pub uuid: String,
    pub system_build_version: String,

    /// NOTE: This refers to AirPlayReceiverMeta.system_pairing_identity
    pub local_airplay_receiver_pairing_identity: String,
}

#[derive(Debug)]
pub struct AirPlayReceiverMeta {
    pub name: String,
    pub ip_addresses: Vec<IpAddr>,
    pub port: u16,

    pub firmware_version: Option<String>,
    pub access_control_level: Option<i64>,
    pub bluetooth_address: Option<String>,
    pub device_id: Option<String>,
    pub features: Option<AirPlayFeatures>,
    pub required_sender_features: Option<AirPlayFeatures>,
    pub flags: Option<u64>,
    pub group_id: Option<String>,
    pub group_contains_discoverable_leader: Option<bool>,
    pub group_public_name: Option<String>,
    pub is_group_leader: Option<bool>,
    pub home_group_id: Option<String>,
    pub household_id: Option<String>,
    pub parent_group_id: Option<String>,
    pub parent_group_contains_discoverable_leader: Option<bool>,
    pub tight_sync_id: Option<String>,
    pub homekit_home_id: Option<String>,
    pub model: Option<String>,
    pub manufacturer: Option<String>,
    pub serial_number: Option<String>,
    pub protocol_version: Option<String>,
    pub public_airplay_pairing_identity: Option<String>,
    pub public_system_pairing_identity: Option<String>,
    pub public_key: Option<Vec<u8>>,
    pub airplay_version: Option<String>,
    pub os_version: Option<String>,

    pub media_remote: Option<AirPlayReceiverMediaRemoteMeta>,
}

impl AirPlayReceiverMeta {
    pub fn is_sane(&self) -> bool {
        self.model.is_some() &&
        self.features.is_some() &&
        self.name.len() > 0 &&
        self.device_id.is_some() &&
        !self.name.ends_with(".local")
    }
}

fn response_to_meta(response: Response) -> Option<AirPlayReceiverMeta> {
    let airplay_record_name = match &response.answers.iter().find(|x| x.name == "_airplay._tcp.local" && match x.kind {
        RecordKind::PTR(_) => true,
        _ => false,
    })?.kind {
        RecordKind::PTR(x) => Some(x),
        _ => None,
    }?.clone();

    let mut name = airplay_record_name.trim_end_matches("_airplay._tcp.local");

    let airplay_entries: HashMap<&str, &str> = match &response.additional.iter().find(|x| x.name == airplay_record_name && match x.kind {
        RecordKind::TXT(_) => true,
        _ => false,
    })?.kind {
        RecordKind::TXT(x) => Some(x),
        _ => None,
    }?.into_iter().map(|x| x.split_once('=').unwrap()).collect();

    let (hostname, airplay_port) = response.additional.iter().find_map(|x| {
        if x.name == airplay_record_name {
            match &x.kind {
                RecordKind::SRV { port, target, .. } => Some((target.as_str(), *port)),
                _ => None,
            }
        } else {
            None
        }
    })?;

    let ip_addresses: Vec<IpAddr> = response.additional.iter().filter_map(|x| {
        if x.name == *hostname {
            match &x.kind {
                RecordKind::A(x) => Some(IpAddr::V4(x.clone())),
                RecordKind::AAAA(x) => Some(IpAddr::V6(x.clone())),
                _ => None,
            }
        } else {
            None
        }
    }).collect();

    Some(AirPlayReceiverMeta {
        name: name.to_string(),
        ip_addresses,
        port: airplay_port,

        firmware_version: airplay_entries.get("fv").map(|x| x.to_string()),
        access_control_level: airplay_entries.get("acl").map(|x| i64::from_str_radix(x, 10).unwrap()),
        bluetooth_address: airplay_entries.get("btaddr").map(|x| x.to_string()),
        device_id: airplay_entries.get("deviceid").map(|x| x.to_string()),
        features: airplay_entries.get("features").map(|x| {
            let (x, y) = x.split_once(',').unwrap();
            
            AirPlayFeatures::from((
                u32::from_str_radix(x.trim_start_matches("0x"), 16).unwrap(),
                u32::from_str_radix(y.trim_start_matches("0x"), 16).unwrap(),
            ))
        }),
        required_sender_features: airplay_entries.get("rsf").map(|x| AirPlayFeatures::from(u64::from_str_radix(x.trim_start_matches("0x"), 16).unwrap())),
        flags: airplay_entries.get("flags").map(|x| u64::from_str_radix(x.trim_start_matches("0x"), 10).unwrap()),
        group_id: airplay_entries.get("gid").map(|x| x.to_string()),
        group_contains_discoverable_leader: airplay_entries.get("gcgl").map(|x| *x == "1"),
        group_public_name: airplay_entries.get("gpn").map(|x| x.to_string()),
        is_group_leader: airplay_entries.get("igl").map(|x| *x == "1"),
        home_group_id: airplay_entries.get("hgid").map(|x| x.to_string()),
        household_id: airplay_entries.get("hmid").map(|x| x.to_string()),
        parent_group_id: airplay_entries.get("pgid").map(|x| x.to_string()),
        parent_group_contains_discoverable_leader: airplay_entries.get("pgcgl").map(|x| *x == "1"),
        tight_sync_id: airplay_entries.get("tsid").map(|x| x.to_string()),
        homekit_home_id: airplay_entries.get("hkid").map(|x| x.to_string()),
        model: airplay_entries.get("model").map(|x| x.to_string()),
        manufacturer: airplay_entries.get("manufacturer").map(|x| x.to_string()),
        serial_number: airplay_entries.get("serialNumber").map(|x| x.to_string()),
        protocol_version: airplay_entries.get("protovers").map(|x| x.to_string()),
        public_airplay_pairing_identity: airplay_entries.get("pi").map(|x| x.to_string()),
        public_system_pairing_identity: airplay_entries.get("psi").map(|x| x.to_string()),
        public_key: airplay_entries.get("pk").map(|x| x
            .as_bytes()
            .chunks(2)
            .map(std::str::from_utf8)
            .map(|x| u8::from_str_radix(x.unwrap(), 16).unwrap())
            .collect()
        ),
        airplay_version: airplay_entries.get("srcvers").map(|x| x.to_string()),
        os_version: airplay_entries.get("osvers").map(|x| x.to_string()),

        media_remote: None,
    })
}

pub async fn find() {
    let stream = mdns::discover::all("_airplay._tcp.local", Duration::from_secs(1))
        .unwrap()
        .listen();

    pin_mut!(stream);

    println!("looking...");

    while let Some(Ok(response)) = stream.next().await {
        let meta = response_to_meta(response.clone());
        if response.additional.len() == 0 { // TODO: better check
            println!("Received weird RAOP PTR broadcast.");
        } else {
            if let Some(meta) = meta {
                println!("{} ({})", meta.name, meta.device_id.as_ref().unwrap_or(&"?".to_string()));
            } else {
                println!("Unrecognized!");
            }
            println!("{:#?}", response);
        }
        std::io::stdout().flush().unwrap()
    }
}
