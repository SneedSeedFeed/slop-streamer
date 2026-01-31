#[macro_export]
macro_rules! define_contract {
    ($vis:vis trait $trait:ident $(: $supertrait:ident + $publictrait:ident)? => $wrapper:ident {
        $($field:ident: $ty:ty $(= $default:expr)?),* $(,)?
    }) => {
        define_contract!($vis trait $trait $(: $supertrait + $publictrait)* => $wrapper {
            [$($field: $ty $(= $default)?,)*]
            const []
        });
    };

    ($vis:vis trait $trait:ident $(: $supertrait:ident + $publictrait:ident)? => $wrapper:ident {
        [$($field:ident: $ty:ty $(= $default:expr)?),* $(,)?]
        const [$($const_field:literal: $const_value:literal),* $(,)?]
    } ) => {
        $vis trait $trait {
            $(
                define_contract!(@trait_method $field, $ty $(, $default)?);
            )*

            fn into_wrapped(self) -> $wrapper<Self>
            where
                Self: Sized,
            {
                $wrapper(self)
            }

            fn as_wrapped(&self) -> $wrapper<&Self> {
                $wrapper(self)
            }

            /// Get a reference to this value as its wrapper type.
            /// This is safe because the wrapper is #[repr(transparent)].
            fn as_wrapper_ref(&self) -> &$wrapper<Self>
            where
                Self: Sized,
            {
                // Safety: $wrapper is #[repr(transparent)], so &Self and &$wrapper<Self>
                // have identical layout and can be safely transmuted
                unsafe {&*(std::ptr::from_ref(self) as *const $wrapper<Self>)}
            }
        }


        impl<T: $trait + ?Sized> $trait for &T {
            $(
                define_contract!(@delegate_method $trait, $field, $ty);
            )*
        }

        impl<T: $trait + ?Sized> $trait for &mut T {
            $(
                define_contract!(@delegate_method $trait, $field, $ty);
            )*
        }

        impl<T: $trait + ?Sized> $trait for Box<T> {
            $(
                define_contract!(@delegate_method $trait, $field, $ty);
            )*
        }

        impl<T: $trait + ?Sized> $trait for std::rc::Rc<T> {
            $(
                define_contract!(@delegate_method $trait, $field, $ty);
            )*
        }

        impl<T: $trait + ?Sized> $trait for std::sync::Arc<T> {
            $(
                define_contract!(@delegate_method $trait, $field, $ty);
            )*
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(transparent)]
        $vis struct $wrapper<T>(pub T);

        impl<T> $wrapper<T> {
            pub fn new(inner: T) -> Self {
                Self(inner)
            }

            pub fn into_inner(self) -> T {
                self.0
            }

            pub fn as_inner(&self) -> &T {
                &self.0
            }
        }

        impl<T> AsRef<T> for $wrapper<T> {
            fn as_ref(&self) -> &T {
                &self.0
            }
        }

        impl<T> std::ops::Deref for $wrapper<T> {
            type Target = T;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<T> serde::Serialize for $wrapper<T>
            where T: $trait
        {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut map = serializer.serialize_map(None)?;
                $(
                    serde::ser::SerializeMap::serialize_entry(&mut map, $const_field, $const_value)?;
                )*
                $(
                    define_contract!(@serialize_field map, self.0.$field(), $field $(, $default)?);
                )*
                serde::ser::SerializeMap::end(map)
            }
        }


        impl<T> private::Sealed for $wrapper<T> where T: $trait {}
        $(
            impl<T: $trait> $supertrait for $wrapper<T> {
                fn as_erased(&self) -> &dyn erased_serde::Serialize {
                    self
                }
            }
        )*

        $(
            impl<T: $trait> $publictrait for $wrapper<T> {
                fn erase_variant(&self) -> &dyn $supertrait {
                    self
                }
            }
        )*
    };

    // implementations for pointers and references to T
    (@delegate_method $trait:ident, $field:ident, $ty:ty) => {
        fn $field(&self) -> $ty {
            <T as $trait>::$field(self)
        }
    };

    (@trait_method $field:ident, $ty:ty) => {
        fn $field(&self) -> $ty;
    };

    (@trait_method $field:ident, $ty:ty, $default:expr) => {
        fn $field(&self) -> $ty {
            $default
        }
    };

    // serialize field without default (always serialize)
    (@serialize_field $map:expr, $value:expr, $field:ident) => {
        serde::ser::SerializeMap::serialize_entry(&mut $map, stringify!($field), &$value)?;
    };

    // serialize field with default - check if it's None
    (@serialize_field $map:expr, $value:expr, $field:ident, $default:expr) => {
        define_contract!(@serialize_field_with_default $map, $value, $field, $default);
    };

    // serialize with default = None - skip if None
    (@serialize_field_with_default $map:expr, $value:expr, $field:ident, None) => {
        if let Some(ref val) = $value {
            serde::ser::SerializeMap::serialize_entry(&mut $map, stringify!($field), val)?;
        }
    };

    // serialize with other defaults - always serialize
    (@serialize_field_with_default $map:expr, $value:expr, $field:ident, $default:expr) => {
        serde::ser::SerializeMap::serialize_entry(&mut $map, stringify!($field), &$value)?;
    };
}
