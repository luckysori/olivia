#[doc(hidden)]
#[macro_export]
macro_rules! impl_fromstr_deserailize {
    (
        name => $name:literal,
        fn from_bytes$(<$($tpl:ident  $(: $tcl:ident)?),*>)?($input:ident : [u8;$len:literal]) ->  Option<$type:path> $block:block
    ) => {

        impl$(<$($tpl $(:$tcl)?),*>)? core::str::FromStr for $type  {
            type Err = $crate::HexError;

            /// Parses the string as hex and interprets tries to convert the
            /// resulting byte array into the desired value.
            fn from_str(hex: &str) -> Result<$type , $crate::HexError> {
                use $crate::hex_val;
                if hex.len() % 2 == 1 {
                    Err($crate::HexError::InvalidHex)
                } else if $len * 2 != hex.len() {
                    Err($crate::HexError::InvalidLength)
                } else {
                    let mut buf = [0u8; $len];

                    for (i, hex_byte) in hex.as_bytes().chunks(2).enumerate() {
                        buf[i] = hex_val(hex_byte[0])? << 4 | hex_val(hex_byte[1])?
                    }

                    let $input = buf;
                    let result = $block;
                    result.ok_or($crate::HexError::InvalidEncoding)
                }
            }
        }

        impl<'de, $($($tpl $(: $tcl)?),*)?> serde::Deserialize<'de> for $type  {
            fn deserialize<Deser: serde::Deserializer<'de>>(
                deserializer: Deser,
            ) -> Result<$type , Deser::Error> {
                {
                    if deserializer.is_human_readable() {
                        #[allow(unused_parens)]
                        struct HexVisitor$(<$($tpl),*>)?$((core::marker::PhantomData<($($tpl),*)> ))?;
                        impl<'de, $($($tpl $(: $tcl)?),*)?> serde::de::Visitor<'de> for HexVisitor$(<$($tpl),*>)? {
                            type Value = $type ;
                            fn expecting(
                                &self,
                                f: &mut core::fmt::Formatter,
                            ) -> core::fmt::Result {
                                write!(f, "a {}-byte hex encoded {}", $len, $name)?;
                                Ok(())
                            }

                            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<$type , E> {
                                use $crate::HexError::*;
                                <$type  as core::str::FromStr>::from_str(v).map_err(|e| match e {
                                    InvalidLength => E::invalid_length(v.len(), &format!("{}", $len).as_str()),
                                    InvalidEncoding => E::invalid_value(serde::de::Unexpected::Str(v), &self),
                                    InvalidHex => E::custom("invalid hex")
                                })
                            }
                        }

                        #[allow(unused_parens)]
                        return deserializer.deserialize_str(HexVisitor$((core::marker::PhantomData::<($($tpl),*)>))?);
                    }
                }

                {
                    #[allow(unused_parens)]
                    struct BytesVisitor$(<$($tpl),*>)?$((core::marker::PhantomData<($($tpl),*)> ))?;

                    impl<'de, $($($tpl $(: $tcl)?),*)?> serde::de::Visitor<'de> for BytesVisitor$(<$($tpl),*>)? {
                        type Value = $type ;

                        fn expecting(
                            &self,
                            f: &mut core::fmt::Formatter,
                        ) -> core::fmt::Result {
                            write!(f, "a valid {}-byte encoding of a {}", $len, $name)?;
                            Ok(())
                        }

                        fn visit_seq<A>(self, mut seq: A) -> Result<$type , A::Error>
                        where A: serde::de::SeqAccess<'de> {

                            let mut $input = [0u8; $len];
                            for i in 0..$len {
                                $input[i] = seq.next_element()?
                                    .ok_or_else(|| serde::de::Error::custom(format_args!("invalid length {}, expected {}", i, &self as &dyn serde::de::Expected)))?;
                            }

                            let result = $block;
                            result.ok_or(serde::de::Error::custom(format_args!("invalid byte encoding, expected {}", &self as &dyn serde::de::Expected)))
                        }
                    }

                    #[allow(unused_parens)]
                    deserializer.deserialize_tuple($len, BytesVisitor$((core::marker::PhantomData::<($($tpl),*)>))?)
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_fromsql {
    (
        name => $name:literal,
        fn from_bytes$(<$($tpl:ident  $(: $tcl:ident)?),*>)?($input:ident : [u8;$len:literal]) ->  Option<$type:path> $block:block
    ) => {
        impl<DB: diesel::backend::Backend>
            diesel::deserialize::FromSql<diesel::sql_types::Binary, DB> for $type
        where
            Vec<u8>: diesel::deserialize::FromSql<diesel::sql_types::Binary, DB>,
        {
            fn from_sql(bytes: Option<&DB::RawValue>) -> diesel::deserialize::Result<Self> {
                let vec = <Vec<u8> as diesel::deserialize::FromSql<
                    diesel::sql_types::Binary,
                    DB,
                >>::from_sql(bytes)?;
                if vec.len() != $len {
                    return Err(format!(
                        "wrong length for {}, expected {} got {}",
                        $name,
                        $len,
                        vec.len()
                    ))?;
                }
                let mut $input = [0u8; $len];
                $input.copy_from_slice(&vec[..]);
                let result = $block;
                Ok(result.ok_or(format!(
                    "Invalid {} from database '{}'",
                    $name,
                    $crate::util::to_hex(&$input[..])
                ))?)
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_tosql {
    (fn to_bytes$(<$($tpl:ident  $(: $tcl:ident)?),*>)?($self:ident : &$type:path) -> $(&)?[u8;$len:literal] $block:block) => {
        impl<DB: diesel::backend::Backend> diesel::serialize::ToSql<sql_types::Binary, DB>
            for $type
        {
            fn to_sql<W: std::io::Write>(
                &self,
                out: &mut diesel::serialize::Output<W, DB>,
            ) -> diesel::serialize::Result {
                let $self = self;
                let bytes = $block;
                diesel::serialize::ToSql::<diesel::sql_types::Binary, DB>::to_sql(
                    bytes.as_ref(),
                    out,
                )
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_serialize {
    (fn to_bytes$(<$($tpl:ident  $(: $tcl:ident)?),*>)?($self:ident : &$type:path) -> $(&)?[u8;$len:literal] $block:block) => {
        impl$(<$($tpl $(:$tcl)?),*>)? serde::Serialize for $type {
            fn serialize<Ser: serde::Serializer>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error> {
                 {
                    if serializer.is_human_readable() {
                        return serializer.collect_str(&self)
                    }
                }
                //NOTE: idea taken from https://github.com/dalek-cryptography/curve25519-dalek/pull/297/files
                use serde::ser::SerializeTuple;
                let $self = &self;
                let bytes = $block;
                let mut tup = serializer.serialize_tuple($len)?;
                for byte in bytes.iter() {
                    tup.serialize_element(byte)?;
                }
                tup.end()
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_display_debug {
    (fn to_bytes$(<$($tpl:ident  $(: $tcl:ident)?),*>)?($self:ident : &$type_name:ident$(<$($tpr:path),+>)?) -> $($tail:tt)*) => {
        impl$(<$($tpl $(:$tcl)?),*>)? core::fmt::Display for $type_name$(<$($tpr),+>)? {
            /// Displays as hex.
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                let $self = &self;
                $crate::impl_display_debug!(@output f, $self, $($tail)*);
                Ok(())
            }
        }

        impl$(<$($tpl $(:$tcl)?),*>)? core::fmt::Debug for $type_name$(<$($tpr),+>)? {
            /// Formats the type as hex and any markers on the type.
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                let $self = &self;
                write!(f, "{}", stringify!($type_name))?;
                $(
                    write!(f, "<")?;
                    $crate::impl_display_debug!(@recursive_print f, $(core::any::type_name::<$tpr>().rsplit("::").next().unwrap()),*);
                    write!(f, ">")?;
                )?
                    write!(f, "(")?;
                $crate::impl_display_debug!(@output f, $self, $($tail)*);
                write!(f, ")")?;
                Ok(())
            }
        }
    };
    (@output $f:ident, $self:ident, Result<$(&)?[u8;$len:literal], &str> $block:block) => {
        let res: Result<[u8;$len], &str> = $block;
        match res {
            Ok(bytes) => {
                for byte in bytes.iter() {
                    write!($f, "{:02x}", byte)?
                }
            },
            Err(string) => {
                write!($f, "{}", string)?
            }
        }
    };
    (@output $f:ident, $self:ident, $(&)?[u8;$len:literal] $block:block) => {
        let bytes = $block;
        for byte in bytes.iter() {
            write!($f, "{:02x}", byte)?
        }
    };
    (@recursive_print $f:ident, $next:expr, $($tt:tt)+) => {
        $f.write_str($next)?;
        $f.write_str(",")?;
        $crate::impl_display_debug!(@recursive_print $f, $($tt)+)
    };
    (@recursive_print $f:ident, $next:expr) => {
        $f.write_str($next)?;
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_display_debug_serialize {
    ($($tt:tt)+) => {
        $crate::impl_serialize!($($tt)+);
        $crate::impl_display_debug!($($tt)+);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_display_debug_serialize_tosql {
    ($($tt:tt)+) => {
        $crate::impl_display_debug_serialize!($($tt)+);
        $crate::impl_tosql!($($tt)+);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_fromstr_deserailize_fromsql {
    ($($tt:tt)+) => {
        $crate::impl_fromstr_deserailize!($($tt)+);
        $crate::impl_fromsql!($($tt)+);
    };
}
