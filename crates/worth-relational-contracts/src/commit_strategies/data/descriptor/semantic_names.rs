use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize};

macro_rules! semantic_name_type {
    ($type_name:ident, $label:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        pub struct $type_name(String);

        impl $type_name {
            pub fn new(name: impl Into<String>) -> Self {
                Self(name.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl<'de> Deserialize<'de> for $type_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserialize_non_empty_name(deserializer, $label).map(Self)
            }
        }
    };
}

semantic_name_type!(CommitStrategySemanticName, "commit strategy semantic name");
semantic_name_type!(CommitStrategyFamilyName, "commit strategy family name");
semantic_name_type!(StrategyInputSchemaName, "strategy input schema name");
semantic_name_type!(StrategyOutputSchemaName, "strategy output schema name");
semantic_name_type!(StrategyIntentName, "strategy intent name");
semantic_name_type!(PersistentArtifactName, "persistent artifact name");

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StrategyInputSchemaVersion(pub u16);

fn deserialize_non_empty_name<'de, D>(
    deserializer: D,
    label: &'static str,
) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value.trim().is_empty() {
        return Err(D::Error::custom(format!("{label} must not be empty")));
    }
    Ok(value)
}
