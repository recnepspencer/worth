//! Ordinary fact writers maintain selected retained charge facts per key.
use crate::data::persistent_ord_map::{RetainedMapMutationDenial, RetainedMapMutationOutcome};
use crate::data::retained_storage::RetainedStoragePreparation as Work;
use crate::diagnostics::facts::{ExplanationFact, ProvenanceFact};
use crate::diagnostics::policy::ArtifactRetentionPolicy;

use super::super::DiagnosticsState;

impl DiagnosticsState {
    pub fn record_explanation_fact(&mut self, fact: ExplanationFact) {
        if self.installed_retention_budget.explanation_retention != ArtifactRetentionPolicy::Retain
        {
            return;
        }
        let mut work = Work::new(usize::MAX);
        if self.explanation_facts.prepared_retained_charge().is_err() {
            self.explanation_facts
                .prepare_retained_charge(&mut work)
                .expect("ordinary explanation fact root reconstitution");
        }
        expect_accounted(
            self.explanation_facts
                .insert_with_retained_charge(fact.node, fact, &mut work),
        );
    }

    pub fn record_provenance_fact(&mut self, fact: ProvenanceFact) {
        if self.installed_retention_budget.provenance_retention != ArtifactRetentionPolicy::Retain {
            return;
        }
        let mut work = Work::new(usize::MAX);
        if self.provenance_facts.prepared_retained_charge().is_err() {
            self.provenance_facts
                .prepare_retained_charge(&mut work)
                .expect("ordinary provenance fact root reconstitution");
        }
        expect_accounted(
            self.provenance_facts
                .insert_with_retained_charge(fact.node, fact, &mut work),
        );
    }
}

fn expect_accounted<T: std::fmt::Debug>(
    result: Result<RetainedMapMutationOutcome<Option<T>>, RetainedMapMutationDenial>,
) {
    match result.expect("ordinary fact root was prepared") {
        RetainedMapMutationOutcome::Accounted { .. } => {}
        RetainedMapMutationOutcome::Unaccounted { denial, .. } => {
            panic!("ordinary fact charge failed: {denial:?}")
        }
    }
}
