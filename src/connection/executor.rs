use crate::error::RXQLiteError;
use crate::type_info::DataType;
use crate::RXQLiteColumn;
use crate::{
    RXQLite, RXQLiteConnection, RXQLiteQueryResult, RXQLiteRow, RXQLiteStatement, RXQLiteTypeInfo,
    RXQLiteValue,
};
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
//use futures_util::TryStreamExt;
use sqlx_core::describe::Describe;
use sqlx_core::error::Error;
use sqlx_core::executor::{Execute, Executor};
use sqlx_core::ext::ustr::UStr;
use sqlx_core::try_stream;
use sqlx_core::Either;
use rxqlite_common::Column;

trait IntoRXQLitTypeInfo {
  fn into_rxqlite_type_info(self)->RXQLiteTypeInfo;
}

impl IntoRXQLitTypeInfo for rxqlite_common::TypeInfo {
  fn into_rxqlite_type_info(self)->RXQLiteTypeInfo {
    match self {
      Self::Null => RXQLiteTypeInfo(DataType::Null),
      Self::Int => RXQLiteTypeInfo(DataType::Int),
      Self::Float => RXQLiteTypeInfo(DataType::Float),
      Self::Text => RXQLiteTypeInfo(DataType::Text),
      Self::Blob => RXQLiteTypeInfo(DataType::Blob),
      Self::Numeric => RXQLiteTypeInfo(DataType::Numeric),
      Self::Bool => RXQLiteTypeInfo(DataType::Bool),
      Self::Int64 => RXQLiteTypeInfo(DataType::Int64),
      Self::Date => RXQLiteTypeInfo(DataType::Date),
      Self::Time => RXQLiteTypeInfo(DataType::Time),
      Self::DateTime => RXQLiteTypeInfo(DataType::Datetime),
    }
  }
}

impl IntoRXQLitTypeInfo for &rxqlite_common::TypeInfo {
  fn into_rxqlite_type_info(self)->RXQLiteTypeInfo {
    match self {
      rxqlite_common::TypeInfo::Null => RXQLiteTypeInfo(DataType::Null),
      rxqlite_common::TypeInfo::Int => RXQLiteTypeInfo(DataType::Int),
      rxqlite_common::TypeInfo::Float => RXQLiteTypeInfo(DataType::Float),
      rxqlite_common::TypeInfo::Text => RXQLiteTypeInfo(DataType::Text),
      rxqlite_common::TypeInfo::Blob => RXQLiteTypeInfo(DataType::Blob),
      rxqlite_common::TypeInfo::Numeric => RXQLiteTypeInfo(DataType::Numeric),
      rxqlite_common::TypeInfo::Bool => RXQLiteTypeInfo(DataType::Bool),
      rxqlite_common::TypeInfo::Int64 => RXQLiteTypeInfo(DataType::Int64),
      rxqlite_common::TypeInfo::Date => RXQLiteTypeInfo(DataType::Date),
      rxqlite_common::TypeInfo::Time => RXQLiteTypeInfo(DataType::Time),
      rxqlite_common::TypeInfo::DateTime => RXQLiteTypeInfo(DataType::Datetime),
    }
  }
}

