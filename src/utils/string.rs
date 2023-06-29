use serde::Serialize;

#[allow(dead_code)]
pub fn to_json_str<T>(value: &T) -> String
where
    T: ?Sized + Serialize,
{
    match serde_json::to_string(value) {
        Ok(data) => data,
        Err(e) => e.to_string(),
    }
}

#[allow(dead_code)]
pub fn to_json_str_pretty<T>(value: &T) -> String
where
    T: ?Sized + Serialize,
{
    match serde_json::to_string_pretty(value) {
        Ok(data) => data,
        Err(e) => e.to_string(),
    }
}
