use crate::{
    handler::BinIpcHandler,
    secure_arena::{SecureArena, SecureArenaId},
    util::{declare_error, AppHandleExt},
    BinIpcError,
};
use std::sync::Mutex;
use tauri::{
    http::{
        header::{self, HeaderValue},
        method::Method,
        Response,
    },
    Manager,
};

declare_error!(InvalidRequestMethod; "Request method must be POST.");
declare_error!(InvalidRequestUrl; "Request URL is invalid.");
declare_error!(TooManyRequests; "Too many requests.");
declare_error!(HandlerPanicked; "bin-ipc handler was panicked");

pub fn create<R: tauri::Runtime, H: BinIpcHandler<R>>(
    scheme: &str,
    handler: H,
) -> impl Fn(
    &tauri::AppHandle<R>,
    &tauri::http::Request,
) -> std::result::Result<tauri::http::Response, Box<dyn std::error::Error>>
       + Send
       + Sync
       + 'static {
    let parser = RequestUrlParser::new(&scheme);
    move |app, req| {
        if req.method() != Method::POST {
            return Err(InvalidRequestMethod.into());
        }

        // `PluginBuilder::setup`を複数回呼び出した場合、前のセットアップ関数を上書きしてしまうため、初回に初期化する。
        let state = app.lazy_state(|_| BinIpcState::new());

        let Ok(url) = parser.parse(req.uri()) else {
            return Err(InvalidRequestUrl.into());
        };

        match url {
            RequestUrl::IpcSpawn(command) => {
                let Ok(id) = state.sessions.alloc(Session {
                    result: Mutex::new(None),
                }) else {
                    return Err(TooManyRequests.into());
                };

                let task = match handler.handle(&app, command, req.body()) {
                    Ok(v) => v,
                    Err(e) => return create_error_result(e),
                };
                tauri::async_runtime::spawn({
                    let app = app.clone();
                    async move {
                        let response = task.await;
                        let state = app.state::<BinIpcState>();
                        match state.sessions.get(id) {
                            None => (),
                            Some(session) => {
                                let mut lock = session.result.lock().unwrap();
                                *lock = Some(response);
                                drop(lock);
                                drop(session);
                                // TODO: error handling
                                let _result = app.emit_all("bin-ipc:ready", id);
                            }
                        };
                    }
                });

                Ok(create_json_response(&id, BinIpcStatus::Ok))
            }
            RequestUrl::IpcPoll(id) => {
                let state = app.state::<BinIpcState>();
                let Some(v) = state.sessions.get(id) else {
                    return Err(InvalidRequestUrl.into());
                };

                let result = v.result.lock().unwrap().take();
                drop(v);
                match result {
                    Some(r) => {
                        let _ = state.sessions.delete(id);
                        match r {
                            Ok(v) => Ok(create_response(v, BinIpcStatus::Ok)),
                            Err(e) => create_error_result(e),
                        }
                    }
                    None => Ok(create_response(Vec::new(), BinIpcStatus::Pending)),
                }
            }
        }
    }
}

fn create_json_response<T: serde::Serialize>(v: &T, status: BinIpcStatus) -> tauri::http::Response {
    let mut res = create_response(serde_json::to_vec(v).unwrap(), status);
    res.headers_mut().append(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json; charset=utf-8"),
    );
    res
}

fn create_error_result(
    e: BinIpcError,
) -> Result<tauri::http::Response, Box<dyn std::error::Error>> {
    if !e.is_reportable() {
        return Err(e.into_inner());
    }

    let e = e.into_inner();
    Ok(create_response(
        e.to_string().into_bytes(),
        BinIpcStatus::Error,
    ))
}

fn create_response(buf: Vec<u8>, status: BinIpcStatus) -> tauri::http::Response {
    let mut res = Response::new(buf);

    let headers = res.headers_mut();
    headers.append(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    headers.append(
        header::ACCESS_CONTROL_EXPOSE_HEADERS,
        HeaderValue::from_static(BIN_IPC_STATUS),
    );
    headers.append(BIN_IPC_STATUS, status.into());

    res
}

enum BinIpcStatus {
    Ok,
    Pending,
    Error,
}

impl Into<HeaderValue> for BinIpcStatus {
    fn into(self) -> HeaderValue {
        match self {
            BinIpcStatus::Ok => HeaderValue::from_static("ok"),
            BinIpcStatus::Pending => HeaderValue::from_static("pending"),
            BinIpcStatus::Error => HeaderValue::from_static("error"),
        }
    }
}

const BIN_IPC_STATUS: &str = "bin-ipc-status";

struct Session {
    result: Mutex<Option<Result<Vec<u8>, BinIpcError>>>,
}

struct BinIpcState {
    sessions: SecureArena<Session>,
}

impl BinIpcState {
    pub fn new() -> Self {
        Self {
            sessions: SecureArena::new(),
        }
    }
}

enum RequestUrl<'a> {
    // [scheme]://localhost/ipc/spawn/[command]
    IpcSpawn(&'a str),
    // [scheme]://localhost/ipc/poll/[id]
    IpcPoll(SecureArenaId),
}

struct RequestUrlParser {
    url_prefix_len: usize,
}

impl RequestUrlParser {
    pub fn new(name: &str) -> Self {
        Self {
            url_prefix_len: name.len() + "://localhost/".len(),
        }
    }

    pub fn parse<'a>(&self, url: &'a str) -> Result<RequestUrl<'a>, ()> {
        if url.len() <= self.url_prefix_len {
            return Err(());
        }

        let url = &url[self.url_prefix_len..];
        let Some((category, rest)) = url.split_once('/') else {
            return Err(());
        };

        match category {
            "ipc" => {
                let Some((op, rest)) = rest.split_once('/') else {
                    return Err(());
                };

                match op {
                    "spawn" => Ok(RequestUrl::IpcSpawn(rest)),
                    "poll" => {
                        let Ok(id) = SecureArenaId::from_str_radix(rest, 10) else {
                            return Err(());
                        };
                        Ok(RequestUrl::IpcPoll(id))
                    }
                    _ => Err(()),
                }
            }
            _ => Err(()),
        }
    }
}
