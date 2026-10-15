#![cfg(feature = "serdevalid")]

use serde_valid::Validate;
use swagger::Nullable;

#[derive(Validate)]
struct Collection {
    #[validate(min_items = 1)]
    #[validate(max_items = 2)]
    values: Nullable<Vec<String>>,
}

#[derive(Validate)]
struct OptionalCollection {
    #[validate(min_items = 1)]
    #[validate(max_items = 2)]
    values: Option<Nullable<Vec<String>>>,
}

#[test]
fn nullable_collection_bounds_apply_to_present_values() {
    assert!(Collection {
        values: Nullable::Null
    }
    .validate()
    .is_ok());
    assert!(OptionalCollection { values: None }.validate().is_ok());
    assert!(OptionalCollection {
        values: Some(Nullable::Null)
    }
    .validate()
    .is_ok());
    for size in 0..=3 {
        let values = vec![String::new(); size];
        assert_eq!(
            Collection {
                values: Nullable::Present(values.clone())
            }
            .validate()
            .is_ok(),
            (1..=2).contains(&size)
        );
        assert_eq!(
            OptionalCollection {
                values: Some(Nullable::Present(values))
            }
            .validate()
            .is_ok(),
            (1..=2).contains(&size)
        );
    }
}
