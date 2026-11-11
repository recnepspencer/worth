use std::sync::Arc;
mod readmitted;

use worth_relational::facade::history::RelationalCommitReceipt;
use worth_runtime_world::facade::{
    CompositeCommitIdentity, CompositeComponentChangePosture, CompositePublicationAttemptIdentity,
    CompositeSignalPublicationIdentity, ProductBranchIdentity, ProductBranchIncarnation,
    ProductBranchObservation, ProductBranchReferenceGeneration,
};

use crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationReceipt;

/// Descriptive identity of the exact World transition that committed a Query
/// application. Construction requires its consumed live delivery or the exact
/// readmitted World/Native source pair; descriptive recovery grants no delivery.
#[derive(Clone)]
pub struct WorthQueryCommittedProductPublication {
    custody: PublicationCustody,
}

#[derive(Clone)]
enum PublicationCustody {
    /// The performed publication, shared with every live observer of it.
    Performed(WorthQueryProductPublicationReceipt),
    /// Immutable publication identity without performed facts or owner bases.
    /// Releasing this copy does not assert retirement of the actual source.
    Described(Arc<DescriptiveProductPublication>),
}

struct DescriptiveProductPublication {
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

impl WorthQueryCommittedProductPublication {
    pub(in crate::domain_computation::primary_graph) fn preflight_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryHomeRetirementDenial> {
        match &mut self.custody {
            PublicationCustody::Performed(receipt) => receipt.preflight_home_retirement(request),
            PublicationCustody::Described(_) => Ok(()),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn advance_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> Result<
        crate::domain_computation::primary_graph::WorthQueryOperationHomeRetirementProgress,
        crate::domain_computation::primary_graph::WorthQueryHomeRetirementDenial,
    > {
        match &mut self.custody {
            PublicationCustody::Performed(receipt) => receipt.advance_home_retirement(request, native),
            PublicationCustody::Described(_) => Ok(crate::domain_computation::primary_graph::WorthQueryOperationHomeRetirementProgress::Complete),
        }
    }

    /// Canonical World facts remain available only from actual consumed custody.
    pub(in crate::domain_computation::primary_graph) fn original_movement(
        &self,
    ) -> Option<&worth_runtime_world::facade::ConsumedCompositePublication> {
        self.performed().map(|receipt| receipt.publication())
    }

    /// Join metadata to the real performed predecessor, including its admission.
    pub(in crate::domain_computation::primary_graph) fn follows_observation(
        &self,
        observation: &ProductBranchObservation,
    ) -> bool {
        self.performed().is_some_and(|receipt| {
            let before = receipt.publication().old_product_head();
            before.branch_identity() == observation.branch_identity()
                && before.lifecycle_incarnation() == observation.lifecycle_incarnation()
                && before.reference_generation() == observation.reference_generation()
                && before.selected_commit() == observation.selected_commit()
                && before.basis().relational_basis() == observation.basis().relational_basis()
        })
    }
    /// The original performed selection, without retaining its owner bases.
    /// Identity-only custody cannot supply the missing full basis.
    pub(in crate::domain_computation::primary_graph) fn observed_product_identity(
        &self,
    ) -> Option<crate::basis::WorthQueryProductBranchReadIdentity> {
        self.performed().map(|receipt| {
            crate::basis::WorthQueryProductBranchReadIdentity::from_reference_snapshot(
                receipt.publication().new_product_head(),
            )
        })
    }

    pub(in crate::domain_computation::primary_graph) fn from_receipt(
        receipt: WorthQueryProductPublicationReceipt,
    ) -> Self {
        Self {
            custody: PublicationCustody::Performed(receipt),
        }
    }

    /// Takes the World history protection out of this copy, leaving it
    /// detached; a retired publication has none.
    pub(in crate::domain_computation::primary_graph) fn take_history(
        &mut self,
    ) -> Option<
        crate::domain_computation::execution_runtime::product_world::WorthQueryCommitHistoryHold,
    > {
        match &mut self.custody {
            PublicationCustody::Performed(receipt) => receipt.take_history(),
            PublicationCustody::Described(_) => None,
        }
    }

    /// The same identities without the performed publication, once its
    /// commit's history retired, so holding it pins no World or owner state.
    pub(in crate::domain_computation::primary_graph) fn retired(&self) -> Self {
        self.descriptive()
    }

    /// Preserve original immutable identities without retaining physical history.
    /// This describes the copy; it does not retire the actual publication.
    pub(in crate::domain_computation::primary_graph) fn descriptive(&self) -> Self {
        if let PublicationCustody::Described(_) = &self.custody {
            return self.clone();
        }
        let description = DescriptiveProductPublication {
            product_branch: self.product_branch().clone(),
            product_incarnation: self.product_incarnation(),
            product_generation: self.product_generation(),
            composite_commit: self.composite_commit().clone(),
            publication_attempt: self.publication_attempt().clone(),
            relational_commit: self.relational_commit().clone(),
            relational_posture: self.relational_posture(),
            signal_posture: self.signal_posture(),
            signal_publication: self.signal_publication().cloned(),
            conditional_definition_generation: self.conditional_definition_generation(),
        };
        Self {
            custody: PublicationCustody::Described(Arc::new(description)),
        }
    }

    fn performed(&self) -> Option<&WorthQueryProductPublicationReceipt> {
        match &self.custody {
            PublicationCustody::Performed(receipt) => Some(receipt),
            PublicationCustody::Described(_) => None,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn take_fresh_product_change(
        &self,
    ) -> Option<
        crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange,
    >{
        self.performed()?.take_fresh_delivery()
    }

    pub fn product_branch(&self) -> &ProductBranchIdentity {
        match &self.custody {
            PublicationCustody::Performed(receipt) => {
                receipt.publication().new_product_head().branch_identity()
            }
            PublicationCustody::Described(description) => &description.product_branch,
        }
    }

    pub fn product_incarnation(&self) -> ProductBranchIncarnation {
        match &self.custody {
            PublicationCustody::Performed(receipt) => receipt
                .publication()
                .new_product_head()
                .lifecycle_incarnation(),
            PublicationCustody::Described(description) => description.product_incarnation,
        }
    }

    pub fn product_generation(&self) -> ProductBranchReferenceGeneration {
        match &self.custody {
            PublicationCustody::Performed(receipt) => receipt
                .publication()
                .new_product_head()
                .reference_generation(),
            PublicationCustody::Described(description) => description.product_generation,
        }
    }

    pub fn composite_commit(&self) -> &CompositeCommitIdentity {
        match &self.custody {
            PublicationCustody::Performed(receipt) => receipt.publication().commit().identity(),
            PublicationCustody::Described(description) => &description.composite_commit,
        }
    }

    /// Whether `observation` still selects this publication: nothing has been
    /// published on its branch since, so the effect it committed is current
    /// with nothing to compare.
    pub(in crate::domain_computation::primary_graph) fn is_selected_at(
        &self,
        observation: &ProductBranchObservation,
    ) -> bool {
        observation.lifecycle_incarnation() == self.product_incarnation()
            && observation.reference_generation() == self.product_generation()
            && observation.selected_commit() == self.composite_commit()
    }

    pub fn publication_attempt(&self) -> &CompositePublicationAttemptIdentity {
        match &self.custody {
            PublicationCustody::Performed(receipt) => receipt.publication().attempt_identity(),
            PublicationCustody::Described(description) => &description.publication_attempt,
        }
    }

    /// Exact Relational result contained by this performed World publication.
    /// It is descriptive component evidence and cannot authorize publication.
    pub fn relational_commit(&self) -> &RelationalCommitReceipt {
        match &self.custody {
            PublicationCustody::Performed(receipt) => {
                &receipt
                    .publication()
                    .component_results()
                    .relational_commit_result()
                    .expect("a committed Query application performs one Relational candidate")
                    .envelope()
                    .commit
            }
            PublicationCustody::Described(description) => &description.relational_commit,
        }
    }

    pub fn relational_posture(&self) -> CompositeComponentChangePosture {
        match &self.custody {
            PublicationCustody::Performed(receipt) => receipt
                .publication()
                .component_results()
                .relational_posture(),
            PublicationCustody::Described(description) => description.relational_posture,
        }
    }

    pub fn signal_posture(&self) -> CompositeComponentChangePosture {
        match &self.custody {
            PublicationCustody::Performed(receipt) => {
                receipt.publication().component_results().signal_posture()
            }
            PublicationCustody::Described(description) => description.signal_posture,
        }
    }

    pub fn signal_publication(&self) -> Option<&CompositeSignalPublicationIdentity> {
        match &self.custody {
            PublicationCustody::Performed(receipt) => {
                receipt.publication().commit().signal_publication_identity()
            }
            PublicationCustody::Described(description) => description.signal_publication.as_ref(),
        }
    }

    pub fn conditional_definition_generation(&self) -> Option<u64> {
        match &self.custody {
            PublicationCustody::Performed(receipt) => receipt.conditional_definition_generation(),
            PublicationCustody::Described(description) => {
                description.conditional_definition_generation
            }
        }
    }

    /// Borrow the actual native poststate retained by a performed World owner.
    /// Descriptive identity cannot reconstruct or lend this custody.
    pub(in crate::domain_computation::primary_graph) fn retained_relational_basis(
        &self,
    ) -> Option<&worth_relational::facade::branch::AdmittedRelationalBranchBasis> {
        Some(
            self.performed()?
                .publication()
                .new_product_head()
                .basis()
                .relational_basis(),
        )
    }

    pub(in crate::domain_computation::primary_graph) fn take_successor_observation(
        &self,
    ) -> Option<worth_runtime_world::facade::ProductBranchObservation> {
        self.performed()?.take_live_successor_observation()
    }

    pub(in crate::domain_computation::primary_graph) fn take_output_demand_observation(
        &self,
    ) -> Option<worth_runtime_world::facade::ProductBranchObservation> {
        self.performed()?.take_output_demand_observation()
    }

    pub(in crate::domain_computation::primary_graph) fn take_client_observation(
        &self,
    ) -> Option<worth_runtime_world::facade::ProductBranchObservation> {
        self.performed()?.take_client_observation()
    }

    pub(in crate::domain_computation::primary_graph) fn has_output_demand_observation_for_test(
        &self,
    ) -> bool {
        self.performed().is_some_and(
            WorthQueryProductPublicationReceipt::has_output_demand_observation_for_test,
        )
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

/// The identities the performed receipt compares, so a descriptive copy equals
/// the publication it was taken from.
impl PartialEq for WorthQueryCommittedProductPublication {
    fn eq(&self, other: &Self) -> bool {
        self.composite_commit() == other.composite_commit()
            && self.publication_attempt() == other.publication_attempt()
            && self.conditional_definition_generation() == other.conditional_definition_generation()
    }
}

impl Eq for WorthQueryCommittedProductPublication {}

mod court_doors_for_test;
