use serde::{de::DeserializeSeed, de::Visitor, Deserialize};

use thiserror::Error;
/// Deserializer error
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// Missing expected key=value pairing
    #[error("Missing key=value pairing")]
    MissingKeyValue,
    /// Non-UTF-8 data found prior to URL decoding
    #[error("Non-UTF-8 data found prior to URL decoding")]
    OuterUtf8(#[source] std::str::Utf8Error),
    /// Non-UTF-8 data found during URL decoding
    #[error("Non-UTF-8 data found during URL decoding")]
    InnerUtf8(#[source] std::string::FromUtf8Error),
    /// Underlying IO error serializing data
    #[error("IO Error: {0}")]
    Io(#[source] std::io::Error),
    /// Tried to parse a unit, but text was present
    #[error("Non-empty unit")]
    NonEmptyUnit,
    /// An object contained a key, but no value. Note, an object may be any key value pairing (e.g. a struct or a map).
    #[error("Missing value in key/value pairing")]
    MissingValueForObject,
    /// Expected a float when deserializing, but didn't find one
    #[error("Expected a float when parsing")]
    ExpectedFloat(std::num::ParseFloatError),
    /// Expected a integer when deserializing, but didn't find one
    #[error("Expected a integer when parsing")]
    ExpectedInt(std::num::ParseIntError),
    /// Multiple characters encountered when parsing a single char
    #[error("Got multiple characters when only expecting a single char - got: {0}")]
    MultiCharacterChar(char),
    /// No characters encountered when parsing a single char
    #[error("Got no characters when expecting a single char")]
    EmptyChar,
    /// Unable to parse without knowing expected type - format is not self describing.
    #[error("Unable to parse without knowing expected type")]
    AnyTypeUnsupported,
    /// Unable to serialize/deserialize arbitrary enum types - format is not sufficient
    #[error("Unsupported enum type")]
    UnsupportedEnumType,
    /// Invalid bool - expecting true or false
    #[error("Invalid boolean - expected true or false - got: {0}")]
    InvalidBool(String),
    /// Error from deserializer, from serde::de::Error::custom()
    #[error("{0}")]
    Custom(String),
}

type Result<T> = std::result::Result<T, Error>;
impl serde::de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Custom(format!("{}", msg))
    }
}

struct DeserializerField<'a>(std::str::Split<'a, char>);

impl<'a> serde::de::SeqAccess<'a> for DeserializerField<'a> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'a>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        match self.0.next() {
            Some(s) => {
                let mut d = Deserializer(s.as_bytes());
                Ok(Some(seed.deserialize(&mut d)?))
            }
            None => Ok(None),
        }
    }
}

impl<'a> serde::de::MapAccess<'a> for DeserializerField<'a> {
    type Error = Error;

    fn next_key_seed<T: DeserializeSeed<'a>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        serde::de::SeqAccess::next_element_seed(self, seed)
    }

    fn next_value_seed<T: DeserializeSeed<'a>>(&mut self, seed: T) -> Result<T::Value> {
        serde::de::SeqAccess::next_element_seed(self, seed)?.ok_or(Error::MissingValueForObject)
    }
}

struct Deserializer<'a>(&'a [u8]);

impl<'a, 'b> serde::de::VariantAccess<'a> for &'b mut Deserializer<'a> {
    type Error = Error;
    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T: DeserializeSeed<'a>>(self, _seed: T) -> Result<T::Value> {
        Err(Error::UnsupportedEnumType)
    }
    fn tuple_variant<V: Visitor<'a>>(self, _len: usize, _visitor: V) -> Result<V::Value> {
        Err(Error::UnsupportedEnumType)
    }
    fn struct_variant<V: Visitor<'a>>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value> {
        Err(Error::UnsupportedEnumType)
    }
}

impl<'a, 'b> serde::de::EnumAccess<'a> for &'b mut Deserializer<'a> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V: DeserializeSeed<'a>>(self, seed: V) -> Result<(V::Value, Self::Variant)> {
        let v = seed.deserialize(&mut *self)?;
        Ok((v, self))
    }
}

