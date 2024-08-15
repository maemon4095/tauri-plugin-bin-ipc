use std::num::ParseIntError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecureArenaId(u64);

const ID_BITS: u32 = 32;
const KEY_UPPER_BOUND: u64 = 1 << (53 - ID_BITS);

impl SecureArenaId {
    pub(crate) fn id(&self) -> usize {
        (self.0 & ((1 << ID_BITS) - 1)) as usize
    }

    pub(crate) fn key(&self) -> u64 {
        self.0 >> ID_BITS
    }

    pub(super) fn new(id: usize, mut rng: impl rand::Rng) -> Result<Self, ()> {
        if id & !((1 << ID_BITS) - 1) != 0 {
            return Err(());
        }
        let key = rng.gen_range(0..KEY_UPPER_BOUND);

        Ok(Self((id as u64) | (key << ID_BITS)))
    }

    pub fn from_str_radix(src: &str, radix: u32) -> Result<SecureArenaId, ParseIntError> {
        u64::from_str_radix(src, radix).map(Self)
    }
}

impl serde::Serialize for SecureArenaId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.0)
    }
}

impl<'a> serde::Deserialize<'a> for SecureArenaId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        struct Visitor;

        impl<'a> serde::de::Visitor<'a> for Visitor {
            type Value = SecureArenaId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("expected 53bit integer.")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(SecureArenaId(v))
            }
        }

        deserializer.deserialize_u64(Visitor)
    }
}
