//! Implementations of OpenAPI `oneOf` and `anyOf` types, assuming
use serde::{
    de::Error,
    private::de::{Content, ContentRefDeserializer},
    Deserialize, Deserializer, Serialize, Serializer,
};

// Define a macro to define the common parts between `OneOf` and `AnyOf` enums for a specific
// number of inner types.
macro_rules! common_one_any_of {
    (
        $t:ident,
        $($i:ident),*
    ) => {
        /// $t
        #[derive(Debug, PartialEq)]
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

// Define a macro to define the `AnyOf` enum for a specific number of inner types.
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
                $(
                    if let Ok(inner) = $i::deserialize(ContentRefDeserializer::<De::Error>::new(&content)) {
                        return Ok(Self::$i(inner));
                    }
                )*
                Err(De::Error::custom("data did not match any within anyOf"))
            }
        }
    }
}

// Use the `any_of!` macro to define the `AnyOf` enum for 1-16 inner types.
one_of!(AnyOf1, A);
one_of!(AnyOf2, A, B);
one_of!(AnyOf3, A, B, C);
one_of!(AnyOf4, A, B, C, D);
one_of!(AnyOf5, A, B, C, D, E);
one_of!(AnyOf6, A, B, C, D, E, F);
one_of!(AnyOf7, A, B, C, D, E, F, G);
one_of!(AnyOf8, A, B, C, D, E, F, G, H);
one_of!(AnyOf9, A, B, C, D, E, F, G, H, I);
one_of!(AnyOf10, A, B, C, D, E, F, G, H, I, J);
one_of!(AnyOf12, A, B, C, D, E, F, G, H, I, J, K);
one_of!(AnyOf13, A, B, C, D, E, F, G, H, I, J, K, L);
one_of!(AnyOf14, A, B, C, D, E, F, G, H, I, J, K, L, M);
one_of!(AnyOf15, A, B, C, D, E, F, G, H, I, J, K, L, M, N);
one_of!(AnyOf16, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