impl<'a, 'b> serde::de::Deserializer<'a> for &'b mut Deserializer<'a> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'a>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::AnyTypeUnsupported)
    }

    fn deserialize_bool<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        let buf = std::str::from_utf8(self.0).map_err(Error::OuterUtf8)?;
        visitor.visit_bool(match buf {
            "true" => true,
            "false" => false,
            _ => {
                return Err(Error::InvalidBool(buf.to_owned()));
            }
        })
    }

    fn deserialize_byte_buf<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        let buf = urlencoding::decode_binary(self.0);
        use std::borrow::Cow::*;
        match buf {
            Borrowed(buf) => visitor.visit_borrowed_bytes(buf),
            Owned(buf) => visitor.visit_byte_buf(buf),
        }
    }

    fn deserialize_bytes<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        let buf = urlencoding::decode_binary(self.0);
        use std::borrow::Cow::*;
        match buf {
            Borrowed(buf) => visitor.visit_borrowed_bytes(buf),
            Owned(buf) => visitor.visit_byte_buf(buf),
        }
    }

    fn deserialize_char<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        let buf = std::str::from_utf8(self.0).map_err(Error::OuterUtf8)?;
        use std::borrow::Cow::*;
        let buf = if buf.contains('%') {
            urlencoding::decode(buf).map_err(Error::InnerUtf8)?
        } else {
            Borrowed(buf)
        };
        let mut chars = buf.chars();
        let c = chars.next();
        if let Some(c) = c {
            if let Some(cb) = chars.next() {
                return Err(Error::MultiCharacterChar(cb));
            }
            visitor.visit_char(c)
        } else {
            Err(Error::EmptyChar)
        }
    }

    fn deserialize_enum<V: Visitor<'a>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_enum(self)
    }

    fn deserialize_f32<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        let buf = std::str::from_utf8(self.0).map_err(Error::OuterUtf8)?;
        visitor.visit_f32(buf.parse().map_err(Error::ExpectedFloat)?)
    }

    fn deserialize_f64<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        let buf = std::str::from_utf8(self.0).map_err(Error::OuterUtf8)?;
        visitor.visit_f64(buf.parse().map_err(Error::ExpectedFloat)?)
    }

    fn deserialize_i8<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        let buf = std::str::from_utf8(self.0).map_err(Error::OuterUtf8)?;
        visitor.visit_i8(buf.parse().map_err(Error::ExpectedInt)?)
    }

    fn deserialize_i16<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        let buf = std::str::from_utf8(self.0).map_err(Error::OuterUtf8)?;
        visitor.visit_i16(buf.parse().map_err(Error::ExpectedInt)?)
    }

    fn deserialize_i32<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        let buf = std::str::from_utf8(self.0).map_err(Error::OuterUtf8)?;
        visitor.visit_i32(buf.parse().map_err(Error::ExpectedInt)?)
    }

    fn deserialize_i64<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        let buf = std::str::from_utf8(self.0).map_err(Error::OuterUtf8)?;
        visitor.visit_i64(buf.parse().map_err(Error::ExpectedInt)?)
    }

    fn deserialize_i128<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        let buf = std::str::from_utf8(self.0).map_err(Error::OuterUtf8)?;
        visitor.visit_i128(buf.parse().map_err(Error::ExpectedInt)?)
    }

    fn deserialize_identifier<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'a>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::AnyTypeUnsupported)
    }

    fn deserialize_map<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        let buf = std::str::from_utf8(self.0).map_err(Error::OuterUtf8)?;
        visitor.visit_map(DeserializerField(buf.split(',')))
    }

    fn deserialize_newtype_struct<V: Visitor<'a>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        let buf = std::str::from_utf8(self.0).map_err(Error::OuterUtf8)?;
        visitor.visit_map(DeserializerField(buf.split(',')))
    }

    fn deserialize_option<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        if self.0.is_empty() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_seq<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        let buf = std::str::from_utf8(self.0).map_err(Error::OuterUtf8)?;
        visitor.visit_seq(DeserializerField(buf.split(',')))
    }

    fn deserialize_str<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        let buf = std::str::from_utf8(self.0).map_err(Error::OuterUtf8)?;
        use std::borrow::Cow::*;
        let buf = if buf.contains('%') {
            urlencoding::decode(buf).map_err(Error::InnerUtf8)?
        } else {
            Borrowed(buf)
        };
        match buf {
            Borrowed(s) => visitor.visit_borrowed_str(s),
            Owned(s) => visitor.visit_string(s),
        }
    }

    fn deserialize_string<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_struct<V: Visitor<'a>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        let buf = std::str::from_utf8(self.0).map_err(Error::OuterUtf8)?;
        visitor.visit_map(DeserializerField(buf.split(',')))
    }

    fn deserialize_tuple<V: Visitor<'a>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        let buf = std::str::from_utf8(self.0).map_err(Error::OuterUtf8)?;
        visitor.visit_seq(DeserializerField(buf.split(',')))
    }

    fn deserialize_tuple_struct<V: Visitor<'a>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        let buf = std::str::from_utf8(self.0).map_err(Error::OuterUtf8)?;
        visitor.visit_seq(DeserializerField(buf.split(',')))
    }

    fn deserialize_u8<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        let buf = std::str::from_utf8(self.0).map_err(Error::OuterUtf8)?;
        visitor.visit_u8(buf.parse().map_err(Error::ExpectedInt)?)
    }

    fn deserialize_u16<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        let buf = std::str::from_utf8(self.0).map_err(Error::OuterUtf8)?;
        visitor.visit_u16(buf.parse().map_err(Error::ExpectedInt)?)
    }

    fn deserialize_u32<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        let buf = std::str::from_utf8(self.0).map_err(Error::OuterUtf8)?;
        visitor.visit_u32(buf.parse().map_err(Error::ExpectedInt)?)
    }

    fn deserialize_u64<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        let buf = std::str::from_utf8(self.0).map_err(Error::OuterUtf8)?;
        visitor.visit_u64(buf.parse().map_err(Error::ExpectedInt)?)
    }

    fn deserialize_u128<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        let buf = std::str::from_utf8(self.0).map_err(Error::OuterUtf8)?;
        visitor.visit_u128(buf.parse().map_err(Error::ExpectedInt)?)
    }

    fn deserialize_unit<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        if !self.0.is_empty() {
            return Err(Error::NonEmptyUnit);
        }
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V: Visitor<'a>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        let buf = std::str::from_utf8(self.0).map_err(Error::OuterUtf8)?;
        visitor.visit_map(DeserializerField(buf.split(',')))
    }
}

