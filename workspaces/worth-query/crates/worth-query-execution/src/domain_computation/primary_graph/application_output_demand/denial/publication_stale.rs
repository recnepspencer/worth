//! Descriptions of the native refusal already selected by the producer owner.
//! These values retain neither observation custody nor permission to retry.

use super::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialStage, WorthQueryApplicationStaleAttempt,
};
use worth_relational::facade::mvcc::{CompanionCellEditStop, CompanionPreflightStop};
use worth_runtime_world::facade::{
    CompositeCommitIdentity, ProductBranchIncarnation, ProductBranchObservation,
    ProductBranchReferenceGeneration, ProductBranchReferenceSnapshot, RuntimeWorldOwnerIdentity,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum ProducerPublicationStale {
    CommitDecisionFacts(WorthQueryApplicationStaleAttempt),
    CommitProductHead {
        expected: ProductPublicationIdentity,
        observed: Option<ProductPublicationIdentity>,
    },
    CommitProductBasisStale {
        stage: WorthQueryApplicationCommitDenialStage,
    },
    CommitNoEffectPreflight(CompanionPreflightStop),
    CommitDeferredPreflight(CompanionPreflightStop),
    ReadyCurrentnessClosure(CompanionCellEditStop),
    ReadyCurrentnessRegistration(CompanionCellEditStop),
    ReadyRefreshClaimMoved {
        selected: ProductPublicationIdentity,
        rejection: crate::domain_computation::primary_graph::application_output_demand::RequiredRefreshRejection,
    },
}

/// Fixed-size descriptive identities from an existing native observation.
/// Do not retain the observation, its graph roots or its history obligations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct ProductPublicationIdentity {
    owner: RuntimeWorldOwnerIdentity,
    incarnation: ProductBranchIncarnation,
    generation: ProductBranchReferenceGeneration,
    commit: CompositeCommitIdentity,
}

impl ProductPublicationIdentity {
    pub(super) fn selected(product: &ProductBranchObservation) -> Self {
        Self {
            owner: product.owner_identity(),
            incarnation: product.lifecycle_incarnation(),
            generation: product.reference_generation(),
            commit: product.selected_commit().clone(),
        }
    }

    pub(super) fn observed(product: &ProductBranchReferenceSnapshot) -> Self {
        Self {
            owner: product.owner_identity(),
            incarnation: product.lifecycle_incarnation(),
            generation: product.reference_generation(),
            commit: product.selected_commit().clone(),
        }
    }
}

impl ProducerPublicationStale {
    pub(in crate::domain_computation::primary_graph) fn product_head(
        stale: &crate::domain_computation::product_publication::WorthQueryProductStaleApplication,
    ) -> Self {
        Self::CommitProductHead {
            expected: ProductPublicationIdentity::selected(stale.expected_product()),
            observed: stale
                .observed_product()
                .map(ProductPublicationIdentity::observed),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn ready_refresh_claim_moved(
        selected: &ProductBranchObservation,
        rejection: crate::domain_computation::primary_graph::application_output_demand::RequiredRefreshRejection,
    ) -> Self {
        Self::ReadyRefreshClaimMoved {
            selected: ProductPublicationIdentity::selected(selected),
            rejection,
        }
    }
}

impl WorthQueryOutputDemandDenial {
    pub(in crate::domain_computation::primary_graph) fn with_publication_stale(
        mut self,
        cause: Option<ProducerPublicationStale>,
    ) -> Self {
        if let Some(cause) = cause {
            self.evidence_mut().publication_stale = Some(cause);
        }
        self
    }
}
