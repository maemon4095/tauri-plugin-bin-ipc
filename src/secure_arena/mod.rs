mod id;

use pigeonhole::VecPigeonhole;
use rand::{rngs::SmallRng, SeedableRng};
use std::sync::{Mutex, RwLock, RwLockReadGuard};

pub use id::SecureArenaId;

struct Entry<T> {
    value: T,
    key: u64,
}
pub struct SecureArena<T> {
    rng: Mutex<SmallRng>,
    items: RwLock<VecPigeonhole<Entry<T>>>,
}

#[derive(Debug)]
pub enum SecureArenaError {
    TooManyItems,
}

#[allow(unused)]
pub struct SecureArenaGuard<'a, T> {
    items_lock: RwLockReadGuard<'a, VecPigeonhole<Entry<T>>>,
    ptr: *const T,
}

impl<'a, T> std::ops::Deref for SecureArenaGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<T> SecureArena<T> {
    pub fn new() -> Self {
        Self {
            rng: Mutex::new(SmallRng::from_entropy()),
            items: RwLock::new(VecPigeonhole::new()),
        }
    }

    pub fn get(&self, id: SecureArenaId) -> Option<SecureArenaGuard<'_, T>> {
        let lock = self.items.read().unwrap();

        match lock.get(id.id() as usize).filter(|c| c.key == id.key()) {
            Some(v) => {
                let ptr: *const T = &v.value;
                Some(SecureArenaGuard {
                    items_lock: lock,
                    ptr,
                })
            }
            None => None,
        }
    }

    pub fn alloc(&self, item: T) -> Result<SecureArenaId, SecureArenaError> {
        let mut lock = self.items.write().unwrap();
        let reservation = lock.reserve();
        let mut rng = self.rng.lock().unwrap();
        let Ok(id) = SecureArenaId::new(reservation.id(), &mut *rng) else {
            return Err(SecureArenaError::TooManyItems);
        };
        reservation.set(Entry {
            value: item,
            key: id.key(),
        });
        Ok(id)
    }

    pub fn delete(&self, id: SecureArenaId) -> Option<T> {
        let mut lock = self.items.write().unwrap();
        lock.remove(id.id()).map(|e| e.value)
    }
}