/// Deserialize an OpenAPI parameter in the style=form,explode=false variant.
///
/// See https://spec.openapis.org/oas/v3.1.0.html#style-examples
///
/// Note, nested structures in this form are likely to be ambiguous when decoding.
/// Consider another encoding style, such as using JSON, or a different parameter
/// encoding style
pub fn from_slice<'a, T: Deserialize<'a>>(value: &'a [u8]) -> Result<T> {
    let mut de = Deserializer(value);
    let value = Deserialize::deserialize(&mut de)?;
    Ok(value)
}

/// Deserialize an OpenAPI parameter in the style=form,explode=false variant.
///
/// See https://spec.openapis.org/oas/v3.1.0.html#style-examples
///
/// Note, nested structures in this form are likely to be ambiguous when decoding.
/// Consider another encoding style, such as using JSON, or a different parameter
/// encoding style
pub fn from_str<'a, T: Deserialize<'a>>(value: &'a str) -> Result<T> {
    let mut de = Deserializer(value.as_bytes());
    let value = Deserialize::deserialize(&mut de)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::from_str;

    #[test]
    fn test_from_str_string() {
        assert_eq!("blue", from_str::<&str>("blue").unwrap());
    }

    #[test]
    fn test_from_str_none() {
        assert_eq!(None, from_str::<Option<()>>("").unwrap());
    }

    #[test]
    fn test_from_str_empty_unit() {
        assert_eq!((), from_str("").unwrap());
    }

    #[test]
    fn test_from_str_some_string() {
        assert_eq!(Some("blue"), from_str("blue").unwrap());
    }
    #[test]
    /// As per <https://spec.openapis.org/oas/latest.html#style-examples>
    fn test_from_str_array() {
        assert_eq!(
            vec!["blue", "black", "brown"],
            from_str::<Vec<&str>>("blue,black,brown").unwrap(),
        );
    }

    #[derive(Debug, serde::Deserialize, PartialEq)]
    struct Object {
        #[serde(rename = "R")]
        r: i32,
        #[serde(rename = "G")]
        g: i32,
        #[serde(rename = "B")]
        b: i32,
    }

    #[test]
    /// As per <https://spec.openapis.org/oas/latest.html#style-examples>
    fn test_from_str_object() {
        let s = Object {
            r: 100,
            g: 200,
            b: 150,
        };
        assert_eq!(s, from_str("R,100,G,200,B,150").unwrap(),);
    }

    #[test]
    fn test_from_str_map() {
        let mut s = std::collections::HashMap::new();
        s.insert("R", 100);
        s.insert("G", 200);
        s.insert("B", 150);
        assert_eq!(s, from_str("R,100,G,200,B,150").unwrap());
    }

    #[derive(Debug, serde::Deserialize, PartialEq)]
    struct Escaped {
        semi: String,
        dot: &'static str,
        comma: String,
    }

    #[test]
    /// As per RFC 6570, Section 1.2 / 3.2.8 / 3.2.9
    fn test_from_str_escaped() {
        let s = Escaped {
            semi: ";".to_string(),
            dot: ".",
            comma: ",".to_string(),
        };
        assert_eq!(
            s,
            from_str("semi,%3B,dot,.,comma,%2C").unwrap()
        );
    }

    #[derive(Debug, serde::Deserialize, PartialEq)]
    enum SampleEnum {
        Yes,
        No,
    }

    #[test]
    fn test_from_str_enum() {
        let s = SampleEnum::Yes;
        assert_eq!(s, from_str("Yes").unwrap())
    }
}
