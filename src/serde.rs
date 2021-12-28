//! Serializer and Deserializers for OpenAPI Specification data formats

mod ser;
mod de;

/// Serializer and Deserializers for OpenAPI style=form data format
pub mod form {
    pub use super::ser::to_string;
    pub use super::de::{from_str, from_slice};
}

/// Serializer and Deserializers for OpenAPI style=simple data format
pub mod simple {
    pub use super::ser::to_string;
    pub use super::de::{from_str, from_slice};
}
