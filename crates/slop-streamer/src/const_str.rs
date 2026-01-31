macro_rules! const_str {
    ($vis:vis struct $ident:ident($val:literal)) => {
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $vis struct $ident;

        impl std::fmt::Display for $ident {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Self::VALUE.fmt(formatter)
            }
        }

        impl $ident {
            $vis const VALUE: &str = $val;
        }

        impl AsRef<str> for $ident {
            fn as_ref(&self) -> &str {
                Self::VALUE
            }
        }

        impl std::ops::Deref for $ident {
            type Target = str;

            fn deref(&self) -> &str {
                Self::VALUE
            }
        }

        impl<'de> serde::Deserialize<'de> for $ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::de::Deserializer<'de>
            {
                let val = <&'de str as serde::Deserialize>::deserialize(deserializer)?;

                if val == Self::VALUE {
                    Ok(Self)
                } else {
                    Err(serde::de::Error::invalid_value(serde::de::Unexpected::Str(val), &Self::VALUE))
                }
            }
        }

        impl serde::Serialize for $ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::ser::Serializer
            {
                Self::VALUE.serialize(serializer)
            }
        }
    };
}
