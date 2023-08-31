use std::{collections::HashMap, sync::Arc};

use tokio::{net::{tcp::OwnedWriteHalf, ToSocketAddrs, TcpStream}, sync::{RwLock, oneshot, Mutex}, io::{AsyncWriteExt, AsyncReadExt, self}, task::JoinHandle};

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum Method {
    GET,
    POST,
    SETUP,
    SET_PARAMETER,
    GET_PARAMETER,
    SETPEERS,
    RECORD,
    FLUSH,
    TEARDOWN,
}

impl ToString for Method {
    fn to_string(&self) -> String {
        match &self {
            Self::GET => "GET",
            Self::POST => "POST",
            Self::SETUP => "SETUP",
            Self::SET_PARAMETER => "SET_PARAMETER",
            Self::GET_PARAMETER => "GET_PARAMETER",
            Self::SETPEERS => "SETPEERS",
            Self::RECORD => "RECORD",
            Self::FLUSH => "FLUSH",
            Self::TEARDOWN => "TEARDOWN",
        }.to_string()
    }
}

#[derive(Clone, Debug)]
pub enum Body {
    None,
    Plist(plist::Value),
    Raw(Vec<u8>),
}

#[derive(Clone)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Body,
}

impl Request {
    pub fn new(method: Method, path: impl ToString) -> Request {
        Request {
            method,
            path: path.to_string(),
            headers: HashMap::from([
                ("X-Apple-ProtocolVersion".to_string(), "1".to_string()),
                ("User-Agent".to_string(), "AirPlay/409.16".to_string()),
            ]),
            body: Body::None,
        }
    }

    pub fn set_header(&mut self, name: impl ToString, value: impl ToString) -> Option<String> {
        self.headers.insert(name.to_string(), value.to_string())
    }

    pub(crate) fn normalize(&mut self, seq: usize) {
        let cl: usize = match &self.body {
            Body::Plist(x) => {
                let mut tmp: Vec<u8> = Vec::new();
                x.to_writer_binary(&mut tmp).unwrap();
                tmp.len()
            },
            Body::Raw(x) => x.len(),
            Body::None => 0,
        };

        if cl > 0 {
            self.set_header("Content-Length", cl);
        }

        self.set_header("CSeq", seq);
    }
}

#[derive(Clone, Debug)]
pub struct Response {
    pub status: i32,
    pub headers: HashMap<String, String>,
    pub body: Body,
}

pub struct Client {
    tx: OwnedWriteHalf,
    seq: RwLock<usize>,
    pending_seqs: Arc<Mutex<HashMap<usize, oneshot::Sender<Response>>>>,
    listener_handle: JoinHandle<()>,
}

impl Client {
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self, io::Error> {
        let stream = TcpStream::connect(addr).await.unwrap();
        let (mut rx, tx) = stream.into_split();

        let pending_seqs: Arc<Mutex<HashMap<usize, oneshot::Sender<Response>>>> = Default::default();

        Ok(Client {
            tx,
            seq: RwLock::new(0),
            pending_seqs: pending_seqs.clone(),
            listener_handle: tokio::spawn(async move {
                'request: loop {
                    let mut response_code = -1;
                    let mut response_headers: HashMap<String, String> = HashMap::new();
                    let mut hanging: String = String::new();
                    let mut leftover: Vec<u8> = Vec::new();
                    let mut response_body: Body = Body::None;

                    let mut buf = vec![0_u8; 128];
                    while let Ok(n) = rx.read(&mut buf).await {
                        if n == 0 {
                            break 'request;
                        }

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

                    if let Some(len) = response_headers.get("Content-Length") {
                        let len = str::parse::<usize>(len).unwrap();
                
                        let mut body = leftover;
                
                        if body.len() < len {
                            let mut more_body = vec![0_u8; len - body.len()];
                            rx.read_exact(&mut more_body).await.unwrap();
                            body.extend(more_body);
                        }
                
                        let ct = response_headers.get("Content-Type").cloned().unwrap_or("".to_string());
                
                        if ct == "application/x-apple-binary-plist" {
                            response_body = if let Ok(body) = plist::from_bytes::<plist::Value>(&body) {
                                Body::Plist(body)
                            } else {
                                Body::Raw(body)
                            }
                        } else {
                            response_body = Body::Raw(body);
                        }
                    }

                    let response = Response {
                        status: response_code,
                        headers: response_headers,
                        body: response_body,
                    };

                    if let Some(Ok(seq)) = response.headers.get("CSeq").map(|x| str::parse::<usize>(&x)) {
                        if let Some(entry) = pending_seqs.lock().await.remove(&seq) {
                            entry.send(response).unwrap();
                        } else {
                            println!("Encountered RTSP response with unlistened CSeq: {:#?}", response);
                        }
                    } else {
                        println!("Encountered RTSP response without CSeq: {:#?}", response);
                    }
                }
            }),
        })
    }

    pub async fn request(&mut self, mut request: Request) -> Result<oneshot::Receiver<Response>, io::Error> {
        let seq = {
            let mut sw = self.seq.write().await;
            let x = *sw;
            *sw += 1;
            x
        };

        request.normalize(seq);

        let mut req = format!("{} {} RTSP/1.0\r\n", request.method.to_string(), request.path);

        for (key, value) in request.headers.iter() {
            req += &format!("{}: {}\r\n", key, value);
        }

        req += "\r\n";

        let mut req = req.as_bytes().to_vec();

        match &request.body {
            Body::Plist(x) => {
                let mut body: Vec<u8> = Vec::new(); // TODO: preallocate capacity from Content-Length header
                x.to_writer_binary(&mut body).unwrap();
                req.extend(body);
            },
            Body::Raw(x) => req.extend(x),
            Body::None => {},
        };

        let (tx, rx) = oneshot::channel::<Response>();
        self.pending_seqs.lock().await.insert(seq, tx);
        self.tx.write_all(&req).await?;
        Ok(rx)
    }
}