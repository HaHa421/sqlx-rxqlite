use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::type_info::DataType;
use crate::types::Type;
use crate::{RXQLite, /*RXQLiteArgumentValue,*/ RXQLiteTypeInfo, RXQLiteValueRef};

impl Type<RXQLite> for bool {
    fn type_info() -> RXQLiteTypeInfo {
        RXQLiteTypeInfo(DataType::Bool)
    }

    fn compatible(ty: &RXQLiteTypeInfo) -> bool {
        matches!(ty.0, DataType::Bool | DataType::Int | DataType::Int64)
    }
}

impl<'q> Encode<'q, RXQLite> for bool {
    fn encode_by_ref(&self, args: &mut Vec<rxqlite_common::Value>) -> IsNull {
        args.push(rxqlite_common::Value::Int((*self).into()));

        IsNull::No
    }
}

impl<'r> Decode<'r, RXQLite> for bool {
    fn decode(value: RXQLiteValueRef<'r>) -> Result<bool, BoxDynError> {
        Ok(value.int()? != 0)
    }
}
