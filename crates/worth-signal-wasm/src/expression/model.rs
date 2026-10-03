use std::collections::BTreeMap;

use serde::de::Deserializer;
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
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum SignalValueWire {
            Null(()),
            Bool(bool),
            Number(f64),
            String(String),
            Array(Vec<SignalValue>),
            Object(BTreeMap<String, SignalValue>),
        }

        let wire = SignalValueWire::deserialize(deserializer)?;
        Ok(match wire {
            SignalValueWire::Null(()) => SignalValue::Null,
            SignalValueWire::Bool(value) => SignalValue::Bool(value),
            SignalValueWire::Number(value) => SignalValue::Number(value),
            SignalValueWire::String(value) => SignalValue::String(value),
            SignalValueWire::Array(items) => SignalValue::Array(items),
            SignalValueWire::Object(fields) => SignalValue::Object(fields.into_iter().collect()),
        })
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
