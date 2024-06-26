use crate::encode::{Encode, IsNull};
//use crate::types::Type;
//use crate::type_info::DataType;
use crate::{RXQLite, RXQLiteTypeInfo};
//use crate::error::BoxDynError;

pub(crate) use sqlx_core::arguments::*;
use sqlx_core::types::Type;
//use sqlx_core::encode::IsNull;

/// Implementation of [`Arguments`] for MySQL.
#[derive(Debug, Default, Clone)]
pub struct RXQLiteArguments {
    pub(crate) values: Vec<rxqlite_common::Value>,
    pub(crate) types: Vec<RXQLiteTypeInfo>,
}
/*
impl<'q> Encode<'q, RXQLite> for i8 {
    fn encode_by_ref(&self, args: &mut Vec<rxqlite_common::Value>) -> IsNull {
        args.push(rxqlite_common::Value::from(*self));

        IsNull::No
    }
}
*/
impl RXQLiteArguments {
    pub(crate) fn add<'q, T>(&mut self, value: T)
    where
        T: Encode<'q, RXQLite> + Type<RXQLite>,
    {
        let ty = T::type_info();

        if let IsNull::Yes = value.encode_by_ref(&mut self.values) {
          self.values.push(rxqlite_common::Value::Null);
        } else {
        }
        self.types.push(ty);
    }

    #[doc(hidden)]
    pub fn len(&self) -> usize {
        self.types.len()
    }
}

impl<'q> Arguments<'q> for RXQLiteArguments {
    type Database = RXQLite;

    fn reserve(&mut self, len: usize, size: usize) {
        self.types.reserve(len);
        self.values.reserve(size);
    }

    fn add<T>(&mut self, value: T)
    where
        T: Encode<'q, Self::Database> + Type<Self::Database>,
    {
        self.add(value)
    }
}
