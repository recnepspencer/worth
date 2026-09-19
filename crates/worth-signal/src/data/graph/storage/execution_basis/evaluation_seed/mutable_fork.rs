use super::SignalEvaluationStorage;
use crate::data::retained_storage::SignalConditionalRetentionReservation as Reservation;

impl SignalEvaluationStorage {
    /// Fork current derived roots, retaining accumulated slot diagnostics.
    /// The caller owns old/draft capacity and performed-observation custody.
    #[cfg(test)]
    pub(in crate::data::graph) fn fork_persistent(&mut self) -> Self {
        self.fork_with_resources(None)
    }
    pub(in crate::data::graph) fn fork_reserved(&mut self, resources: &mut Reservation) -> Self {
        self.fork_with_resources(Some(resources))
    }
    fn fork_with_resources(&mut self, mut resources: Option<&mut Reservation>) -> Self {
        macro_rules! fork {
            ($root:expr) => {
                match resources.as_deref_mut() {
                    Some(resources) => $root.fork_reserved(resources),
                    None => $root.fork_persistent(),
                }
            };
        }
        let Self {
            hot,
            warm,
            cold,
            retained_node_ledger,
            retained_node_custody,
            retained_seed_custody,
            fork_custody,
            topology,
            compaction,
            causes,
            cause_readmission_required,
            conditional_versions,
            conditional_versions_custody,
            repeated_admissions,
            partitions,
            diagnostics,
        } = self;
        Self {
            hot: fork!(hot),
            warm: fork!(warm),
            cold: fork!(cold),
            retained_node_ledger: retained_node_ledger.clone(),
            retained_node_custody: retained_node_custody.clone(),
            retained_seed_custody: retained_seed_custody.clone(),
            fork_custody: fork_custody.clone(),
            topology: fork!(topology),
            compaction: compaction.clone(),
            causes: fork!(causes),
            cause_readmission_required: *cause_readmission_required,
            conditional_versions: fork!(conditional_versions),
            conditional_versions_custody: conditional_versions_custody.clone(),
            repeated_admissions: fork!(repeated_admissions),
            partitions: fork!(partitions),
            diagnostics: fork!(diagnostics),
        }
    }
}
