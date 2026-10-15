#![cfg(feature = "serdevalid")]

use serde_valid::Validate;
use swagger::{AnyOf2, OneOf2};

#[derive(Debug, PartialEq, Validate)]
struct Positive {
    #[validate(minimum = 1)]
    value: i32,
}

#[test]
fn oneof_validates_the_selected_member() {
    for value in [0, 1] {
        for union in [
            OneOf2::<Positive, Positive>::A(Positive { value }),
            OneOf2::B(Positive { value }),
        ] {
            assert_eq!(union.validate().is_ok(), value == 1);
        }
    }
}

#[test]
fn anyof_validates_the_selected_member() {
    for value in [0, 1] {
        for union in [
            AnyOf2::<Positive, Positive>::A(Positive { value }),
            AnyOf2::B(Positive { value }),
        ] {
            assert_eq!(union.validate().is_ok(), value == 1);
        }
    }
}

#[test]
fn unions_with_primitive_members_still_serialize_and_deserialize() {
    let oneof: OneOf2<u32, String> = serde_json::from_str("\"future\"").unwrap();
    assert_eq!(serde_json::to_string(&oneof).unwrap(), "\"future\"");
    let anyof: AnyOf2<u32, String> = serde_json::from_str("42").unwrap();
    assert_eq!(serde_json::to_string(&anyof).unwrap(), "42");
}
