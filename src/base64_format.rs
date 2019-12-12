// These functions are only used if the API uses base64-encoded properties, so allow them to be
// dead code.
#![allow(dead_code)]
#[cfg(feature = "serdejson")]
use base64::{decode, encode};
#[cfg(feature = "serdejson")]
use serde::de::{Deserialize, Deserializer, Error};
#[cfg(feature = "serdejson")]
use serde::ser::{Serialize, Serializer};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
/// Base64-encoded byte array
pub struct ByteArray(pub Vec<u8>);

#[cfg(feature = "serdejson")]
impl Serialize for ByteArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&encode(&self.0))
    }
}

#[cfg(feature = "serdejson")]
impl<'de> Deserialize<'de> for ByteArray {
    fn deserialize<D>(deserializer: D) -> Result<ByteArray, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match decode(&s) {
            Ok(bin) => Ok(ByteArray(bin)),
            _ => Err(D::Error::custom("invalid base64")),
        }
    }
}

impl Deref for ByteArray {
    type Target = Vec<u8>;
    fn deref(&self) -> &Vec<u8> {
        &self.0
    }
}

impl DerefMut for ByteArray {
    fn deref_mut(&mut self) -> &mut Vec<u8> {
        &mut self.0
    }
}
