use serde::ser::Serializer;
use serde::de::{Deserialize, Deserializer, Error};
use base64::{encode, decode};

/// Seralize an object as base64.
pub fn serialize_with<S>(obj: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&encode(obj))
}

/// Deserialize an object from base64.
pub fn deserialize_with<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = try!(String::deserialize(deserializer));
    match decode(&s) {
        Ok(bin) => Ok(bin),
        _ => Err(D::Error::custom("invalid base64")),
    }
}
