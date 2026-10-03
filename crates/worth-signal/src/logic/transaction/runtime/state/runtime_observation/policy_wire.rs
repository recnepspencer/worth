//! Public policy encoding. Readers accept legacy v1 and emit only v2;
//! a v1 reader cannot consume the renamed Visited tier.
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{ObservationDeliveryMode, ObservationPolicy, ObservationTrigger};

mod decoding;

const CURRENT_SCHEMA: &str = "worth.signal.observation-policy.v2";
const LEGACY_SCHEMA: &str = "worth.signal.observation-policy.v1";

#[derive(Serialize, Deserialize)]
enum WireTrigger {
    Touched,
    Visited,
    Recomputed,
    MeaningfulChange,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicyWire {
    #[serde(default = "legacy_schema")]
    schema_version: String,
    trigger: WireTrigger,
    delivery_mode: ObservationDeliveryMode,
}

fn legacy_schema() -> String {
    LEGACY_SCHEMA.to_owned()
}

impl Serialize for ObservationPolicy {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ObservationPolicy", 3)?;
        state.serialize_field("schema_version", CURRENT_SCHEMA)?;
        state.serialize_field("trigger", &self.trigger)?;
        state.serialize_field("delivery_mode", &self.delivery_mode)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for ObservationPolicy {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = decoding::deserialize(deserializer)?;
        let legacy = match wire.schema_version.as_str() {
            LEGACY_SCHEMA => true,
            CURRENT_SCHEMA => false,
            _ => {
                return Err(serde::de::Error::custom(
                    "unsupported observation policy schema",
                ))
            }
        };
        let trigger = match (legacy, wire.trigger) {
            (true, WireTrigger::Touched) | (false, WireTrigger::Visited) => {
                ObservationTrigger::Visited
            }
            (_, WireTrigger::Recomputed) => ObservationTrigger::Recomputed,
            (_, WireTrigger::MeaningfulChange) => ObservationTrigger::MeaningfulChange,
            _ => {
                return Err(serde::de::Error::custom(
                    "observation tier disagrees with policy schema",
                ))
            }
        };
        Ok(Self::new(trigger, wire.delivery_mode))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn legacy_policy_is_migrated_without_emitting_legacy_names() {
        for schema in [None, Some(LEGACY_SCHEMA)] {
            let mut wire =
                serde_json::json!({"trigger":"Touched", "delivery_mode":"PerCommittedTransaction"});
            if let Some(schema) = schema {
                wire["schema_version"] = schema.into();
            }
            let policy: ObservationPolicy = serde_json::from_value(wire).unwrap();
            assert_eq!(policy, ObservationPolicy::visited());
            assert_eq!(serde_json::to_value(policy).unwrap()["trigger"], "Visited");
        }
    }

    #[test]
    fn unsupported_or_mixed_policy_schema_is_rejected() {
        for (schema, trigger) in [
            (Some("worth.signal.observation-policy.v3"), "Visited"),
            (Some(CURRENT_SCHEMA), "Touched"),
            (Some(LEGACY_SCHEMA), "Visited"),
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
                "trigger":"Touched", "delivery_mode":"PerCommittedTransaction", "version":2
            }))
            .is_err()
        );
        assert!(serde_json::from_value::<ObservationPolicy>(serde_json::json!({
            "schema_version":null, "trigger":"Touched", "delivery_mode":"PerCommittedTransaction"
        })).is_err());
    }
}
