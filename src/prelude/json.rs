use crate::interop::{from_value, to_value};
use crate::interpreter::{RuntimeError, Value, type_name};
use crate::prelude::{err_str_tuple, ok_tuple};
use crate::{members, native};

members! {
    "Json",
    parse => native!(1, move |_env, args| {
        let Value::String(s) = &args[0] else {
            return Err(RuntimeError::NativeTypeError {
                native: "Json.parse",
                expected: "string",
                got: type_name(&args[0]),
            });
        };
        match serde_json::from_str::<serde_json::Value>(s) {
            Ok(j) => Ok(ok_tuple(to_value(&j)?)),
            Err(e) => Ok(err_str_tuple(e.to_string())),
        }
    }),
    stringify => native!(1, move |_env, args| {
        let json: serde_json::Value = from_value(args[0].clone())?;
        serde_json::to_string(&json)
            .map(|s| Value::String(s.into()))
            .map_err(|e| RuntimeError::Serialize(e.to_string()))
    }),
    stringify_pretty => native!(1, move |_env, args| {
        let json: serde_json::Value = from_value(args[0].clone())?;
        serde_json::to_string_pretty(&json)
            .map(|s| Value::String(s.into()))
            .map_err(|e| RuntimeError::Serialize(e.to_string()))
    }),
}
