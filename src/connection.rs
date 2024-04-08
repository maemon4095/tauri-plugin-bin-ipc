use std::{
    ops::{Deref, DerefMut},
    sync::{Mutex, RwLock},
};

use crate::Body;
use futures::channel::mpsc::{Receiver, Sender};
use pigeonhole::VecPigeonhole;
use tauri::async_runtime::JoinHandle;

pub struct Connection {
    pub key: u32,
    pub tx: Sender<Body>,
    pub rx: Receiver<Body>,
    pub handle: JoinHandle<()>,
}

pub struct ConnectionBag(RwLock<VecPigeonhole<Mutex<Connection>>>);

impl ConnectionBag {
    pub fn new() -> Self {
        Self(RwLock::new(VecPigeonhole::new()))
    }

    pub fn connections(&self) -> impl '_ + Deref<Target = VecPigeonhole<Mutex<Connection>>> {
        self.0.read().unwrap()
    }

    pub fn connections_mut(&self) -> impl '_ + DerefMut<Target = VecPigeonhole<Mutex<Connection>>> {
        self.0.write().unwrap()
    }
}
