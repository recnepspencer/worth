use worth_runtime_world::facade::{
    CompositeCommitIdentity, CompositeComponentChangePosture, CompositePublicationAttemptIdentity,
    CompositeSignalPublicationIdentity, ConsumedCompositePublication, ProductBranchIdentity,
    ProductBranchIncarnation, ProductBranchReferenceGeneration,
};

/// Descriptive identity of the exact World transition that committed a Query
/// application. Construction requires World's permanently consumed delivery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCommittedProductPublication {
    product_branch: ProductBranchIdentity,
    product_incarnation: ProductBranchIncarnation,
    product_generation: ProductBranchReferenceGeneration,
    composite_commit: CompositeCommitIdentity,
    publication_attempt: CompositePublicationAttemptIdentity,
    relational_posture: CompositeComponentChangePosture,
    signal_posture: CompositeComponentChangePosture,
    signal_publication: Option<CompositeSignalPublicationIdentity>,
    conditional_definition_generation: Option<u64>,
}

impl WorthQueryCommittedProductPublication {
    pub(in crate::domain_computation::primary_graph) fn from_consumed(
        publication: &ConsumedCompositePublication,
        conditional_definition_generation: Option<u64>,
    ) -> Self {
        let product = publication.new_product_head();
        Self {
            product_branch: product.branch_identity().clone(),
            product_incarnation: product.lifecycle_incarnation(),
            product_generation: product.reference_generation(),
            composite_commit: publication.commit().identity().clone(),
            publication_attempt: publication.attempt_identity().clone(),
            relational_posture: publication.component_results().relational_posture(),
            signal_posture: publication.component_results().signal_posture(),
            signal_publication: publication
                .component_results()
                .signal_publication_identity(),
            conditional_definition_generation,
        }
    }

    pub fn product_branch(&self) -> &ProductBranchIdentity {
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
