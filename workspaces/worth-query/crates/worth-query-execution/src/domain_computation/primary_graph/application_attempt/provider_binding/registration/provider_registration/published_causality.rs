//! What an application attempt's World publication made live.

use super::WorthQueryPrimaryGraphApplicationAttempt;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcomeIdentity;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;

pub(in crate::domain_computation::primary_graph) struct WorthQueryPublishedApplicationCausality {
    outcome_identity: WorthQueryApplicationCommitOutcomeIdentity,
    emitted_effect_count: usize,
}

impl WorthQueryPublishedApplicationCausality {
    pub(in crate::domain_computation::primary_graph) const fn outcome_identity(
        &self,
    ) -> WorthQueryApplicationCommitOutcomeIdentity {
        self.outcome_identity
    }

    pub(in crate::domain_computation::primary_graph) const fn emitted_effect_count(&self) -> usize {
        self.emitted_effect_count
    }
}

impl WorthQueryPrimaryGraphApplicationAttempt {
    pub(in crate::domain_computation::primary_graph) fn publish_causality(
        self,
        provider: &WorthQueryPrimaryGraphProvider,
        publication: crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
    ) -> WorthQueryPublishedApplicationCausality {
        let emitted_effect_count = provider.publish_application_commit_causality(
            self.live_delivery_reservation
                .expect("World publication reserved live causality before owner effects"),
            publication,
            self.effects.into_emissions(),
        );
        WorthQueryPublishedApplicationCausality {
            outcome_identity: self.outcome_identity,
            emitted_effect_count,
        }
    }
}
