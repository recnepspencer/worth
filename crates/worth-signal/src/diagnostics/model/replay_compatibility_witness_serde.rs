use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize)]
struct CompatibilityWitnessRecord {
    schema_version: String,
    fact_inventory: SignalMergeCompatibilityFactInventory,
    compatibility_digest: String,
}
