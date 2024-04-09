mod channel;
mod channel_credentials;
mod connection;
mod error;
mod event_emitter;
mod listener;
mod request_path;
mod state;

use channel_credentials::ChannelCredentials;
use connection::Connection;
use error::{InvalidMethodError, RequestPathError};
use event_emitter::EventEmitter;
use futures::SinkExt;
use listener::Listener;
use rand::Rng;
use request_path::RequestPath;
use state::State;
use std::sync::{Arc, Mutex};
use tauri::{
    async_runtime,
    http::{self, status::StatusCode, Request, Response},
    plugin::{Builder as PluginBuilder, TauriPlugin},
    AppHandle, Runtime,
};

pub use channel::{Receiver, Sender};

macro_rules! get_connection_into {
    ($c: ident; $id:ident, $key: ident -> $v: ident) => {
        let $v = $c.get($id).ok_or(RequestPathError)?;
        #[allow(unused_mut)]
        let mut $v = $v.lock().unwrap();
        if $v.key != $key {
            return Err(RequestPathError.into());
        }
    };
}

type Body = Vec<u8>;

pub struct Builder<R: Runtime> {
    inits: Vec<Box<dyn FnOnce(PluginBuilder<R>) -> PluginBuilder<R>>>,
}

impl<R: Runtime> Default for Builder<R> {
    fn default() -> Self {
        Self::new()
    }
}
/// TODO: TcpListenerと同じInterface
/// ```rust
/// let listener = bind(80).await?;
/// let conn = listener.accept().await?;
/// ```
///
/// ```ts
/// const conn = await connect({ port: 80 });
/// ```
impl<R: Runtime> Builder<R> {
    pub fn new() -> Self {
        Self { inits: Vec::new() }
    }

    pub fn register_bin_ipc_protocol<L: Listener<R>>(
        mut self,
        scheme: impl Into<String>,
        listener: L,
    ) -> Self {
        let scheme = scheme.into();
        self.inits.push(Box::new(move |builder| {
            let state = State::new(scheme.clone(), listener);
            builder.register_uri_scheme_protocol(scheme, move |app_handle, req| {
                if req.method() != http::method::Method::POST {
                    return Err(InvalidMethodError.into());
                }

                let uri: http::Uri = req.uri().parse()?;
                let path: RequestPath = uri.path().parse()?;
                let body = req.body();
                match path {
                    RequestPath::Push { id, key } => {
                        let connections = state.connections();
                        get_connection_into!(connections; id, key -> connection);
                        let body = body.clone();
                        async_runtime::block_on(async { connection.tx.send(body).await })?;
                        any_origin_response()
                            .status(StatusCode::OK)
                            .body(Vec::new())
                    }
                    RequestPath::Pop { id, key } => {
                        let connections = state.connections();
                        get_connection_into!(connections; id, key -> connection);
                        let result = connection.rx.try_next();
                        match result {
                            Ok(Some(body)) => {
                                any_origin_response().status(StatusCode::OK).body(body)
                            }
                            Ok(None) => any_origin_response()
                                .status(StatusCode::NO_CONTENT)
                                .body(Vec::new()),
                            Err(_) => any_origin_response()
                                .status(StatusCode::CONTINUE)
                                .body(Vec::new()),
                        }
                    }
                    RequestPath::CloseDown { id, key } => {
                        let connections = state.connections();
                        get_connection_into!(connections; id, key -> connection);
                        connection.rx.close();
                        any_origin_response().body(Vec::new())
                    }
                    RequestPath::CloseUp { id, key } => {
                        let connections = state.connections();
                        get_connection_into!(connections; id, key -> connection);
                        connection.tx.close_channel();
                        any_origin_response().body(Vec::new())
                    }
                    RequestPath::Close { id, key } => {
                        let connections = state.connections();
                        get_connection_into!(connections; id, key -> connection);
                        connection.tx.close_channel();
                        connection.rx.close();
                        any_origin_response().body(Vec::new())
                    }
                    RequestPath::Connect => handshake(app_handle, &state, req),
                    RequestPath::CleanUp { id, key } => {
                        {
                            // key validation
                            let connections = state.connections();
                            get_connection_into!(connections; id, key -> connection);
                        }
                        // disconnect event response. assert js side cleanup is done
                        cleanup(app_handle, &state, id, req)
                    }
                }
            })
        }));

        self
    }

    pub fn build(self) -> TauriPlugin<R> {
        let mut builder = PluginBuilder::new("bin-ipc");
        for init in self.inits {
            builder = init(builder);
        }
        builder.build()
    }
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
