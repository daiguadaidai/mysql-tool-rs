use chrono::{Duration, Local, NaiveDateTime, TimeZone};
use std::ops::Add;

pub const NORMAL_FMT: &str = "%Y-%m-%d %H:%M:%S";
#[allow(dead_code)]
pub const NORMAL_ZERO_TIME_FMT: &str = "%Y-%m-%d 00:00:00";
#[allow(dead_code)]
pub const PARTITION_NAME_FMT: &str = "p%Y%m%d";
#[allow(dead_code)]
pub const ZERO_DATETIME: &str = "0000-01-01 00:00:00";

#[allow(dead_code)]
pub fn get_datetime_by_days(days: i64) -> NaiveDateTime {
    let date = NaiveDateTime::parse_from_str(ZERO_DATETIME, NORMAL_FMT).unwrap();

    return date.add(Duration::days(days));
}

#[allow(dead_code)]
pub fn get_datetime_by_timestamp(timestamp: i64) -> NaiveDateTime {
    Local.timestamp_opt(timestamp, 0).unwrap().naive_local()
}

#[allow(dead_code)]
pub fn now_datetime() -> NaiveDateTime {
    return Local::now().naive_local();
}

#[allow(dead_code)]
pub fn now_str(format: &str) -> String {
    return Local::now().format(format).to_string();
}

pub mod opt_datetime_format_normal {
    use crate::utils::time::NORMAL_FMT;
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    #[allow(dead_code)]
    pub fn serialize<S>(date: &Option<NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(d) = date {
            let s = format!("{}", d.format(NORMAL_FMT));
            serializer.serialize_str(&s)
        } else {
            serializer.serialize_none()
        }
    }

    #[allow(dead_code)]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<String>::deserialize(deserializer)?;
        if let Some(s) = opt {
            if s.is_empty() {
                return Ok(None);
            } else {
                return NaiveDateTime::parse_from_str(&s, NORMAL_FMT)
                    .map_err(serde::de::Error::custom)
                    .map(|value| Some(value));
            }
        } else {
            return Ok(None);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::time::{now_datetime, opt_datetime_format_normal};
    use chrono::NaiveDateTime;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug, Clone)]
    struct TestA {
        #[serde(with = "opt_datetime_format_normal")]
        a: Option<NaiveDateTime>,
        #[serde(with = "opt_datetime_format_normal")]
        b: Option<NaiveDateTime>,
        // 字符串转化为结构体没有字段变成None, 需要加上default
        #[serde(default, with = "opt_datetime_format_normal")]
        c: Option<NaiveDateTime>,
    }

    #[test]
    fn test_datetime_format_normal_serialize() {
        let a = TestA {
            a: Some(now_datetime()),
            b: None,
            c: None,
        };

        println!(
            "obj -> json str: {}",
            serde_json::to_string_pretty(&a).unwrap()
        )
    }

    #[test]
    fn test_datetime_format_normal_deserialize() {
        let raw: &str = r#"
{
  "a": "2023-01-31 16:43:07",
  "b": null
}
        "#;

        let a = serde_json::from_str::<TestA>(raw).unwrap();

        println!("json str -> obj: {:?}", &a)
    }
}
