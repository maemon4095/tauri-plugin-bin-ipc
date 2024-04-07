mod channel;
mod channel_credentials;
mod connection;
mod error;
mod event_emitter;
mod listener;
mod request_path;
mod state;

use error::{InvalidMethodError, RequestPathError};
use futures::SinkExt;
use request_path::{RequestPath, RequestType};
use state::State;
use tauri::{
    async_runtime,
    http::{self, status::StatusCode, Request, Response},
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime,
};

type Body = Vec<u8>;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    let state: State<R> = State::new(|app: &AppHandle<R>, tx, rx| {
        tauri::async_runtime::spawn(async { Ok::<(), ()>(()) })
    });

    Builder::new("bin-ipc")
        .register_uri_scheme_protocol("bin-ipc", move |app_handle, req| {
            if req.method() != http::method::Method::POST {
                return Err(InvalidMethodError.into());
            }

            let uri: http::Uri = req.uri().parse()?;
            let path: RequestPath = uri.path().parse()?;
            let connections = state.connections();
            let connection = connections.get(path.id).ok_or(RequestPathError)?;
            let mut connection = connection.lock().unwrap();
            if connection.key != path.key {
                return Err(RequestPathError.into());
            }
            let body = req.body();
            match path.ty {
                RequestType::Push => {
                    let body = body.clone();
                    async_runtime::block_on(async { connection.tx.send(body).await })?;
                    any_origin_response()
                        .status(StatusCode::OK)
                        .body(Vec::new())
                }
                RequestType::Pop => {
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
                RequestType::CloseDown => {
                    connection.rx.close();
                    any_origin_response().body(Vec::new())
                }
                RequestType::CloseUp => {
                    connection.tx.close_channel();
                    any_origin_response().body(Vec::new())
                }
                RequestType::Connect => handshake(app_handle, &state, req),
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
    let (server, client) = channel::channel(|| Ok(()), 32, 32);

    let handle = state.listener.listen(app_handle, server.0, server.1);
    let credentials = state.connect(client.0, client.1, handle);

    return any_origin_response()
        .status(StatusCode::OK)
        .body(serde_json::to_vec(&credentials)?);
}
