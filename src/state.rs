use crate::connection::Connection;
use crate::connection::ConnectionBag;
use crate::event_emitter::EventEmitter;
use crate::listener::{Listener, ListenerBox};
use pigeonhole::VecPigeonhole;
use rand::RngCore;
use rand::SeedableRng;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

pub struct State<R: tauri::Runtime> {
    pub scheme: Arc<String>,
    pub listener: ListenerBox<R>,
    pub rng: Mutex<Box<dyn RngCore + Send + Sync>>,
    pub bag: ConnectionBag,
}

impl<R: tauri::Runtime> State<R> {
    pub fn new<L: Listener<R>>(scheme: String, listener: L) -> Self {
        let scheme = Arc::new(scheme);
        Self {
            scheme: Arc::clone(&scheme),
            listener: ListenerBox::new(
                move |app, id, _r| {
                    EventEmitter::new(&scheme, id).emit_cleanup(app).unwrap();
                },
                listener,
            ),
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

    pub fn close(&self, id: usize) -> Result<Connection, ()> {
        self.connections_mut()
            .remove(id)
            .map(|m| m.into_inner().unwrap())
    }
}
