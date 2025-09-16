//! Implementations of OpenAPI `oneOf` and `anyOf` types, assuming rules are just types
#[cfg(feature = "conversion")]
use frunk_enum_derive::LabelledGenericEnum;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
#[cfg(feature = "serdevalid")]
use serde_valid::Validate;
use std::fmt;
use std::str::FromStr;
use std::string::ToString;

#[cfg(feature = "serdejson")]
use serde_value::Value as SerdeValue;

// Define a macro to define the common parts between `OneOf` and `AnyOf` enums for a specific
// number of inner types.
macro_rules! common_one_any_of {
    (
        $schema:ident,
        $t:ident,
        $($i:ident),*
    ) => {
        #[doc = concat!("`", stringify!($t), "` type.\n\nThis allows modelling of ", stringify!($schema), " JSON schemas.")]
        #[cfg_attr(feature = "conversion", derive(LabelledGenericEnum))]
        #[cfg_attr(feature = "serdevalid", derive(Validate))]
        #[derive(Debug, PartialEq, Clone)]
        pub enum $t<$($i),*> where
            $($i: PartialEq,)*
        {
            $(
                #[doc = concat!("`", stringify!($i), "` variant of `", stringify!($t), "`")]
                $i($i)
            ),*
        }

        impl<$($i),*> Serialize for $t<$($i),*> where
            $($i: PartialEq + Serialize,)*
        {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                match self {
                    $(Self::$i(inner) => inner.serialize(serializer)),*
                }
            }
        }

        impl<$($i),*> fmt::Display for $t<$($i),*> where
            $($i: PartialEq + ToString,)*
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(Self::$i(inner) => write!(f, "{}", inner.to_string())),*
                }
            }
        }
    }
}

// Define a macro to define the `OneOf` enum for a specific number of inner types.
macro_rules! one_of {
    (
        $t:ident,
        $($i:ident),*
    ) => {
        common_one_any_of!(oneOf, $t, $($i),*);

        impl<'b, $($i),*> Deserialize<'b> for $t<$($i),*> where
            $($i: PartialEq + for<'a> Deserialize<'a>,)*
        {
            fn deserialize<De: Deserializer<'b>>(deserializer: De) -> Result<Self, De::Error> {
                // Capture once into a generic value (serde_value supports all JSON-like data)
                let captured: SerdeValue = SerdeValue::deserialize(deserializer)?;
                let mut result = Err(De::Error::custom("data did not match any within oneOf"));
                $(
                    if let Ok(inner) = <$i as Deserialize>::deserialize(captured.clone()) {
                        if result.is_err() {
                            result = Ok(Self::$i(inner));
                        } else {
                            return Err(De::Error::custom("data matched multiple within oneOf"))
                        }
                    }
                )*
                result
            }
        }

        impl<$($i),*> FromStr for $t<$($i),*> where
            $($i: PartialEq + FromStr,)*
        {
            type Err = &'static str;
            fn from_str(x: &str) -> Result<Self, Self::Err> {
                let mut result = Err("data did not match any within oneOf");
                $(
                    if let Ok(inner) = $i::from_str(x) {
                        if result.is_err() {
                            result = Ok(Self::$i(inner));
                        } else {
                            return Err("data matched multiple within oneOf")
                        }
                    }
                )*
                result
            }
        }
    }
}

