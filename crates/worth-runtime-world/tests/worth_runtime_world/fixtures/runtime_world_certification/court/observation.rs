pub(crate) use std::collections::{BTreeMap, BTreeSet};

/// Semantic projection only: no runtime identities, authority, or algorithms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SupplyChainObservation {
    pub(crate) records: BTreeMap<String, String>,
    pub(crate) links: BTreeSet<(String, String)>,
}
