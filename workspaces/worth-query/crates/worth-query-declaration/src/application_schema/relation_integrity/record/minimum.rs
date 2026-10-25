//! Owned target-kind names; canonical meaning retains the authored set semantics.
use super::super::ApplicationRelationMinimumReachability;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationRelationMinimumReachabilityRecord {
    pub minimum_reachable_targets: u32,
    pub target_kinds: Vec<String>,
}

impl From<ApplicationRelationMinimumReachability> for ApplicationRelationMinimumReachabilityRecord {
    fn from(authored: ApplicationRelationMinimumReachability) -> Self {
        Self {
            minimum_reachable_targets: authored.minimum_reachable_targets,
            target_kinds: authored
                .target_kinds
                .iter()
                .map(|kind| (*kind).to_owned())
                .collect(),
        }
    }
}

impl ApplicationRelationMinimumReachabilityRecord {
    pub(crate) fn normalized_kinds(&self) -> Vec<String> {
        let mut kinds = self.target_kinds.clone();
        kinds.sort_unstable();
        kinds.dedup();
        kinds
    }
}
