//! Runtime-owned admission for one physical post-commit dispatch attempt.

use std::sync::Arc;

use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::application_aftermath::external_effect::WorthQueryAdmittedExternalDispatchAttempt;
use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation;

/// Why this application runtime could not admit a physical dispatch attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQueryExternalDispatchAdmissionDenial {
    ForeignRelationalRuntime,
    ForeignProductWorld,
    PublicationCommitMismatch,
    AttemptIdentityExhausted,
}

/// Opaque, move-only ordinal minted only while the runtime admits an attempt.
pub(in crate::domain_computation) struct WorthQueryExternalDispatchAttemptOrdinal(u64);

impl WorthQueryExternalDispatchAttemptOrdinal {
    fn mint(value: u64) -> Self {
        Self(value)
    }

    pub(in crate::domain_computation) fn into_value(self) -> u64 {
        self.0
    }

    #[cfg(test)]
    pub(in crate::domain_computation) const fn value_for_test(&self) -> u64 {
        self.0
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn admit_external_dispatch_attempt(
        &self,
        committed: WorthQueryCommittedDispatchOutboxObservation,
    ) -> Result<WorthQueryAdmittedExternalDispatchAttempt, WorthQueryExternalDispatchAdmissionDenial>
    {
        self.primary_provider.observe_external_dispatch_admission();
        let relational_runtime = self
            .product_runtime
            .source
            .authoritative_source_profile()
            .runtime_instance_id();
        if committed.relational_runtime_instance_id() != relational_runtime {
            return Err(WorthQueryExternalDispatchAdmissionDenial::ForeignRelationalRuntime);
        }
        if committed
            .committed_product_publication()
            .product_branch()
            .owner_identity()
            != self.product_runtime.owner.owner_identity()
        {
            return Err(WorthQueryExternalDispatchAdmissionDenial::ForeignProductWorld);
        }
        if committed.commit_reference()
            != committed
                .committed_product_publication()
                .relational_commit()
        {
            return Err(WorthQueryExternalDispatchAdmissionDenial::PublicationCommitMismatch);
        }
        let ordinal = WorthQueryExternalDispatchAttemptOrdinal::mint(
            self.next_external_dispatch_attempt
                .fetch_update(
                    std::sync::atomic::Ordering::AcqRel,
                    std::sync::atomic::Ordering::Acquire,
                    |current| current.checked_add(1),
                )
                .map_err(|_| WorthQueryExternalDispatchAdmissionDenial::AttemptIdentityExhausted)?,
        );
        Ok(WorthQueryAdmittedExternalDispatchAttempt::seal(
            committed,
            self.runtime.authority_identity(),
            ordinal,
            Arc::clone(&self.authorization_clock),
        ))
    }
}
