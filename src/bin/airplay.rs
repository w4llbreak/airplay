use std::collections::HashMap;

use airplay::look_for;
use plist::Dictionary;
use tokio::{net::TcpStream, io::{AsyncWriteExt, AsyncReadExt}};


#[tokio::main]
async fn main() {
    let meta = look_for("Nappali (2)".to_string()).await.expect("Device not found");

    let mut rtsp_stream = TcpStream::connect((meta.ip_addresses.iter().find_map(|x| match x {
        std::net::IpAddr::V4(x) => Some(x),
        std::net::IpAddr::V6(x) => None,
    }).expect("No IPv4 address found for device").to_owned(), meta.port)).await.expect("Failed to connect to RTSP");

    let (mut rtsp_rx, mut rtsp_tx) = rtsp_stream.split();

    let mut info_req = Dictionary::new();

    info_req.insert("qualifier".to_string(), plist::Value::Array(vec! [
        plist::Value::String("txtAirPlay".to_string()),
    ]));

    let info_req = plist::Value::Dictionary(info_req);
    let mut info_payload: Vec<u8> = Vec::with_capacity(70);
    info_req.to_writer_binary(&mut info_payload).unwrap();

    rtsp_tx.write_all(b"GET /info RTSP/1.0\r\nX-Apple-ProtocolVersion: 1\r\nCSeq: 0\r\nUser-Agent: AirPlay/409.16\r\n\r\n").await.unwrap();
    rtsp_tx.write_all(&info_payload).await.unwrap();

    let mut buf = vec![0_u8; 128];

    let mut response_code = -1;
    let mut response_headers: HashMap<String, String> = HashMap::new();
    let mut hanging: String = String::new();
    let mut leftover: Vec<u8> = Vec::new();

    loop {
        let n = rtsp_rx.read(&mut buf).await.unwrap();

        let mut parsed = String::from_utf8(buf[0..n].to_vec());

        if let Err(err) = parsed {
            let valid_up_to = err.utf8_error().valid_up_to();
            parsed = String::from_utf8(buf[0..valid_up_to].to_vec());
            leftover.extend(&buf[valid_up_to..n]);
        }

        let parsed = parsed.unwrap();

        let parsed = hanging.clone() + &parsed;
        let mut parsed = parsed.as_str();

        let mut over = false;

        while parsed.contains("\r\n") {
            let (x, _parsed) = parsed.split_once("\r\n").unwrap();
            parsed = _parsed;

            if response_code == -1 {
                response_code = 9; // TODO: parse actual response code
            } else if x.len() == 0 {
                over = true;
                break;
            } else {
                let (key, value) = x.split_once(": ").unwrap();
                response_headers.insert(key.to_string(), value.to_string());
            }
        }

        hanging = parsed.to_string();

        if over {
            break;
        }
    }

    if hanging.len() > 0 {
        let mut lo = hanging.as_bytes().to_vec();
        lo.append(&mut leftover);
        leftover = lo;
    }

    println!("{} {:#?}", response_code, response_headers);

    if let Some(len) = response_headers.get("Content-Length") {
        let len = str::parse::<usize>(len).unwrap();

        let mut body = leftover;

        if body.len() < len {
            let mut more_body = vec![0_u8; len - body.len()];
            rtsp_rx.read_exact(&mut more_body).await.unwrap();
            body.extend(more_body);
        }

        let ct = response_headers.get("Content-Type").cloned().unwrap_or("".to_string());

        if ct == "application/x-apple-binary-plist" {
            println!("{:#?}", plist::from_bytes::<plist::Value>(&body));
        } else {
            println!("{}", ct);
        }
    }
}
