//! Admit the existing scalar-or-one-key-map source grammar without Content.
use super::{AggregateIdentitySource, DerivedRelationIdentity, KindId};
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;

impl<'de> Deserialize<'de> for AggregateIdentitySource {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(SourceVisitor)
    }
}
struct SourceVisitor;
impl<'de> Visitor<'de> for SourceVisitor {
    type Value = AggregateIdentitySource;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a u32 relation kind or one aggregate source map")
    }
    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
        u32::try_from(value)
            .map(|kind| AggregateIdentitySource::Orbit(KindId(kind)))
            .map_err(|_| E::invalid_value(de::Unexpected::Unsigned(value), &self))
    }
    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
        u32::try_from(value)
            .map(|kind| AggregateIdentitySource::Orbit(KindId(kind)))
            .map_err(|_| E::invalid_value(de::Unexpected::Signed(value), &self))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let key = map
            .next_key::<SourceField>()?
            .ok_or_else(|| de::Error::custom("missing aggregate source"))?;
        let source = match key {
            SourceField::Relations => AggregateIdentitySource::Components {
                relations: map.next_value::<Vec<KindId>>()?,
            },
            SourceField::Orbit => AggregateIdentitySource::DerivedOrbit {
                derived_orbit: map.next_value::<DerivedRelationIdentity>()?,
            },
            SourceField::Components => AggregateIdentitySource::DerivedComponents {
                derived_components: map.next_value::<DerivedRelationIdentity>()?,
            },
        };
        // Every original map variant denied unknown fields: no mixed or
        // repeated source key may be ignored after the selected value.
        if map.next_key::<SourceField>()?.is_some() {
            return Err(de::Error::custom("duplicate or mixed aggregate source"));
        }
        Ok(source)
    }
}
enum SourceField {
    Relations,
    Orbit,
    Components,
}
impl<'de> Deserialize<'de> for SourceField {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct FieldVisitor;
        impl Visitor<'_> for FieldVisitor {
            type Value = SourceField;
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an aggregate source key")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                match value {
                    "relations" => Ok(SourceField::Relations),
                    "derived_orbit" => Ok(SourceField::Orbit),
                    "derived_components" => Ok(SourceField::Components),
                    _ => Err(E::unknown_field(
                        value,
                        &["relations", "derived_orbit", "derived_components"],
                    )),
                }
            }
        }
        deserializer.deserialize_identifier(FieldVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aggregate_source_preserves_scalar_and_exact_map_codecs() {
        for input in [
            "9",
            r#"{"relations":[3,7]}"#,
            r#"{"derived_orbit":{"schema":"sample","version":2,"name":"orbit"}}"#,
            r#"{"derived_components":{"schema":"sample","version":2,"name":"components"}}"#,
        ] {
            let source: AggregateIdentitySource = serde_json::from_str(input).unwrap();
            assert_eq!(serde_json::to_string(&source).unwrap(), input);
            let bytes = rmp_serde::to_vec_named(&source).unwrap();
            assert_eq!(
                rmp_serde::from_slice::<AggregateIdentitySource>(&bytes).unwrap(),
                source
            );
        }
        for input in [
            "-1",
            "4294967296",
            "[]",
            "{}",
            r#"{"relations":[],"other":0}"#,
            r#"{"relations":[],"relations":[]}"#,
            r#"{"relations":[],"derived_orbit":{"schema":"s","version":1,"name":"d"}}"#,
        ] {
            assert!(
                serde_json::from_str::<AggregateIdentitySource>(input).is_err(),
                "{input}"
            );
        }
    }
}
