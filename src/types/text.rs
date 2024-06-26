use crate::{RXQLite, /*RXQLiteArgumentValue, */ RXQLiteTypeInfo, RXQLiteValueRef};
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::{Text, Type};
use std::fmt::Display;
use std::str::FromStr;

impl<T> Type<RXQLite> for Text<T> {
    fn type_info() -> RXQLiteTypeInfo {
        <String as Type<RXQLite>>::type_info()
    }

    fn compatible(ty: &RXQLiteTypeInfo) -> bool {
        <String as Type<RXQLite>>::compatible(ty)
    }
}

impl<'q, T> Encode<'q, RXQLite> for Text<T>
where
    T: Display,
{
    fn encode_by_ref(&self, buf: &mut Vec<rxqlite_common::Value>) -> IsNull {
        Encode::<RXQLite>::encode(self.0.to_string(), buf)
    }
}

impl<'r, T> Decode<'r, RXQLite> for Text<T>
where
    T: FromStr,
    BoxDynError: From<<T as FromStr>::Err>,
{
    fn decode(value: RXQLiteValueRef<'r>) -> Result<Self, BoxDynError> {
        //Ha better to decode &str
        let s: String = Decode::<RXQLite>::decode(value)?;
        Ok(Self(s.as_str().parse()?))
    }
}
