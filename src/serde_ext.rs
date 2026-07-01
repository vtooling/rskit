use anyhow::Result;
use bincode;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};

/// Default datetime format used by the [`sdt`]/[`ddt`] serde helpers.
pub const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.3f";

/// Serialize a `DateTime<Local>` as a formatted string (see [`DATETIME_FORMAT`]).
pub fn sdt<S: serde::Serializer>(data: &DateTime<Local>, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&data.format(DATETIME_FORMAT).to_string())
}

/// Deserialize a `DateTime<Local>` from a formatted string (see [`DATETIME_FORMAT`]).
pub fn ddt<'de, D: serde::Deserializer<'de>>(d: D) -> Result<DateTime<Local>, D::Error> {
    let s = String::deserialize(d)?;
    let naive =
        NaiveDateTime::parse_from_str(&s, DATETIME_FORMAT).map_err(serde::de::Error::custom)?;
    Local
        .from_local_datetime(&naive)
        .single()
        .ok_or_else(|| serde::de::Error::custom("ambiguous or invalid local datetime"))
}

/// Serialize a value to a binary (bincode) byte vector.
pub fn to_bin<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    Ok(bincode::serialize(value)?)
}

/// Deserialize a value from a binary (bincode) byte slice.
pub fn from_bin<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T> {
    Ok(bincode::deserialize(bytes)?)
}

/// Serialize a value to a JSON string.
pub fn to_json<T: Serialize>(value: &T) -> Result<String> {
    Ok(serde_json::to_string(value)?)
}

/// Serialize a value to a pretty (indented) JSON string.
pub fn to_json_pretty<T: Serialize>(value: &T) -> Result<String> {
    Ok(serde_json::to_string_pretty(value)?)
}

/// Deserialize a value from a JSON string.
pub fn from_json<T: for<'de> Deserialize<'de>>(s: &str) -> Result<T> {
    Ok(serde_json::from_str(s)?)
}

/// Flatten a serializable value into a sorted, URL-style `k=v&k2=v2` string.
///
/// Maps become `key=value` pairs joined by `&` and sorted by key. Arrays become a
/// comma-joined list of values. Scalars are returned verbatim. Returns `None` for
/// `null`.
pub fn flatten_query<T: Serialize>(value: &T) -> Option<String> {
    let v = serde_json::to_value(value).ok()?;
    value_to_query(&v)
}

fn value_to_query(v: &serde_json::Value) -> Option<String> {
    match v {
        serde_json::Value::Null => None,
        serde_json::Value::Bool(b) => Some(b.to_string()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Array(arr) => Some(
            arr.iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(","),
        ),
        serde_json::Value::Object(map) => {
            let mut pairs: Vec<String> = map
                .iter()
                .map(|(k, val)| format!("{}={}", k, val))
                .collect();
            pairs.sort();
            Some(pairs.join("&"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Aoo {
        name: String,
        age: i32,
        #[serde(serialize_with = "sdt", deserialize_with = "ddt")]
        date: DateTime<Local>,
    }

    fn sample() -> Aoo {
        Aoo {
            name: "ok".into(),
            age: 18,
            date: Local.with_ymd_and_hms(2024, 8, 15, 11, 0, 16).unwrap()
                + chrono::Duration::milliseconds(100),
        }
    }

    #[test]
    fn json_roundtrip() {
        let s = to_json(&sample()).unwrap();
        let back: Aoo = from_json(&s).unwrap();
        assert_eq!(back, sample());
    }

    #[test]
    fn json_pretty_contains_newline() {
        let s = to_json_pretty(&sample()).unwrap();
        assert!(s.contains('\n'));
    }

    #[test]
    fn from_json_invalid() {
        assert!(from_json::<Aoo>("{not json").is_err());
    }

    #[test]
    fn bin_roundtrip() {
        let a = sample();
        let bytes = to_bin(&a).unwrap();
        let back: Aoo = from_bin(&bytes).unwrap();
        assert_eq!(back, a);
    }

    #[test]
    fn bin_garbage_errors() {
        assert!(from_bin::<Aoo>(&[0xff, 0xff]).is_err());
    }

    #[test]
    fn datetime_format_roundtrip() {
        let raw = "2024-08-15 11:00:16.100";
        let s = format!("{{\"name\":\"ok\",\"age\":18,\"date\":\"{}\"}}", raw);
        let aoo: Aoo = from_json(&s).unwrap();
        assert_eq!(aoo.name, "ok");
        assert_eq!(aoo.age, 18);
        // re-serialize and the date should match the canonical format
        let out = to_json(&aoo).unwrap();
        assert!(out.contains(&format!("\"date\":\"{}\"", raw)));
    }

    #[test]
    fn flatten_query_object_sorted() {
        let v = serde_json::json!({"b": 2, "a": 1});
        assert_eq!(flatten_query(&v), Some("a=1&b=2".to_string()));
    }

    #[test]
    fn flatten_query_array() {
        let v = serde_json::json!([1, 2, 3]);
        assert_eq!(flatten_query(&v), Some("1,2,3".to_string()));
    }

    #[test]
    fn flatten_query_null() {
        let v = serde_json::Value::Null;
        assert_eq!(flatten_query(&v), None);
    }

    #[test]
    fn flatten_query_string() {
        let v = serde_json::json!("hi");
        assert_eq!(flatten_query(&v), Some("hi".to_string()));
    }
}
