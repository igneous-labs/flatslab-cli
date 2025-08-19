use std::fmt;

use serde::{
    Deserializer, Serializer,
    de::{Error, Visitor},
};

pub fn serialize<S: Serializer>(v: &i32, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_i32(*v)
}

pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<i32, D::Error> {
    struct V;

    impl Visitor<'_> for V {
        type Value = i32;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("Either raw i32 nanos or `{decimal}%` e.g. 0.1%")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            // allow underscores for human-readable formatting
            let v = v.replace("_", "");

            if let Ok(v_i64) = v.parse() {
                return self.visit_i64(v_i64);
            }

            if !v.ends_with("%") {
                return Err(Error::custom("string did not end with %"));
            }

            let pct: f64 = v.split_at(v.len() - 1).0.parse().map_err(Error::custom)?;

            let nanos = pct * 10_000_000.0;

            (nanos.round() as i64).try_into().map_err(Error::custom)
        }

        fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v)
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            v.try_into().map_err(Error::custom)
        }
    }

    d.deserialize_str(V)
}
