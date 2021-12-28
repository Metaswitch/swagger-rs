use serde::Serialize;
use std::io::Write;

type Ok = ();

use thiserror::Error;

/// Serializer error
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// Underlying IO error serializing data
    #[error("IO Error: {0}")]
    Io(#[source] std::io::Error),
    /// Unable to serialize/deserialize arbitrary enum types - format is not sufficient
    #[error("Unsupported enum type")]
    UnsupportedEnumType,
    /// Error from serialize, from serde::se::Error::custom()
    #[error("{0}")]
    Custom(String),
}

type Result<T> = std::result::Result<T, Error>;
impl serde::ser::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Custom(format!("{}", msg))
    }
}

struct SerializerField<'a> {
    writer: &'a mut Vec<u8>,
    first: bool,
}

impl<'a> SerializerField<'a> {
    fn new(writer: &'a mut Vec<u8>) -> Self {
        Self {
            writer,
            first: true,
        }
    }
}

impl<'a> serde::ser::SerializeSeq for SerializerField<'a> {
    type Ok = Ok;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        if !self.first {
            self.writer.write(",".as_bytes()).map_err(Error::Io)?;
        }
        let mut s = Serializer {
            writer: self.writer,
        };
        value.serialize(&mut s)?;
        self.first = false;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeTuple for SerializerField<'a> {
    type Ok = Ok;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeTupleStruct for SerializerField<'a> {
    type Ok = Ok;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeTupleVariant for SerializerField<'a> {
    type Ok = Ok;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<()> {
        unimplemented!()
        //serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!()
        //Ok(())
    }
}

impl<'a> serde::ser::SerializeMap for SerializerField<'a> {
    type Ok = Ok;
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<()> {
        serde::ser::SerializeSeq::serialize_element(self, key)
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeStruct for SerializerField<'a> {
    type Ok = Ok;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        serde::ser::SerializeSeq::serialize_element(self, key)?;
        serde::ser::SerializeSeq::serialize_element(self, value)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a> serde::ser::SerializeStructVariant for SerializerField<'a> {
    type Ok = Ok;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<()> {
        unimplemented!()
        // serde::ser::SerializeSeq::serialize_element(self, key)?;
        // serde::ser::SerializeSeq::serialize_element(self, value)?;
        // Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        unimplemented!();
        // Ok(())
    }
}

struct Serializer<'a> {
    writer: &'a mut Vec<u8>,
}

impl<'a> serde::Serializer for &'a mut Serializer<'a> {
    type Ok = Ok;
    type Error = Error;
    type SerializeSeq = SerializerField<'a>;
    type SerializeTuple = SerializerField<'a>;
    type SerializeTupleStruct = SerializerField<'a>;
    type SerializeTupleVariant = SerializerField<'a>;
    type SerializeMap = SerializerField<'a>;
    type SerializeStruct = SerializerField<'a>;
    type SerializeStructVariant = SerializerField<'a>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        write!(self.writer, "{}", v).map_err(Error::Io)
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        self.writer
            .write(urlencoding::encode_binary(v).as_bytes())
            .map_err(Error::Io)?;
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        write!(self.writer, "{}", v).map_err(Error::Io)
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        write!(self.writer, "{}", v).map_err(Error::Io)
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        write!(self.writer, "{}", v).map_err(Error::Io)
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        write!(self.writer, "{}", v).map_err(Error::Io)
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        write!(self.writer, "{}", v).map_err(Error::Io)
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        write!(self.writer, "{}", v).map_err(Error::Io)
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        write!(self.writer, "{}", v).map_err(Error::Io)
    }
    fn serialize_i128(self, v: i128) -> Result<Self::Ok> {
        write!(self.writer, "{}", v).map_err(Error::Io)
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializerField::new(self.writer))
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok> {
        Err(Error::UnsupportedEnumType)
    }
    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(())
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializerField::new(self.writer))
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok> {
        value.serialize(self)
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.writer
            .write(urlencoding::encode(v).as_bytes())
            .map_err(Error::Io)?;
        Ok(())
    }
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializerField::new(self.writer))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::UnsupportedEnumType)
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(SerializerField::new(self.writer))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(SerializerField::new(self.writer))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::UnsupportedEnumType)
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        write!(self.writer, "{}", v).map_err(Error::Io)
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        write!(self.writer, "{}", v).map_err(Error::Io)
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        write!(self.writer, "{}", v).map_err(Error::Io)
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        write!(self.writer, "{}", v).map_err(Error::Io)
    }
    fn serialize_u128(self, v: u128) -> Result<Self::Ok> {
        write!(self.writer, "{}", v).map_err(Error::Io)
    }
    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.writer.write(variant.as_bytes()).map_err(Error::Io)?;
        Ok(())
    }
}

