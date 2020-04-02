//! Implementations of OpenAPI `oneOf` and `anyOf` types, assuming rules are just types
use serde::{
    de::Error,
    private::de::{Content, ContentRefDeserializer},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::str::FromStr;
use std::string::ToString;

// Define a macro to define the common parts between `OneOf` and `AnyOf` enums for a specific
// number of inner types.
macro_rules! common_one_any_of {
    (
        $t:ident,
        $($i:ident),*
    ) => {
        /// $t
        #[derive(Debug, PartialEq, Clone)]
        pub enum $t<$($i),*> where
            $($i: PartialEq,)*
        {
            $(
                /// $i type
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

        impl<$($i),*> ToString for $t<$($i),*> where
            $($i: PartialEq + ToString,)*
        {
            fn to_string(&self) -> String {
                match self {
                    $(Self::$i(inner) => inner.to_string()),*
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
        common_one_any_of!($t, $($i),*);

        impl<'b, $($i),*> Deserialize<'b> for $t<$($i),*> where
            $($i: PartialEq + for<'a> Deserialize<'a>,)*
        {
            fn deserialize<De: Deserializer<'b>>(deserializer: De) -> Result<Self, De::Error> {
                let content = Content::deserialize(deserializer)?;
                let mut result = Err(De::Error::custom("data did not match any within oneOf"));
                $(
                    if let Ok(inner) = $i::deserialize(ContentRefDeserializer::<De::Error>::new(&content)) {
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
one_of!(OneOf12, A, B, C, D, E, F, G, H, I, J, K);
one_of!(OneOf13, A, B, C, D, E, F, G, H, I, J, K, L);
one_of!(OneOf14, A, B, C, D, E, F, G, H, I, J, K, L, M);
one_of!(OneOf15, A, B, C, D, E, F, G, H, I, J, K, L, M, N);
one_of!(OneOf16, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);

#[cfg(test)]
mod one_of_tests {
    use super::OneOf2;
    type TestOneOf = OneOf2<u32, String>;

    #[test]
    fn serialize_one_of_a() {
        assert_eq!(serde_json::to_string(&TestOneOf::A(123)).unwrap(), "123");
    }
    #[test]
    fn serialize_one_of_b() {
        assert_eq!(
            serde_json::to_string(&TestOneOf::B("hello".to_string())).unwrap(),
            "\"hello\""
        );
    }

    #[test]
    fn to_string_one_of_a() {
        assert_eq!(TestOneOf::A(123).to_string(), "123");
    }
    #[test]
    fn to_string_one_of_b() {
        assert_eq!(TestOneOf::B("hello".to_string()).to_string(), "hello");
    }

    #[test]
    fn deserialize_one_of_a() {
        assert_eq!(
            serde_json::from_str::<TestOneOf>("123").unwrap(),
            TestOneOf::A(123)
        );
    }
    #[test]
    fn deserialize_one_of_b() {
        assert_eq!(
            serde_json::from_str::<TestOneOf>("\"hello\"").unwrap(),
            TestOneOf::B("hello".to_string())
        );
    }
    #[test]
    fn deserialize_one_of_fails_multiple_matches() {
        // Fails because 123 can be parsed as either u32 or u16.
        assert!(serde_json::from_str::<OneOf2<u32, u16>>("123").is_err());
    }
    #[test]
    fn deserialize_one_of_fails_no_match() {
        // Fails because can't parse an object.
        assert!(serde_json::from_str::<TestOneOf>("{}").is_err());
    }

    #[test]
    fn from_str_one_of_fails_multiple_matches() {
        // Fails because 123 can be parsed as either u32 or String.
        assert!("123".parse::<TestOneOf>().is_err());
    }
    #[test]
    fn from_str_one_of_b() {
        assert_eq!(
            "hello".parse::<TestOneOf>().unwrap(),
            TestOneOf::B("hello".to_string())
        );
    }
    #[test]
    fn from_str_one_of_a() {
        // Swap the order of values so that we can check an A match works.
        assert_eq!(
            "hello".parse::<OneOf2<String, u32>>().unwrap(),
            OneOf2::<String, u32>::A("hello".to_string())
        );
    }
    #[test]
    fn from_str_one_of_fails_no_match() {
        // Fails because can't parse a string as a number.
        assert!("hello".parse::<OneOf2<u32, u16>>().is_err());
    }
}

// Define a macro to define the `AnyOf` enum for a specific number of inner types.
macro_rules! any_of {
    (
        $t:ident,
        $($i:ident),*
    ) => {
        common_one_any_of!($t, $($i),*);

        impl<'b, $($i),*> Deserialize<'b> for $t<$($i),*> where
            $($i: PartialEq + for<'a> Deserialize<'a>,)*
        {
            fn deserialize<De: Deserializer<'b>>(deserializer: De) -> Result<Self, De::Error> {
                let content = Content::deserialize(deserializer)?;
                $(
                    if let Ok(inner) = $i::deserialize(ContentRefDeserializer::<De::Error>::new(&content)) {
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
any_of!(AnyOf12, A, B, C, D, E, F, G, H, I, J, K);
any_of!(AnyOf13, A, B, C, D, E, F, G, H, I, J, K, L);
any_of!(AnyOf14, A, B, C, D, E, F, G, H, I, J, K, L, M);
any_of!(AnyOf15, A, B, C, D, E, F, G, H, I, J, K, L, M, N);
any_of!(AnyOf16, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);

#[cfg(test)]
mod any_of_tests {
    use super::AnyOf2;
    type TestAnyOf = AnyOf2<u32, String>;

    #[test]
    fn serialize_any_of_a() {
        assert_eq!(serde_json::to_string(&TestAnyOf::A(123)).unwrap(), "123");
    }
    #[test]
    fn serialize_any_of_b() {
        assert_eq!(
            serde_json::to_string(&TestAnyOf::B("hello".to_string())).unwrap(),
            "\"hello\""
        );
    }

    #[test]
    fn to_string_any_of_a() {
        assert_eq!(TestAnyOf::A(123).to_string(), "123");
    }
    #[test]
    fn to_string_any_of_b() {
        assert_eq!(TestAnyOf::B("hello".to_string()).to_string(), "hello");
    }

    #[test]
    fn deserialize_any_of_a() {
        assert_eq!(
            serde_json::from_str::<TestAnyOf>("123").unwrap(),
            TestAnyOf::A(123)
        );
    }
    #[test]
    fn deserialize_any_of_b() {
        assert_eq!(
            serde_json::from_str::<TestAnyOf>("\"hello\"").unwrap(),
            TestAnyOf::B("hello".to_string())
        );
    }
    #[test]
    fn deserialize_any_of_multiple_matches() {
        // 123 can be parsed as either u32 or u16, so we parse it as the first.
        assert_eq!(
            serde_json::from_str::<AnyOf2<u32, u16>>("123").unwrap(),
            AnyOf2::<u32, u16>::A(123)
        );
    }
    #[test]
    fn deserialize_any_of_fails_no_match() {
        // Fails because can't parse an object.
        assert!(serde_json::from_str::<TestAnyOf>("{}").is_err());
    }

    #[test]
    fn from_str_any_of_a() {
        assert_eq!("123".parse::<TestAnyOf>().unwrap(), TestAnyOf::A(123));
    }
    #[test]
    fn from_str_any_of_b() {
        assert_eq!(
            "hello".parse::<TestAnyOf>().unwrap(),
            TestAnyOf::B("hello".to_string())
        );
    }
    #[test]
    fn from_str_any_of_fails_no_match() {
        // Fails because can't parse a string as a number.
        assert!("hello".parse::<AnyOf2<u32, u16>>().is_err());
    }
}
