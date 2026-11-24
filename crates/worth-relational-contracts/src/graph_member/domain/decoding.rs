//! Scalar-or-projected grammar without buffering an untagged Content tree.
use super::{KindId, NativeMemberDomain, ProjectedMemberDomainIdentity};
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;

impl<'de> Deserialize<'de> for NativeMemberDomain {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(DomainVisitor)
    }
}
struct DomainVisitor;
impl<'de> Visitor<'de> for DomainVisitor {
    type Value = NativeMemberDomain;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a u32 entity kind or a projected domain map")
    }
    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
        u32::try_from(value)
            .map(|kind| NativeMemberDomain::Entity(KindId(kind)))
            .map_err(|_| E::invalid_value(de::Unexpected::Unsigned(value), &self))
    }
    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
        u32::try_from(value)
            .map(|kind| NativeMemberDomain::Entity(KindId(kind)))
            .map_err(|_| E::invalid_value(de::Unexpected::Signed(value), &self))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut projected = None;
        while let Some(field) = map.next_key::<DomainField>()? {
            match field {
                DomainField::Projected => {
                    if projected.is_some() {
                        return Err(de::Error::duplicate_field("projected"));
                    }
                    projected = Some(map.next_value::<ProjectedMemberDomainIdentity>()?);
                }
                DomainField::Other => {
                    map.next_value::<de::IgnoredAny>()?;
                }
            }
        }
        Ok(NativeMemberDomain::Projected {
            projected: projected.ok_or_else(|| de::Error::missing_field("projected"))?,
        })
    }
}
// A field identifier borrows its input; unknown field spelling allocates no key.
enum DomainField {
    Projected,
    Other,
}
impl<'de> Deserialize<'de> for DomainField {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct FieldVisitor;
        impl Visitor<'_> for FieldVisitor {
            type Value = DomainField;
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a domain field name")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(if value == "projected" {
                    DomainField::Projected
                } else {
                    DomainField::Other
                })
            }
        }
        deserializer.deserialize_identifier(FieldVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn domain_scalar_and_projected_preserve_json_and_messagepack_grammar() {
        let inputs = [
            "17",
            r#"{"projected":{"schema":"sample","version":3,"name":"case-domain"}}"#,
        ];
        for input in inputs {
            let value: NativeMemberDomain = serde_json::from_str(input).unwrap();
            assert_eq!(serde_json::to_string(&value).unwrap(), input);
            let encoded = rmp_serde::to_vec_named(&value).unwrap();
            assert_eq!(
                rmp_serde::from_slice::<NativeMemberDomain>(&encoded).unwrap(),
                value
            );
        }
        for input in [
            "-1",
            "4294967296",
            "17.0",
            "[]",
            "{}",
            r#"{"projected":null}"#,
        ] {
            assert!(
                serde_json::from_str::<NativeMemberDomain>(input).is_err(),
                "{input}"
            );
        }
        let duplicate = r#"{"projected":{"schema":"s","version":1,"name":"d"},"projected":{"schema":"s","version":1,"name":"d"}}"#;
        assert!(serde_json::from_str::<NativeMemberDomain>(duplicate).is_err());
    }
}
