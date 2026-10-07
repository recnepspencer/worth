//! Public policy encoding. The wire names its schema so a reader that
//! predates the Visited tier rejects it instead of misreading it.
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{ObservationDeliveryMode, ObservationPolicy, ObservationTrigger};

#[derive(Clone, Copy, Serialize, Deserialize)]
enum PolicySchema {
    #[serde(rename = "worth.signal.observation-policy.v2")]
    V2,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicyWire {
    schema_version: PolicySchema,
    trigger: ObservationTrigger,
    delivery_mode: ObservationDeliveryMode,
}

impl Serialize for ObservationPolicy {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        PolicyWire {
            schema_version: PolicySchema::V2,
            trigger: self.trigger,
            delivery_mode: self.delivery_mode,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ObservationPolicy {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = PolicyWire::deserialize(deserializer)?;
        Ok(Self::new(wire.trigger, wire.delivery_mode))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CURRENT_SCHEMA: &str = "worth.signal.observation-policy.v2";

    #[test]
    fn visited_emits_versioned_public_vocabulary() {
        let wire = serde_json::to_value(ObservationPolicy::visited()).unwrap();
        assert_eq!(
            wire,
            serde_json::json!({
                "schema_version": CURRENT_SCHEMA,
                "trigger": "Visited",
                "delivery_mode": "PerCommittedTransaction"
            })
        );
        assert_eq!(
            serde_json::from_value::<ObservationPolicy>(wire).unwrap(),
            ObservationPolicy::visited()
        );
    }

    #[test]
    fn only_the_current_schema_and_visited_spelling_decode() {
        for (schema, trigger) in [
            (Some("worth.signal.observation-policy.v3"), "Visited"),
            (Some("worth.signal.observation-policy.v1"), "Touched"),
            (Some("worth.signal.observation-policy.v1"), "Visited"),
            (Some(CURRENT_SCHEMA), "Touched"),
            (None, "Touched"),
            (None, "Visited"),
        ] {
            let mut wire =
                serde_json::json!({"trigger":trigger,"delivery_mode":"PerCommittedTransaction"});
            if let Some(schema) = schema {
                wire["schema_version"] = schema.into();
            }
            assert!(serde_json::from_value::<ObservationPolicy>(wire).is_err());
        }
        assert!(
            serde_json::from_value::<ObservationPolicy>(serde_json::json!({
                "schema_version":CURRENT_SCHEMA, "trigger":"Visited",
                "delivery_mode":"PerCommittedTransaction", "version":2
            }))
            .is_err()
        );
        assert!(serde_json::from_value::<ObservationPolicy>(serde_json::json!({
            "schema_version":null, "trigger":"Visited", "delivery_mode":"PerCommittedTransaction"
        }))
        .is_err());
    }
}
