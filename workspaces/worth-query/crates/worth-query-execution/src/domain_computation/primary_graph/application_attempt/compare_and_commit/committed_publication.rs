use worth_relational::facade::history::RelationalCommitReceipt;
use worth_runtime_world::facade::{
    CompositeCommitIdentity, CompositeComponentChangePosture, CompositePublicationAttemptIdentity,
    CompositeSignalPublicationIdentity, ProductBranchIdentity, ProductBranchIncarnation,
    ProductBranchReferenceGeneration,
};

/// Descriptive identity of the exact World transition that committed a Query
/// application. Construction requires World's permanently consumed delivery.
#[derive(Clone)]
pub struct WorthQueryCommittedProductPublication {
    receipt: crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationReceipt,
}

impl WorthQueryCommittedProductPublication {
    pub(in crate::domain_computation::primary_graph) fn from_receipt(
        receipt: crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationReceipt,
    ) -> Self {
        Self { receipt }
    }

    pub fn product_branch(&self) -> &ProductBranchIdentity {
        self.receipt
            .publication()
            .new_product_head()
            .branch_identity()
    }

    pub fn product_incarnation(&self) -> ProductBranchIncarnation {
        self.receipt
            .publication()
            .new_product_head()
            .lifecycle_incarnation()
    }

    pub fn product_generation(&self) -> ProductBranchReferenceGeneration {
        self.receipt
            .publication()
            .new_product_head()
            .reference_generation()
    }

    pub fn composite_commit(&self) -> &CompositeCommitIdentity {
        self.receipt.publication().commit().identity()
    }

    pub fn publication_attempt(&self) -> &CompositePublicationAttemptIdentity {
        self.receipt.publication().attempt_identity()
    }

    /// Exact Relational result contained by this performed World publication.
    /// It is descriptive component evidence and cannot authorize publication.
    pub fn relational_commit(&self) -> &RelationalCommitReceipt {
        &self
            .receipt
            .publication()
            .component_results()
            .relational_commit_result()
            .expect("a committed Query application performs one Relational candidate")
            .envelope()
            .commit
    }

    pub fn relational_posture(&self) -> CompositeComponentChangePosture {
        self.receipt
            .publication()
            .component_results()
            .relational_posture()
    }

    pub fn signal_posture(&self) -> CompositeComponentChangePosture {
        self.receipt
            .publication()
            .component_results()
            .signal_posture()
    }

    pub fn signal_publication(&self) -> Option<&CompositeSignalPublicationIdentity> {
        self.receipt
            .publication()
            .commit()
            .signal_publication_identity()
    }

    pub fn conditional_definition_generation(&self) -> Option<u64> {
        self.receipt.conditional_definition_generation()
    }

    pub(in crate::domain_computation::primary_graph) fn take_successor_observation(
        &self,
    ) -> Option<worth_runtime_world::facade::ProductBranchObservation> {
        self.receipt.take_successor_observation()
    }
}

impl std::fmt::Debug for WorthQueryCommittedProductPublication {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryCommittedProductPublication")
            .field("product_branch", self.product_branch())
            .field("composite_commit", self.composite_commit())
            .field("publication_attempt", self.publication_attempt())
            .finish_non_exhaustive()
    }
}

impl PartialEq for WorthQueryCommittedProductPublication {
    fn eq(&self, other: &Self) -> bool {
        self.receipt == other.receipt
    }
}

impl Eq for WorthQueryCommittedProductPublication {}