impl<'c> Executor<'c> for &'c mut RXQLiteConnection {
    type Database = RXQLite;

    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
        mut query: E,
    ) -> BoxStream<'e, Result<Either<RXQLiteQueryResult, RXQLiteRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let sql = query.sql();
        let arguments = query.take_arguments();
        //let persistent = query.persistent() && arguments.is_some();

        //let args = Vec::with_capacity(arguments.len());

        Box::pin(try_stream! {
          let result_or_rows = self.inner.fetch_many(sql, match arguments {
            Some(arguments)=>arguments.values,
            _=>vec![],
          }).await;
          match result_or_rows {
            Ok(result_or_rows)=> {
              //println!("{}:({})",file!(),line!());

              //pin_mut!(cursor);
              let mut result_or_rows_iter=result_or_rows.into_iter();
              while let Some(result_or_row) = result_or_rows_iter.next() {
                match result_or_row {
                  Ok(result_or_row) => {
                    match result_or_row {
                      Either::Left(res)=> {
                        let res=Either::Left(RXQLiteQueryResult {
                          last_insert_rowid: res.last_insert_rowid,
                          changes: res.changes,
                        });
                        r#yield!(res);
                      }
                      Either::Right(row)=> {
                        let size = row.inner.len();
                        let mut values = Vec::with_capacity(size);
                        let mut columns = Vec::with_capacity(size);
                        let mut column_names: sqlx_core::HashMap<UStr,usize> = Default::default();
                        for (_i,col) in row.inner.into_iter().enumerate() {
                          let ordinal = col.ordinal;
                          let column_name= UStr::from(row.columns[ordinal as usize].name.to_string());
                          
                          let column: &Column = &row.columns[ordinal as usize];
                          let rxqlite_type_info = (&column.type_info).into_rxqlite_type_info();
                          values.push(RXQLiteValue::new(col.value,rxqlite_type_info.clone()));
                          columns.push(RXQLiteColumn{
                            name : column_name.clone(),
                            ordinal: ordinal as _,
                            type_info: rxqlite_type_info,
                          });
                          column_names.insert(column_name,ordinal as _);
                        }
                        let row=Either::Right(RXQLiteRow {
                          values: values.into_boxed_slice(),
                          columns: columns.into(),
                          column_names: column_names.into(),
                        });
                        r#yield!(row);
                      }
                    }
                  }
                  Err(err)=>{
                    return Err(RXQLiteError{
                      inner: anyhow::anyhow!(err),
                    }.into());
                  }
                }
              }
              Ok(())
            }
            Err(err)=> {
              Err(RXQLiteError{
                inner: err,
              }.into())
            }
          }
        })
    }

    fn fetch_optional<'e, 'q: 'e, E: 'q>(
        self,
        mut query: E,
    ) -> BoxFuture<'e, Result<Option<RXQLiteRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let sql = query.sql();
        let arguments = query.take_arguments();
        //let persistent = query.persistent() && arguments.is_some();

        //let args = Vec::with_capacity(arguments.len());

        Box::pin(async {
            let row = self
                .inner
                .fetch_optional(
                    sql,
                    match arguments {
                        Some(arguments) => arguments.values,
                        _ => vec![],
                    },
                )
                .await;
            match row {
                Ok(row) => {
                    //println!("{}:({})",file!(),line!());

                    //pin_mut!(cursor);

                    if let Some(row) = row {
                        let size = row.inner.len();
                        let mut values = Vec::with_capacity(size);
                        let mut columns = Vec::with_capacity(size);
                        let mut column_names: sqlx_core::HashMap<UStr,usize> = Default::default();
                        for (_i, col) in row.inner.into_iter().enumerate() {
                          let ordinal = col.ordinal;
                          let column_name= UStr::from(row.columns[ordinal as usize].name.to_string());
                          let column: &Column = &row.columns[ordinal as usize];
                          let rxqlite_type_info = (&column.type_info).into_rxqlite_type_info();
                          values.push(RXQLiteValue::new(col.value,rxqlite_type_info.clone()));
                          columns.push(RXQLiteColumn{
                            name : column_name.clone(),
                            ordinal: ordinal as _,
                            type_info: rxqlite_type_info,
                          });
                          column_names.insert(column_name,ordinal as _);
                          
                          
                        }
                        let row = RXQLiteRow {
                            values: values.into_boxed_slice(),
                            columns: columns.into(),
                            column_names: column_names.into(),
                        };
                        Ok(Some(row))
                    } else {
                        Ok(None)
                    }
                }
                Err(err) => Err(RXQLiteError { inner: err }.into()),
            }
        })
    }

    fn fetch_one<'e, 'q: 'e, E: 'q>(self, mut query: E) -> BoxFuture<'e, Result<RXQLiteRow, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let sql = query.sql();
        let arguments = query.take_arguments();
        //let persistent = query.persistent() && arguments.is_some();

        //let args = Vec::with_capacity(arguments.len());

        Box::pin(async {
            let row = self
                .inner
                .fetch_one(
                    sql,
                    match arguments {
                        Some(arguments) => arguments.values,
                        _ => vec![],
                    },
                )
                .await;
            match row {
                Ok(row) => {
                    let size = row.inner.len();
                    let mut values = Vec::with_capacity(size);
                    let mut columns = Vec::with_capacity(size);
                    let mut column_names: sqlx_core::HashMap<UStr,usize> = Default::default();
                    for (_i, col) in row.inner.into_iter().enumerate() {
                        let ordinal = col.ordinal;
                        let column_name= UStr::from(row.columns[ordinal as usize].name.to_string());
                          let column: &Column = &row.columns[ordinal as usize];
                          let rxqlite_type_info = (&column.type_info).into_rxqlite_type_info();
                          values.push(RXQLiteValue::new(col.value,rxqlite_type_info.clone()));
                          columns.push(RXQLiteColumn{
                            name : column_name.clone(),
                            ordinal: ordinal as _,
                            type_info: rxqlite_type_info,
                          });
                          column_names.insert(column_name,ordinal as _);
                    }
                    let row = RXQLiteRow {
                        values: values.into_boxed_slice(),
                        columns: columns.into(),
                        column_names: column_names.into(),
                    };
                    Ok(row)
                }
                Err(err) => Err(RXQLiteError { inner: err }.into()),
            }
        })
    }

    fn prepare_with<'e, 'q: 'e>(
        self,
        _sql: &'q str,
        _parameters: &[RXQLiteTypeInfo],
    ) -> BoxFuture<'e, Result<RXQLiteStatement<'q>, Error>>
    where
        'c: 'e,
    {
        Box::pin(async {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "prepare_with not supported",
            )))
        })
    }
    #[doc(hidden)]
    fn describe<'e, 'q: 'e>(self, _sql: &'q str) -> BoxFuture<'e, Result<Describe<RXQLite>, Error>>
    where
        'c: 'e,
    {
        Box::pin(async {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "describe not supported",
            )))
        })
    }
}
