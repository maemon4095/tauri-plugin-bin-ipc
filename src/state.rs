use crate::connection::ConnectionBag;
use crate::event_emitter::EventEmitter;
use crate::listener::{Listener, ListenerBox};
use crate::Body;
use crate::{channel_credentials::ChannelCredentials, connection::Connection};
use futures::channel::mpsc::{Receiver, Sender};
use pigeonhole::VecPigeonhole;
use rand::RngCore;
use rand::{Rng, SeedableRng};
use std::ops::{Deref, DerefMut};
use std::sync::Mutex;
use tauri::async_runtime::JoinHandle;
use tauri::AppHandle;

pub struct State<R: tauri::Runtime> {
    pub emitter: EventEmitter<R>,
    pub listener: ListenerBox<R>,
    pub rng: Mutex<Box<dyn RngCore + Send + Sync>>,
    pub bag: ConnectionBag,
}

impl<R: tauri::Runtime> State<R> {
    pub fn new<L: Listener<R>>(scheme: String, app_handle: AppHandle<R>, listener: L) -> Self {
        Self {
            emitter: EventEmitter::new(scheme, app_handle),
            listener: ListenerBox::new(listener),
            rng: Mutex::new(Box::new(rand::rngs::StdRng::from_entropy())),
            bag: ConnectionBag::new(),
        }
    }

    pub fn connections(&self) -> impl '_ + Deref<Target = VecPigeonhole<Mutex<Connection>>> {
        self.bag.connections()
    }

    pub fn connections_mut(&self) -> impl '_ + DerefMut<Target = VecPigeonhole<Mutex<Connection>>> {
        self.bag.connections_mut()
    }

    pub fn rng(&self) -> std::sync::MutexGuard<'_, Box<dyn RngCore + Send + Sync>> {
        self.rng.lock().unwrap()
    }

    pub fn connect(
        &self,
        client_tx: Sender<Body>,
        client_rx: Receiver<Body>,
        handle: JoinHandle<()>,
    ) -> ChannelCredentials {
        let key = self.rng().gen();
        let id = self.bag.insert(Connection {
            key,
            tx: client_tx,
            rx: client_rx,
            handle,
        });

        ChannelCredentials { id, key }
    }

    fn close(&self, id: usize) -> Result<Connection, ()> {
        self.connections_mut()
            .remove(id)
            .map(|m| m.into_inner().unwrap())
    }
}
