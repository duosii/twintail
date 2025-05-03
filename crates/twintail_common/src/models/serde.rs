use core::fmt;

use serde::{
    Deserialize, Serialize,
    de::{self, Visitor},
};

/// Wrapper that converts f64 floats into an f32 when deserializing.
#[derive(Serialize, Clone)]
pub struct F32Wrapper(f32);

// Custom deserializer that converts f64 to f32
impl<'de> Deserialize<'de> for F32Wrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct F32Visitor;

        impl Visitor<'_> for F32Visitor {
            type Value = F32Wrapper;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a float")
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(F32Wrapper(value as f32))
            }
        }

        deserializer.deserialize_f64(F32Visitor)
    }
}

/// Custom intermediate type for JSON data.
///
/// Acts very similarly to [`serde_json::Value`], but all floats are parsed as f32 instead of f64.
///
/// This is important for suitemaster files as the game client is not able to parse f64 values.
#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ValueF32 {
    Null,
    Bool(bool),
    Integer(i64),
    UInteger(u64),
    Float(F32Wrapper),
    String(String),
    Array(Vec<ValueF32>),
    Object(std::collections::HashMap<String, ValueF32>),
}
