//! Support library for handling headers in a safe manner

use hyper::header::{Header, Headers};

/// Trait to add a mechanism to safely retrieve a header.
///
/// In Hyper 0.11, if you add an Authorization<Basic> header,
/// and then attempt get an Authorization<Bearer> header, the code
/// will panic, as the type ID doesn't match.
pub trait SafeHeaders {
    /// Safely get a header from a hyper::header::Headers
    fn safe_get<H: Header>(&self) -> Option<H>;
}

impl SafeHeaders for Headers {
    fn safe_get<H: Header>(&self) -> Option<H> {
        self.get_raw(H::header_name())
            .map(H::parse_header)
            .map(Result::ok)
            .unwrap_or(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::header::{Authorization, Bearer, Basic};

    #[test]
    fn test() {
        let mut headers = Headers::default();
        let basic = Basic {
            username: "richard".to_string(),
            password: None,
        };
        headers.set::<Authorization<Basic>>(Authorization(basic));
        println!("Auth: {:?}", headers.safe_get::<Authorization<Bearer>>());
    }
}
