use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::policies::RelationalRuntimeProfile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConfigValueSource {
    ProfileDefault,
    BuilderOverride,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigProvenanceEntry {
    pub source: ConfigValueSource,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigProvenance {
    pub profile: RelationalRuntimeProfile,
    pub entries: BTreeMap<String, ConfigProvenanceEntry>,
}

impl ConfigProvenance {
    pub fn source_for(&self, key: &str) -> Option<&ConfigProvenanceEntry> {
        self.entries.get(key)
    }
}
