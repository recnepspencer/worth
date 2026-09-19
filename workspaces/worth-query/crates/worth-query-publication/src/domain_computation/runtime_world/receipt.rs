use worth_query_execution::facade::primary_graph::WorthQueryCommittedProductPublication;
use worth_runtime_world::facade::{
    CompositeCommitIdentity, CompositeComponentChangePosture, CompositePublicationAttemptIdentity,
    CompositeSignalPublicationIdentity, ProductBranchIdentity, ProductBranchIncarnation,
    ProductBranchReferenceGeneration, RelationalCommitReceipt,
};

/// Publication-owned description of the exact Runtime World transition.
/// Construction copies only owner-issued identities from a committed Query
/// terminal and carries no authority to publish, dispatch, or recover it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPublishedProductCommit {
    product_branch: ProductBranchIdentity,
    product_incarnation: ProductBranchIncarnation,
    product_generation: ProductBranchReferenceGeneration,
    composite_commit: CompositeCommitIdentity,
    publication_attempt: CompositePublicationAttemptIdentity,
    relational_commit: RelationalCommitReceipt,
    relational_posture: CompositeComponentChangePosture,
    signal_posture: CompositeComponentChangePosture,
    signal_publication: Option<CompositeSignalPublicationIdentity>,
    conditional_definition_generation: Option<u64>,
}

impl WorthQueryPublishedProductCommit {
    pub(crate) fn from_execution(publication: &WorthQueryCommittedProductPublication) -> Self {
        Self {
            product_branch: publication.product_branch().clone(),
            product_incarnation: publication.product_incarnation(),
            product_generation: publication.product_generation(),
            composite_commit: publication.composite_commit().clone(),
            publication_attempt: publication.publication_attempt().clone(),
            relational_commit: publication.relational_commit().clone(),
            relational_posture: publication.relational_posture(),
            signal_posture: publication.signal_posture(),
            signal_publication: publication.signal_publication().cloned(),
            conditional_definition_generation: publication.conditional_definition_generation(),
        }
    }

    pub const fn product_branch(&self) -> &ProductBranchIdentity {
        &self.product_branch
    }

    pub const fn product_incarnation(&self) -> ProductBranchIncarnation {
        self.product_incarnation
    }

    pub const fn product_generation(&self) -> ProductBranchReferenceGeneration {
        self.product_generation
    }

    pub const fn composite_commit(&self) -> &CompositeCommitIdentity {
        &self.composite_commit
    }

    pub const fn publication_attempt(&self) -> &CompositePublicationAttemptIdentity {
        &self.publication_attempt
    }

    pub const fn relational_commit(&self) -> &RelationalCommitReceipt {
        &self.relational_commit
    }

    pub const fn relational_posture(&self) -> CompositeComponentChangePosture {
        self.relational_posture
    }

    pub const fn signal_posture(&self) -> CompositeComponentChangePosture {
        self.signal_posture
    }

    pub const fn signal_publication(&self) -> Option<&CompositeSignalPublicationIdentity> {
        self.signal_publication.as_ref()
    }

    pub const fn conditional_definition_generation(&self) -> Option<u64> {
        self.conditional_definition_generation
    }
}
