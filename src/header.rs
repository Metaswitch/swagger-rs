use chrono::{DateTime, Utc};
use hyper::header::HeaderValue;
use std::fmt;
use std::ops::Deref;
use uuid::Uuid;

/// Header - `X-Span-ID` - used to track a request through a chain of microservices.
pub const X_SPAN_ID: &str = "X-Span-ID";

/// Wrapper for a string being used as an X-Span-ID.
#[derive(Debug, Clone)]
pub struct XSpanIdString(pub String);

impl XSpanIdString {
    /// Extract an X-Span-ID from a request header if present, and if not
    /// generate a new one.
    pub fn get_or_generate<T>(req: &hyper::Request<T>) -> Self {
        let x_span_id = req.headers().get(X_SPAN_ID);

        x_span_id
            .map(|x| x.to_str().ok())
            .flatten()
            .map(|x| XSpanIdString(x.to_string()))
            .unwrap_or_default()
    }
}

impl Default for XSpanIdString {
    fn default() -> Self {
        XSpanIdString(Uuid::new_v4().to_string())
    }
}

impl fmt::Display for XSpanIdString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A struct to allow homogeneous conversion into a HeaderValue. We can't
/// implement the From/Into trait on HeaderValue because we don't own
/// either of the types.
#[derive(Debug, Clone)]
pub struct IntoHeaderValue<T>(pub T);

// Generic implementations

impl<T> Deref for IntoHeaderValue<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

// Derive for each From<T> in hyper::header::HeaderValue

macro_rules! ihv_generate {
    ($t:ident) => {
        impl From<HeaderValue> for IntoHeaderValue<$t> {
            fn from(hdr_value: HeaderValue) -> Self {
                IntoHeaderValue(hdr_value.to_str().unwrap().parse::<$t>().unwrap())
            }
        }

        impl From<IntoHeaderValue<$t>> for HeaderValue {
            fn from(hdr_value: IntoHeaderValue<$t>) -> Self {
                hdr_value.0.into()
            }
        }
    };
}

ihv_generate!(u64);
ihv_generate!(i64);
ihv_generate!(i16);
ihv_generate!(u16);
ihv_generate!(u32);
ihv_generate!(usize);
ihv_generate!(isize);
ihv_generate!(i32);

// Custom derivations

impl From<HeaderValue> for IntoHeaderValue<Vec<String>> {
    fn from(hdr_value: HeaderValue) -> Self {
        IntoHeaderValue(
            hdr_value
                .to_str()
                .unwrap()
                .split(',')
                .filter_map(|x| match x.trim() {
                    "" => None,
                    y => Some(y.to_string()),
                })
                .collect(),
        )
    }
}

impl From<IntoHeaderValue<Vec<String>>> for HeaderValue {
    fn from(hdr_value: IntoHeaderValue<Vec<String>>) -> Self {
        HeaderValue::from_str(&hdr_value.0.join(", ")).unwrap()
    }
}

impl From<HeaderValue> for IntoHeaderValue<String> {
    fn from(hdr_value: HeaderValue) -> Self {
        IntoHeaderValue(hdr_value.to_str().unwrap().to_string())
    }
}

impl From<IntoHeaderValue<String>> for HeaderValue {
    fn from(hdr_value: IntoHeaderValue<String>) -> Self {
        HeaderValue::from_str(&hdr_value.0).unwrap()
    }
}

impl From<HeaderValue> for IntoHeaderValue<DateTime<Utc>> {
    fn from(hdr_value: HeaderValue) -> Self {
        IntoHeaderValue(
            DateTime::parse_from_rfc3339(hdr_value.to_str().unwrap())
                .unwrap()
                .with_timezone(&Utc),
        )
    }
}

impl From<IntoHeaderValue<DateTime<Utc>>> for HeaderValue {
    fn from(hdr_value: IntoHeaderValue<DateTime<Utc>>) -> Self {
        HeaderValue::from_str(hdr_value.0.to_rfc3339().as_str()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::XSpanIdString;
    use hyper::Request;

    /// Create a request with the given X-Span-ID.
    fn create_xspan_id_req(id: &'static str) -> Request<()> {
        Request::builder().header("X-Span-ID", id).body(()).unwrap()
    }

    #[test]
    fn xspanid_test() {
        XSpanIdString::get_or_generate(&create_xspan_id_req("£"));

        let valid_uuid = "123e4567-e89b-12d3-a456-426655440000";

        assert_eq!(
            valid_uuid,
            &XSpanIdString::get_or_generate(&create_xspan_id_req(valid_uuid)).0
        );
    }
}
