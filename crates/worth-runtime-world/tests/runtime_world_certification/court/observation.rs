use std::collections::{BTreeMap, BTreeSet};

/// Semantic projection only: no runtime identities, authority, or algorithms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupplyChainObservation {
    pub records: BTreeMap<String, String>,
    pub links: BTreeSet<(String, String)>,
}
