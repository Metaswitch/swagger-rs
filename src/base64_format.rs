#[cfg(feature = "serdejson")]
use base64::{decode, encode, DecodeError};
#[cfg(feature = "serdejson")]
use serde::de::{Deserialize, Deserializer, Error};
#[cfg(feature = "serdejson")]
use serde::ser::{Serialize, Serializer};
use std::ops::{Deref, DerefMut};

#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
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

impl std::str::FromStr for ByteArray {
    type Err = DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(decode(s)?))
    }
}

impl ToString for ByteArray {
    fn to_string(&self) -> String {
        encode(&self.0)
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
