use std::{fmt, str::FromStr};

use serde::{
    Deserializer, Serializer,
    de::{Error, Visitor},
};
use solana_pubkey::Pubkey;

pub fn serialize<S: Serializer>(v: &Pubkey, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&v.to_string())
}

pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Pubkey, D::Error> {
    // need visitor to handle both &str and String
    struct V;

    impl Visitor<'_> for V {
        type Value = Pubkey;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("base58 encoded pubkey")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Pubkey::from_str(v).map_err(Error::custom)
        }
    }

    d.deserialize_str(V)
}
