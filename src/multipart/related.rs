//! Helper functions for multipart/related support

use http::header::{HeaderValue, CONTENT_TYPE};
use http::HeaderMap;
use mime::Mime;

/// Construct the Body for a multipart/related request. The mime 0.2.6 library
/// does not parse quoted-string parameters correctly. The boundary doesn't
/// need to be a quoted string if it does not contain a '/', hence ensure
/// no such boundary is used.
pub fn generate_boundary() -> Vec<u8> {
    let mut boundary = mime_multipart::generate_boundary();
    for b in boundary.iter_mut() {
        if *b == b'/' {
            *b = b'.';
        }
    }

    boundary
}

/// Create the multipart headers from a request so that we can parse the
/// body using `mime_multipart::read_multipart_body`.
pub fn create_multipart_headers(content_type: Option<&HeaderValue>) -> Result<HeaderMap, String> {
    let content_type = content_type
        .ok_or_else(|| "Missing Content-Type header".to_string())?
        .to_str()
        .map_err(|e| format!("Couldn't read Content-Type header value: {}", e))?
        .parse::<Mime>()
        .map_err(|_e| "Couldn't parse Content-Type header value".to_string())?;
    // Insert top-level content type header into a Headers object.
    let mut multipart_headers = HeaderMap::new();
    multipart_headers.append(
        CONTENT_TYPE,
        HeaderValue::from_str(content_type.as_ref())
            .map_err(|_e| "Couldn't parse Content-Type header value".to_string())?,
    );

    Ok(multipart_headers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mime_multipart::Node;

    // Test that we can parse the body using read_multipart_body
    #[test]
    fn test_create_multipart_headers_valid_read_multipart_body() {
        let content_type = HeaderValue::from_static("multipart/related; boundary=example");
        let headers = create_multipart_headers(Some(&content_type)).unwrap();

        let body: &[u8] =
            b"--example\r\nContent-Type: text/plain\r\n\r\nHello, World!\r\n--example--";
        let res = mime_multipart::read_multipart_body(&mut &body[..], &headers, false);
        // Check our content types are valid
        match res.unwrap().first().unwrap() {
            Node::Part(h) => {
                assert_eq!(
                    h.headers.get(CONTENT_TYPE).unwrap(),
                    &HeaderValue::from_static("text/plain")
                );
            }
            _ => panic!("Expected Node::Multipart"),
        }
    }

    #[test]
    fn test_create_multipart_headers_valid() {
        let content_type = HeaderValue::from_static("multipart/related; boundary=example");
        let headers = create_multipart_headers(Some(&content_type)).unwrap();
        assert_eq!(
            headers.get(CONTENT_TYPE).unwrap(),
            &HeaderValue::from_static("multipart/related; boundary=example")
        );
    }

    #[test]
    fn test_create_multipart_headers_missing_content_type() {
        let result = create_multipart_headers(None);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Missing Content-Type header");
    }

    #[test]
    fn test_create_multipart_headers_invalid_content_type() {
        let content_type = HeaderValue::from_static("invalid-content-type");
        let result = create_multipart_headers(Some(&content_type));
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            "Couldn't parse Content-Type header value"
        );
    }

    #[test]
    fn test_create_multipart_headers_non_utf8_content_type() {
        let content_type = HeaderValue::from_bytes(b"\xFF\xFF\xFF").unwrap();
        let result = create_multipart_headers(Some(&content_type));
        assert!(result.is_err());
        assert!(result
            .err()
            .unwrap()
            .contains("Couldn't read Content-Type header value"));
    }
}