/// Serialize an OpenAPI parameter in the style=form,explode=false variant.
///
/// See https://spec.openapis.org/oas/v3.1.0.html#style-examples
///
/// Note, nested structures in this form are likely to be ambiguous when decoding.
/// Consider another encoding style, such as using JSON, or a different parameter
/// encoding style
pub fn to_string<T: ?Sized + Serialize>(value: &T) -> Result<String> {
    let mut vec = Vec::with_capacity(128);
    let mut ser = Serializer { writer: &mut vec };
    value.serialize(&mut ser)?;
    Ok(String::from_utf8(vec).expect("Array should always be valid UTF-8"))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    // Style: form. Explode = false

    #[test]
    fn test_to_string_empty_none() {
        let s: Option<()> = None;
        assert_eq!(&super::to_string(&s).unwrap(), "");
    }

    #[test]
    fn test_to_string_empty_unit() {
        let s = ();
        assert_eq!(&super::to_string(&s).unwrap(), "");
    }

    #[test]
    /// As per <https://spec.openapis.org/oas/latest.html#style-examples>
    fn test_to_string_some_string() {
        let s = Some("blue");
        assert_eq!(&super::to_string(&s).unwrap(), "blue");
    }

    #[test]
    fn test_to_string_string() {
        let s = "blue";
        assert_eq!(&super::to_string(&s).unwrap(), "blue");
    }

    #[test]
    /// As per <https://spec.openapis.org/oas/latest.html#style-examples>
    fn test_to_string_array() {
        let s = vec!["blue", "black", "brown"];
        assert_eq!(
            &super::to_string(&s).unwrap(),
            "blue,black,brown"
        );
    }

    #[derive(Debug, serde::Serialize)]
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
    fn test_to_string_object() {
        let s = Object {
            r: 100,
            g: 200,
            b: 150,
        };
        assert_eq!(
            &super::to_string(&s).unwrap(),
            "R,100,G,200,B,150"
        );
    }

    #[test]
    fn test_to_string_map() {
        let mut s = HashMap::new();
        s.insert("R", 100);
        s.insert("G", 200);
        s.insert("B", 150);
        // Can't check answer because order isn't well defined.
        super::to_string(&s).unwrap();
    }

    #[derive(Debug, serde::Serialize)]
    struct Escaped {
        semi: &'static str,
        dot: &'static str,
        comma: &'static str,
    }

    #[test]
    /// As per RFC 6570, Section 1.2 / 3.2.8 / 3.2.9
    fn test_to_string_escaped() {
        let s = Escaped {
            semi: ";",
            dot: ".",
            comma: ",",
        };
        assert_eq!(
            &super::to_string(&s).unwrap(),
            "semi,%3B,dot,.,comma,%2C"
        );
    }

    #[derive(Debug, serde::Serialize, PartialEq)]
    #[allow(dead_code)]
    enum SampleEnum {
        Yes,
        No,
    }

    #[test]
    fn test_to_string_enum() {
        assert_eq!(
            &super::to_string(&SampleEnum::Yes).unwrap(),
            "Yes"
        );
    }
}
