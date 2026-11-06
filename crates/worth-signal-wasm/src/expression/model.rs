use std::collections::BTreeMap;
use std::fmt;

use serde::de::{self, Deserializer, MapAccess, SeqAccess, Unexpected, Visitor};
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Default)]
pub enum SignalValue {
    #[default]
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<SignalValue>),
    Object(Vec<(String, SignalValue)>),
}

impl Serialize for SignalValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Null => serializer.serialize_unit(),
            Self::Bool(value) => serializer.serialize_bool(*value),
            Self::Number(value) => serializer.serialize_f64(*value),
            Self::String(value) => serializer.serialize_str(value),
            Self::Array(items) => items.serialize(serializer),
            Self::Object(fields) => {
                let mut map = serializer.serialize_map(Some(fields.len()))?;
                for (key, value) in fields {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for SignalValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // One self-describing pass. Trying alternatives builds a format error
        // per rejected shape, and the JS boundary materializes every one as a
        // JavaScript `Error` even when a later alternative succeeds.
        deserializer.deserialize_any(SignalValueVisitor)
    }
}

struct SignalValueVisitor;

impl<'de> Visitor<'de> for SignalValueVisitor {
    type Value = SignalValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_unit<E: de::Error>(self) -> Result<SignalValue, E> {
        Ok(SignalValue::Null)
    }

    fn visit_none<E: de::Error>(self) -> Result<SignalValue, E> {
        Ok(SignalValue::Null)
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<SignalValue, E> {
        Ok(SignalValue::Bool(value))
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<SignalValue, E> {
        Ok(SignalValue::Number(value as f64))
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<SignalValue, E> {
        Ok(SignalValue::Number(value as f64))
    }

    fn visit_f64<E: de::Error>(self, value: f64) -> Result<SignalValue, E> {
        Ok(SignalValue::Number(value))
    }

    fn visit_char<E: de::Error>(self, value: char) -> Result<SignalValue, E> {
        Ok(SignalValue::String(value.to_string()))
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<SignalValue, E> {
        Ok(SignalValue::String(value.to_owned()))
    }

    fn visit_string<E: de::Error>(self, value: String) -> Result<SignalValue, E> {
        Ok(SignalValue::String(value))
    }

    fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<SignalValue, E> {
        std::str::from_utf8(value)
            .map(|text| SignalValue::String(text.to_owned()))
            .map_err(|_| E::invalid_value(Unexpected::Bytes(value), &self))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<SignalValue, A::Error> {
        let mut items = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Some(item) = seq.next_element()? {
            items.push(item);
        }
        Ok(SignalValue::Array(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<SignalValue, A::Error> {
        let Some(first_key) = map.next_key::<String>()? else {
            return Ok(SignalValue::Object(Vec::new()));
        };
        if first_key == SERDE_JSON_NUMBER_TOKEN {
            return decimal_number(map.next_value::<String>()?);
        }
        // Canonical key order; a repeated key keeps its last value.
        let mut fields = BTreeMap::new();
        fields.insert(first_key, map.next_value::<SignalValue>()?);
        while let Some((key, value)) = map.next_entry::<String, SignalValue>()? {
            fields.insert(key, value);
        }
        Ok(SignalValue::Object(fields.into_iter().collect()))
    }
}

/// serde_json's `arbitrary_precision` presents every number as a one-entry map
/// under this key, holding the number's decimal text. Feature unification can
/// turn it on for any build that links serde_json, so the decoder reads it as
/// the number it is rather than as an object.
const SERDE_JSON_NUMBER_TOKEN: &str = "$serde_json::private::Number";

fn decimal_number<E: de::Error>(digits: String) -> Result<SignalValue, E> {
    match digits.parse::<f64>() {
        Ok(value) if value.is_finite() => Ok(SignalValue::Number(value)),
        _ => Err(E::invalid_value(
            Unexpected::Str(&digits),
            &"a finite JSON number",
        )),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Expr {
    Value {
        value: SignalValue,
    },
    Read {
        id: String,
    },
    Get {
        target: Box<Expr>,
        field: String,
    },
    At {
        target: Box<Expr>,
        index: Box<Expr>,
    },
    First {
        target: Box<Expr>,
    },
    Last {
        target: Box<Expr>,
    },
    Slice {
        target: Box<Expr>,
        start: Box<Expr>,
        #[serde(default)]
        end: Option<Box<Expr>>,
    },
    Join {
        target: Box<Expr>,
        separator: Box<Expr>,
    },
    Flatten {
        target: Box<Expr>,
    },
    Object {
        fields: Vec<(String, Expr)>,
    },
    Array {
        items: Vec<Expr>,
    },
    Sum {
        args: Vec<Expr>,
    },
    Multiply {
        args: Vec<Expr>,
    },
    Concat {
        args: Vec<Expr>,
    },
    Coalesce {
        args: Vec<Expr>,
    },
    Length {
        target: Box<Expr>,
    },
    Contains {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    MergeObjects {
        args: Vec<Expr>,
    },
    Keys {
        target: Box<Expr>,
    },
    Values {
        target: Box<Expr>,
    },
    HasField {
        target: Box<Expr>,
        field: String,
    },
    Pick {
        target: Box<Expr>,
        fields: Vec<String>,
    },
    Omit {
        target: Box<Expr>,
        fields: Vec<String>,
    },
    Append {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    Abs {
        target: Box<Expr>,
    },
    Min {
        args: Vec<Expr>,
    },
    Max {
        args: Vec<Expr>,
    },
    Sqrt {
        target: Box<Expr>,
    },
    Sin {
        target: Box<Expr>,
    },
    Cos {
        target: Box<Expr>,
    },
    Floor {
        target: Box<Expr>,
    },
    Mod {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Clamp {
        value: Box<Expr>,
        min: Box<Expr>,
        max: Box<Expr>,
    },
    Atan2 {
        y: Box<Expr>,
        x: Box<Expr>,
    },
    Subtract {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Divide {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Eq {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Neq {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Gt {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Gte {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Lt {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Lte {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    And {
        args: Vec<Expr>,
    },
    Or {
        args: Vec<Expr>,
    },
    Not {
        arg: Box<Expr>,
    },
    If {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionSpec {
    pub expr: Expr,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum IdentitySpec {
    Exact,
    Expr { expr: Expr },
}
