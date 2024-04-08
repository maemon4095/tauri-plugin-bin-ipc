mod channel;
mod channel_credentials;
mod connection;
mod error;
mod event_emitter;
mod listener;
mod request_path;
mod state;

use std::sync::{Arc, Mutex};

use channel_credentials::ChannelCredentials;
use connection::Connection;
use error::{InvalidMethodError, RequestPathError};
use event_emitter::EventEmitter;
use futures::SinkExt;
use rand::Rng;
use request_path::RequestPath;
use state::State;
use tauri::{
    async_runtime,
    http::{self, status::StatusCode, Request, Response},
    plugin::{Builder, TauriPlugin},
    AppHandle, Runtime,
};

type Body = Vec<u8>;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    let scheme = "bin-ipc";
    let state: State<R> = State::new(scheme.to_string(), |app: &AppHandle<R>, tx, rx| {
        tauri::async_runtime::spawn(async { Ok::<(), ()>(()) })
    });

    Builder::new("bin-ipc")
        .register_uri_scheme_protocol(scheme, move |app_handle, req| {
            if req.method() != http::method::Method::POST {
                return Err(InvalidMethodError.into());
            }

            let uri: http::Uri = req.uri().parse()?;
            let path: RequestPath = uri.path().parse()?;
            let connections = state.connections();
            let body = req.body();
            match path {
                RequestPath::Push { id, key } => {
                    let connection = connections.get(id).ok_or(RequestPathError)?;
                    let mut connection = connection.lock().unwrap();
                    if connection.key != key {
                        return Err(RequestPathError.into());
                    }
                    let body = body.clone();
                    async_runtime::block_on(async { connection.tx.send(body).await })?;
                    any_origin_response()
                        .status(StatusCode::OK)
                        .body(Vec::new())
                }
                RequestPath::Pop { id, key } => {
                    let connection = connections.get(id).ok_or(RequestPathError)?;
                    let mut connection = connection.lock().unwrap();
                    if connection.key != key {
                        return Err(RequestPathError.into());
                    }

                    let result = connection.rx.try_next();
                    match result {
                        Ok(Some(body)) => any_origin_response().status(StatusCode::OK).body(body),
                        Ok(None) => any_origin_response()
                            .status(StatusCode::NO_CONTENT)
                            .body(Vec::new()),
                        Err(_) => any_origin_response()
                            .status(StatusCode::CONTINUE)
                            .body(Vec::new()),
                    }
                }
                RequestPath::CloseDown { id, key } => {
                    let connection = connections.get(id).ok_or(RequestPathError)?;
                    let mut connection = connection.lock().unwrap();
                    if connection.key != key {
                        return Err(RequestPathError.into());
                    }

                    connection.rx.close();
                    any_origin_response().body(Vec::new())
                }

                RequestPath::CloseUp { id, key } => {
                    let connection = connections.get(id).ok_or(RequestPathError)?;
                    let mut connection = connection.lock().unwrap();
                    if connection.key != key {
                        return Err(RequestPathError.into());
                    }

                    connection.tx.close_channel();
                    any_origin_response().body(Vec::new())
                }
                RequestPath::Connect => handshake(app_handle, &state, req),
                RequestPath::Disconnect { id, key } => {
                    let connection = connections.get(id).ok_or(RequestPathError)?;
                    let connection = connection.lock().unwrap();
                    if connection.key != key {
                        return Err(RequestPathError.into());
                    }
                    cleanup(app_handle, &state, id, req)
                }
            }
        })
        .build()
}

fn any_origin_response() -> http::ResponseBuilder {
    http::ResponseBuilder::new().header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
}

fn handshake<R: Runtime>(
    app_handle: &AppHandle<R>,
    state: &State<R>,
    _req: &Request,
) -> Result<Response, Box<(dyn std::error::Error)>> {
    let mut connections = state.connections_mut();
    let key = state.rng().gen();
    let reservation = connections.reserve();
    let id = reservation.id();
    let on_send = {
        let app = app_handle.clone();
        let scheme = Arc::clone(&state.scheme);
        move || {
            let emitter = EventEmitter::new(&scheme, id);
            emitter.emit_ready(&app).map_err(Into::into)
        }
    };
    let (server, client) = channel::channel(on_send, 32, 32);
    let handle = state.listener.listen(app_handle, id, server.0, server.1);
    reservation.set(Mutex::new(Connection {
        key,
        handle,
        tx: client.0,
        rx: client.1,
    }));

    let credentials = ChannelCredentials { key, id };
    return any_origin_response()
        .status(StatusCode::OK)
        .body(serde_json::to_vec(&credentials)?);
}

fn cleanup<R: tauri::Runtime>(
    _app_handle: &AppHandle<R>,
    state: &State<R>,
    id: usize,
    _req: &Request,
) -> Result<Response, Box<(dyn std::error::Error)>> {
    state.close(id).unwrap();
    return any_origin_response().body(Vec::new());
}