// Use the `one_of!` macro to define the `OneOf` enum for 1-16 inner types.
one_of!(OneOf1, A);
one_of!(OneOf2, A, B);
one_of!(OneOf3, A, B, C);
one_of!(OneOf4, A, B, C, D);
one_of!(OneOf5, A, B, C, D, E);
one_of!(OneOf6, A, B, C, D, E, F);
one_of!(OneOf7, A, B, C, D, E, F, G);
one_of!(OneOf8, A, B, C, D, E, F, G, H);
one_of!(OneOf9, A, B, C, D, E, F, G, H, I);
one_of!(OneOf10, A, B, C, D, E, F, G, H, I, J);
one_of!(OneOf11, A, B, C, D, E, F, G, H, I, J, K);
one_of!(OneOf12, A, B, C, D, E, F, G, H, I, J, K, L);
one_of!(OneOf13, A, B, C, D, E, F, G, H, I, J, K, L, M);
one_of!(OneOf14, A, B, C, D, E, F, G, H, I, J, K, L, M, N);
one_of!(OneOf15, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
one_of!(OneOf16, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

// Define a macro to define the `AnyOf` enum for a specific number of inner types.
macro_rules! any_of {
    (
        $t:ident,
        $($i:ident),*
    ) => {
        common_one_any_of!(anyOf, $t, $($i),*);

        impl<'b, $($i),*> Deserialize<'b> for $t<$($i),*> where
            $($i: PartialEq + for<'a> Deserialize<'a>,)*
        {
            fn deserialize<De: Deserializer<'b>>(deserializer: De) -> Result<Self, De::Error> {
                let captured: SerdeValue = SerdeValue::deserialize(deserializer)?;
                $(
                    if let Ok(inner) = <$i as Deserialize>::deserialize(captured.clone()) {
                        return Ok(Self::$i(inner));
                    }
                )*
                Err(De::Error::custom("data did not match any within anyOf"))
            }
        }

        impl<$($i),*> FromStr for $t<$($i),*> where
            $($i: PartialEq + FromStr,)*
        {
            type Err = &'static str;
            fn from_str(x: &str) -> Result<Self, Self::Err> {
                $(
                    if let Ok(inner) = $i::from_str(x) {
                        return Ok(Self::$i(inner));
                    }
                )*
                Err("data did not match any within anyOf")
            }
        }
    }
}

// Use the `any_of!` macro to define the `AnyOf` enum for 1-16 inner types.
any_of!(AnyOf1, A);
any_of!(AnyOf2, A, B);
any_of!(AnyOf3, A, B, C);
any_of!(AnyOf4, A, B, C, D);
any_of!(AnyOf5, A, B, C, D, E);
any_of!(AnyOf6, A, B, C, D, E, F);
any_of!(AnyOf7, A, B, C, D, E, F, G);
any_of!(AnyOf8, A, B, C, D, E, F, G, H);
any_of!(AnyOf9, A, B, C, D, E, F, G, H, I);
any_of!(AnyOf10, A, B, C, D, E, F, G, H, I, J);
any_of!(AnyOf11, A, B, C, D, E, F, G, H, I, J, K);
any_of!(AnyOf12, A, B, C, D, E, F, G, H, I, J, K, L);
any_of!(AnyOf13, A, B, C, D, E, F, G, H, I, J, K, L, M);
any_of!(AnyOf14, A, B, C, D, E, F, G, H, I, J, K, L, M, N);
any_of!(AnyOf15, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
any_of!(AnyOf16, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anyof_prefers_first_matching_deserialize_number() {
        let json = "123";
        let v: AnyOf2<u32, String> = serde_json::from_str(json).unwrap();
        match v {
            AnyOf2::A(n) => assert_eq!(n, 123),
            AnyOf2::B(_) => panic!("expected A variant"),
        }
    }

    #[test]
    fn anyof_prefers_first_matching_fromstr_number() {
        let v = AnyOf2::<u8, String>::from_str("123").unwrap();
        match v {
            AnyOf2::A(n) => assert_eq!(n, 123),
            AnyOf2::B(_) => panic!("expected A variant"),
        }
    }

    #[test]
    fn anyof_deserialize_string_to_second_variant() {
        let json = "\"hello\"";
        let v: AnyOf2<u32, String> = serde_json::from_str(json).unwrap();
        match v {
            AnyOf2::B(s) => assert_eq!(s, "hello"),
            AnyOf2::A(_) => panic!("expected B variant"),
        }
    }

    #[test]
    fn oneof_deserialize_single_match() {
        let json = "\"hi\"";
        let v: OneOf2<u32, String> = serde_json::from_str(json).unwrap();
        match v {
            OneOf2::B(s) => assert_eq!(s, "hi"),
            OneOf2::A(_) => panic!("expected B variant"),
        }
    }

    #[test]
    fn oneof_deserialize_error_on_multiple_matches() {
        // both u32 and u64 will deserialize from "123" -> ambiguity -> error
        let json = "123";
        let res: Result<OneOf2<u32, u64>, _> = serde_json::from_str(json);
        assert!(res.is_err(), "expected error when multiple variants match");
    }

    #[test]
    fn oneof_fromstr_error_on_multiple_matches() {
        // String::from_str always succeeds and u8::from_str also succeeds here -> multiple matches
        let res = OneOf2::<u8, String>::from_str("123");
        assert!(res.is_err(), "expected error when multiple FromStr matches");
    }

    #[test]
    fn display_and_serialize_roundtrip() {
        let a: OneOf2<u32, String> = OneOf2::A(7u32);
        assert_eq!(a.to_string(), "7");
        let ser = serde_json::to_string(&a).unwrap();
        assert_eq!(ser, "7");

        let b: OneOf2<u32, String> = OneOf2::B(String::from("abc"));
        assert_eq!(b.to_string(), "abc");
        let ser_b = serde_json::to_string(&b).unwrap();
        assert_eq!(ser_b, "\"abc\"");
    }

    #[test]
    fn map_ambiguity_oneof() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct S1 {
            x: u32,
        }
        #[derive(Debug, PartialEq, Deserialize)]
        struct S2 {
            x: u64,
        }
        let json = "{\"x\":123}";
        let res: Result<OneOf2<S1, S2>, _> = serde_json::from_str(json);
        assert!(
            res.is_err(),
            "expected ambiguity error for map matching two structs"
        );
    }

    #[test]
    fn map_first_match_anyof() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct S1 {
            x: u32,
        }
        #[derive(Debug, PartialEq, Deserialize)]
        struct S2 {
            x: u64,
        }
        let json = "{\"x\":123}";
        let v: AnyOf2<S1, S2> = serde_json::from_str(json).unwrap();
        match v {
            AnyOf2::A(s) => assert_eq!(s.x, 123),
            AnyOf2::B(_) => panic!("expected first struct"),
        }
    }

    #[test]
    fn null_ambiguity_oneof() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct Unit;
        let json = "null";
        // Option<Unit> deserializes to None from null; Unit also deserializes from null? (No, unit struct expects object usually) -> To create ambiguity, use Option<String> and Option<u32> both None
        let res: Result<OneOf2<Option<u32>, Option<String>>, _> = serde_json::from_str(json);
        assert!(
            res.is_err(),
            "expected ambiguity with null across two Option types"
        );
    }

    #[test]
    fn null_preference_anyof() {
        let json = "null";
        let v: AnyOf2<Option<u32>, Option<String>> = serde_json::from_str(json).unwrap();
        match v {
            AnyOf2::A(opt) => assert!(opt.is_none()),
            AnyOf2::B(_) => panic!("expected first Option variant"),
        }
    }

    #[test]
    fn sequence_ambiguity_oneof() {
        let json = "[1,2]";
        let res: Result<OneOf2<Vec<u8>, Vec<u16>>, _> = serde_json::from_str(json);
        assert!(
            res.is_err(),
            "expected ambiguity for two vector numeric element types fitting both"
        );
    }

    #[test]
    fn sequence_first_match_anyof() {
        let json = "[1,2]";
        let v: AnyOf2<Vec<u8>, Vec<u16>> = serde_json::from_str(json).unwrap();
        match v {
            AnyOf2::A(v1) => assert_eq!(v1, vec![1, 2]),
            AnyOf2::B(_) => panic!("expected first vector variant"),
        }
    }

    #[test]
    fn large_number_anyof_prefers_second() {
        let json = "5000000000"; // > u32::MAX
        let v: AnyOf2<u32, u64> = serde_json::from_str(json).unwrap();
        match v {
            AnyOf2::B(n) => assert_eq!(n, 5_000_000_000u64),
            AnyOf2::A(_) => panic!("expected second variant since first fails"),
        }
    }

    #[test]
    fn oneof_deserialize_no_match_error() {
        // bool doesn't match u32 or String (needs quotes for string)
        let json = "true";
        let res: Result<OneOf2<u32, String>, _> = serde_json::from_str(json);
        assert!(res.is_err());
        let msg = format!("{}", res.unwrap_err());
        assert!(
            msg.contains("did not match any within oneOf"),
            "unexpected error message: {msg}"
        );
    }

    #[test]
    fn anyof_deserialize_no_match_error() {
        let json = "false"; // neither u32 nor String
        let res: Result<AnyOf2<u32, String>, _> = serde_json::from_str(json);
        assert!(res.is_err());
        let msg = format!("{}", res.unwrap_err());
        assert!(
            msg.contains("did not match any within anyOf"),
            "unexpected error message: {msg}"
        );
    }

    #[test]
    fn oneof_fromstr_single_match() {
        let v = OneOf2::<bool, u8>::from_str("true").expect("bool should parse");
        match v {
            OneOf2::A(b) => assert!(b),
            _ => panic!("expected bool variant"),
        }
    }

    #[test]
    fn oneof_fromstr_no_match() {
        let res = OneOf2::<u32, u16>::from_str("abc");
        assert!(res.is_err());
    }

    #[test]
    fn anyof_fromstr_later_match() {
        let v = AnyOf2::<u32, bool>::from_str("true").expect("bool should parse");
        match v {
            AnyOf2::B(b) => assert!(b),
            _ => panic!("expected second bool variant"),
        }
    }

    #[test]
    fn anyof_fromstr_no_match() {
        let res = AnyOf2::<u32, u16>::from_str("abc");
        assert!(res.is_err());
    }

    #[test]
    fn oneof_higher_arity_ambiguity() {
        // "5" (number) matches u8, u16, u32 -> ambiguity error
        let json = "5";
        let res: Result<OneOf3<u8, u16, u32>, _> = serde_json::from_str(json);
        assert!(res.is_err(), "expected ambiguity for three numeric matches");
    }

    #[test]
    fn anyof_higher_arity_middle_match() {
        // true fails u32, matches bool (middle), should select variant B
        let json = "true";
        let v: AnyOf3<u32, bool, String> = serde_json::from_str(json).unwrap();
        match v {
            AnyOf3::B(b) => assert!(b),
            _ => panic!("expected middle bool variant"),
        }
    }

    #[test]
    fn oneof_display_serialize_variant_c() {
        let c: OneOf3<String, u32, bool> = OneOf3::C(true);
        assert_eq!(c.to_string(), "true");
        let ser = serde_json::to_string(&c).unwrap();
        assert_eq!(ser, "true");
    }

    #[test]
    fn anyof_nested_structure_second_variant() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct A {
            x: u32,
        }
        #[derive(Debug, PartialEq, Deserialize)]
        struct B {
            y: u32,
        }
        let json = "{\"y\":5}";
        let v: AnyOf3<A, B, String> = serde_json::from_str(json).unwrap();
        match v {
            AnyOf3::B(b) => assert_eq!(b.y, 5),
            _ => panic!("expected struct B variant"),
        }
    }

    #[test]
    fn oneof_ambiguity_error_message() {
        let json = "123"; // matches multiple integer widths
        let res: Result<OneOf2<u32, u64>, _> = serde_json::from_str(json);
        let err = res.unwrap_err();
        let msg = format!("{}", err);
        assert!(
            msg.contains("data matched multiple within oneOf"),
            "missing ambiguity message: {msg}"
        );
    }
}
