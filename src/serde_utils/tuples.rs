// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

/// Implement `Display` for wrapper tuples.
///
/// # Example
///
/// `pub struct MyWrapper(pub u64);`
/// `impl_wrapper_display!(MyWrapper);`
#[macro_export]
macro_rules! impl_wrapper_display {
    ($typ:ty) => {
        impl std::fmt::Display for $typ {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                std::fmt::Display::fmt(&self.0, f)
            }
        }
    };
}

/// Implement `FromStr` for wrapper tuples.
///
/// # Example
///
/// `pub struct MyWrapper(pub u64);`
/// `impl_wrapper_from_str!(MyWrapper, u64);`
#[macro_export]
macro_rules! impl_wrapper_from_str {
    ($typ:ty, $inner:ty) => {
        impl std::str::FromStr for $typ {
            type Err = <$inner as std::str::FromStr>::Err;
            fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                Ok(Self(std::str::FromStr::from_str(s)?))
            }
        }
    };
}

/// Implement `Display` and `FromStr` for wrapper tuples.
///
/// # Example
///
/// `pub struct MyWrapper(pub u64);`
/// `impl_wrapper_display_from_str!(MyWrapper, u64);`
#[macro_export]
macro_rules! impl_wrapper_display_from_str {
    ($typ:ty, $inner:ty) => {
        impl_wrapper_display!($typ);
        impl_wrapper_from_str!($typ, $inner);
    };
}

/// Implement wrapper around a singed integer type like i32 or i64.
///
/// # Example
///
/// `pub struct MyWrapper(pub i64);`
/// `impl_wrapper_int!(MyWrapper, i64, u64);
#[macro_export]
macro_rules! impl_wrapper_int {
    ($typ:ty, $si:ty, $ui:ty) => {
        impl From<$si> for $typ {
            fn from(n: $si) -> Self {
                Self(n)
            }
        }

        impl Into<$si> for $typ {
            fn into(self) -> $si {
                self.0
            }
        }

        impl TryFrom<$ui> for $typ {
            type Error = &'static str;
            fn try_from(n: $ui) -> std::result::Result<Self, Self::Error> {
                Ok(Self(n.try_into().map_err(|_| "magnitude")?))
            }
        }

        impl TryInto<$ui> for $typ {
            type Error = &'static str;
            fn try_into(self) -> std::result::Result<$ui, Self::Error> {
                self.0.try_into().map_err(|_| "negative")
            }
        }
    };
}

/// Implement wrapper around non-zero type like NonZeroU8, NonZeroU32, etc.
///
/// # Example
///
/// `pub struct MyWrapper(pub NonZeroU64);`
/// `impl_wrapper_nz!(MyWrapper, NonZeroU64, u64);
#[macro_export]
macro_rules! impl_wrapper_nz {
    ($typ:ty, $nz:ty, $zb:ty) => {
        impl From<$nz> for $typ {
            fn from(t: $nz) -> Self {
                Self(t)
            }
        }

        impl TryFrom<$zb> for $typ {
            type Error = &'static str;
            fn try_from(t: $zb) -> std::result::Result<Self, Self::Error> {
                <$nz>::new(t).map(|nz| Self::from(nz)).ok_or("0 is invalid")
            }
        }

        impl Into<$nz> for $typ {
            fn into(self) -> $nz {
                self.0
            }
        }

        impl Into<$zb> for $typ {
            fn into(self) -> $zb {
                self.0.get()
            }
        }
    };
}

/// Implement various string methods like `as_str()`, `len()` etc.
/// for string wrapper tuples.
///
/// # Example
///
/// `pub struct MyWrapper(pub String);`
/// `impl_wrapper_str!(MyWrapper);
#[macro_export]
macro_rules! impl_wrapper_str {
    ($typ:ty) => {
        impl $typ {
            /// Returns `as_str()` of the inner string.
            #[allow(unused)]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }

            /// Returns `is_empty()` of the inner string.
            #[allow(unused)]
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            /// Returns `len()` of the inner string.
            #[allow(unused)]
            pub fn len(&self) -> usize {
                self.0.len()
            }
        }

        impl AsRef<str> for $typ {
            /// Returns `as_ref()` of the inner string.
            fn as_ref(&self) -> &str {
                self.0.as_ref()
            }
        }

        impl std::borrow::Borrow<str> for $typ {
            /// Returns `borrow()` of the inner string.
            fn borrow(&self) -> &str {
                self.0.borrow()
            }
        }

        impl std::ops::Deref for $typ {
            type Target = str;
            /// Returns `deref()` of the inner string.
            fn deref(&self) -> &Self::Target {
                &*self.0
            }
        }

        impl std::fmt::Display for $typ {
            /// Returns `fmt()` of the inner string.
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.0, f)
            }
        }

        impl PartialEq<str> for $typ {
            /// Returns `eq()` of the inner string.
            fn eq(&self, other: &str) -> bool {
                self.0.as_str() == other
            }
        }

        impl PartialOrd<str> for $typ {
            /// Returns `partial_cmp()` of the inner string.
            fn partial_cmp(&self, other: &str) -> Option<std::cmp::Ordering> {
                self.0.as_str().partial_cmp(other)
            }
        }
    };
}

/// Serialize `struct Typ(T)` as `T`.
#[macro_export]
macro_rules! serde_transparent_tuple {
    ($typ: ident, $fmt: expr) => {
        impl serde::Serialize for $typ {
            /// Returns `serialize()` of the inner `T`.
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                self.0.serialize(serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $typ {
            /// Returns `deserialize()` of the inner `T`.
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                serde::Deserialize::deserialize(deserializer).map($typ)
            }
        }
    };
}
