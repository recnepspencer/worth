//! Public Bank description of one committed application operation.

mod canonical_work;
mod public_description;

pub use canonical_work::{BankCommitCanonicalWorkEvidence, BankCommitCanonicalWorkPhases};

use worth_query_host::facade::publication::application_aftermath::{
    WorthQueryPublishedApplicationAftermath, WorthQueryPublishedExternalEffectPosture,
};
use worth_query_host::facade::publication::domain_computation::WorthQueryApplicationCommitPublicationReceipt;

use super::publication_adapter::BankCommitPublicationProjection;
use super::recovery_evidence::BankCommitRecoveryEvidence;
use public_description::BankCommitPublicDescription;

/// Public Bank receipt backed by closed publication and Bank descriptions.
///
/// The execution receipt is retained only inside the crate-private recovery
/// carrier. `Debug` and equality deliberately observe only the closed public
/// description and cannot reveal or compare execution-owned evidence.
#[derive(Clone)]
pub struct BankCommitReceipt {
    publication: WorthQueryApplicationCommitPublicationReceipt,
    description: BankCommitPublicDescription,
    recovery: BankCommitRecoveryEvidence,
}

impl BankCommitReceipt {
    pub(super) fn from_publication_projection(projection: BankCommitPublicationProjection) -> Self {
        let (publication, recovery_description) = projection.into_parts();
        let description = BankCommitPublicDescription::from_execution(&recovery_description);
        let recovery = BankCommitRecoveryEvidence::from_execution(recovery_description);
        Self {
            publication,
            description,
            recovery,
        }
    }

    pub const fn changed_record_count(&self) -> usize {
        self.publication.boundary_evidence().changed_record_count()
    }

    pub const fn emitted_effect_count(&self) -> usize {
        self.publication.boundary_evidence().emitted_effect_count()
    }

    pub const fn expected_version_count(&self) -> usize {
        self.description.expected_version_count()
    }

    pub const fn expected_fact_count(&self) -> usize {
        self.description.expected_fact_count()
    }

    pub const fn decision_fact_count(&self) -> Option<usize> {
        match self.publication.boundary_evidence().mutation_work() {
            Some(work) => Some(work.decision_fact_count()),
            None => None,
        }
    }

    pub const fn canonical_work(&self) -> BankCommitCanonicalWorkPhases {
        self.description.canonical_work()
    }

    pub const fn publication(&self) -> &WorthQueryApplicationCommitPublicationReceipt {
        &self.publication
    }

    pub const fn aftermath(&self) -> &WorthQueryPublishedApplicationAftermath {
        self.publication.aftermath()
    }

    pub const fn external_dispatch_posture(
        &self,
    ) -> Option<WorthQueryPublishedExternalEffectPosture> {
        match self.aftermath().external_effect() {
            WorthQueryPublishedExternalEffectPosture::NotDeclared
            | WorthQueryPublishedExternalEffectPosture::PendingDispatch => None,
            posture => Some(posture),
        }
    }

    pub const fn co_committed_dispatch_outbox(&self) -> bool {
        self.description.co_committed_dispatch_outbox()
    }

    pub const fn retained_preimage(&self) -> bool {
        self.description.retained_preimage()
    }

    pub const fn performed_preimage_retention_work(&self) -> bool {
        self.description.performed_preimage_retention_work()
    }

    pub(crate) const fn recovery_evidence(&self) -> &BankCommitRecoveryEvidence {
        &self.recovery
    }
}

impl std::fmt::Debug for BankCommitReceipt {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BankCommitReceipt")
            .field("publication", &self.publication)
            .field("description", &self.description)
            .finish()
    }
}

impl PartialEq for BankCommitReceipt {
    fn eq(&self, other: &Self) -> bool {
        self.publication == other.publication && self.description == other.description
    }
}

impl Eq for BankCommitReceipt {}
