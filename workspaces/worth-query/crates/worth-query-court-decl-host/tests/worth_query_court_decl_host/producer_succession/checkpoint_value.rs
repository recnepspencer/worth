//! Diagnostic MessagePack values retain binary fields as JSON byte arrays.
use serde::{
    de::{MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use serde_json::Value;
pub struct InspectionValue(pub Value);
impl<'de> Deserialize<'de> for InspectionValue {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(InspectionVisitor)
    }
}
struct InspectionVisitor;
impl<'de> Visitor<'de> for InspectionVisitor {
    type Value = InspectionValue;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("native diagnostic value")
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
        Ok(InspectionValue(Value::Null))
    }
    fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Self::Value, E> {
        Ok(InspectionValue(v.into()))
    }
    fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
        Ok(InspectionValue(v.into()))
    }
    fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
        Ok(InspectionValue(v.into()))
    }
    fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Self::Value, E> {
        serde_json::Number::from_f64(v)
            .map(|n| InspectionValue(n.into()))
            .ok_or_else(|| E::custom("nonfinite diagnostic number"))
    }
    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(InspectionValue(v.into()))
    }
    fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
        Ok(InspectionValue(v.into()))
    }
    fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
        Ok(InspectionValue(
            v.iter()
                .copied()
                .map(Value::from)
                .collect::<Vec<_>>()
                .into(),
        ))
    }
    fn visit_byte_buf<E: serde::de::Error>(self, v: Vec<u8>) -> Result<Self::Value, E> {
        self.visit_bytes(&v)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut values = Vec::new();
        while let Some(InspectionValue(v)) = seq.next_element()? {
            values.push(v);
        }
        Ok(InspectionValue(values.into()))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut values = serde_json::Map::new();
        while let Some((InspectionValue(key), InspectionValue(value))) = map.next_entry()? {
            let key = match key {
                Value::String(text) => text,
                other => other.to_string(),
            };
            if values.insert(key, value).is_some() {
                return Err(serde::de::Error::custom("duplicate diagnostic map key"));
            }
        }
        Ok(InspectionValue(values.into()))
    }
}
