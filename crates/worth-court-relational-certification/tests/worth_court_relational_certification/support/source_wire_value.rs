//! Independent MessagePack DATA inspection, preserving byte strings as octet arrays.
use serde::de::{Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::Value;
use std::fmt;

pub(crate) fn read(bytes: &[u8]) -> Value {
    rmp_serde::from_slice::<WireValue>(bytes).unwrap().0
}
struct WireValue(Value);
struct WireVisitor;
impl<'de> Deserialize<'de> for WireValue {
    fn deserialize<D: Deserializer<'de>>(input: D) -> Result<Self, D::Error> {
        input.deserialize_any(WireVisitor).map(Self)
    }
}
impl<'de> Visitor<'de> for WireVisitor {
    type Value = Value;
    fn expecting(&self, output: &mut fmt::Formatter) -> fmt::Result {
        output.write_str("MessagePack scalar, byte string, sequence or named map")
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_none<E: serde::de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<Value, E> {
        Ok(value.into())
    }
    fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Value, E> {
        Ok(value.into())
    }
    fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Value, E> {
        Ok(value.into())
    }
    fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<Value, E> {
        serde_json::Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite number"))
    }
    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Value, E> {
        Ok(value.into())
    }
    fn visit_string<E: serde::de::Error>(self, value: String) -> Result<Value, E> {
        Ok(value.into())
    }
    fn visit_bytes<E: serde::de::Error>(self, value: &[u8]) -> Result<Value, E> {
        Ok(Value::Array(
            value.iter().map(|byte| Value::from(*byte)).collect(),
        ))
    }
    fn visit_byte_buf<E: serde::de::Error>(self, value: Vec<u8>) -> Result<Value, E> {
        self.visit_bytes(&value)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Value, A::Error> {
        let mut values = Vec::new();
        while let Some(WireValue(value)) = sequence.next_element()? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut values = serde_json::Map::new();
        while let Some((key, WireValue(value))) = map.next_entry::<String, WireValue>()? {
            if values.insert(key, value).is_some() {
                return Err(serde::de::Error::custom("duplicate key"));
            }
        }
        Ok(Value::Object(values))
    }
}
