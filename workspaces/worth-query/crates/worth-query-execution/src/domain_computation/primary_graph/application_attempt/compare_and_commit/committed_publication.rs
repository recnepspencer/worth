use std::sync::Arc;

use worth_relational::facade::history::RelationalCommitReceipt;
use worth_runtime_world::facade::{
    CompositeCommitIdentity, CompositeComponentChangePosture, CompositePublicationAttemptIdentity,
    CompositeSignalPublicationIdentity, ProductBranchIdentity, ProductBranchIncarnation,
    ProductBranchReferenceGeneration,
};

use crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationReceipt;

/// Descriptive identity of the exact World transition that committed a Query
/// application. Construction requires World's permanently consumed delivery.
#[derive(Clone)]
pub struct WorthQueryCommittedProductPublication {
    custody: PublicationCustody,
}

#[derive(Clone)]
enum PublicationCustody {
    /// The performed publication, shared with every live observer of it.
    Performed(WorthQueryProductPublicationReceipt),
    /// What stays once the commit's history retired: its identities, without
    /// the performed facts or the owner bases they hold.
    Retired(Arc<RetiredProductPublication>),
}

struct RetiredProductPublication {
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
            PublicationCustody::Retired(_) => None,
        }
    }

    /// Keeps this copy's commit in World history for as long as it lives.
    pub(in crate::domain_computation::primary_graph) fn hold_history(
        &mut self,
        hold: crate::domain_computation::execution_runtime::product_world::WorthQueryCommitHistoryHold,
    ) {
        if let PublicationCustody::Performed(receipt) = &mut self.custody {
            receipt.hold_history(hold);
        }
    }

    /// The same identities without the performed publication, once its
    /// commit's history retired, so holding it pins no World or owner state.
    pub(in crate::domain_computation::primary_graph) fn retired(&self) -> Self {
        if let PublicationCustody::Retired(_) = &self.custody {
            return self.clone();
        }
        let retired = RetiredProductPublication {
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
            custody: PublicationCustody::Retired(Arc::new(retired)),
        }
    }

    fn performed(&self) -> Option<&WorthQueryProductPublicationReceipt> {
        match &self.custody {
            PublicationCustody::Performed(receipt) => Some(receipt),
            PublicationCustody::Retired(_) => None,
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
            PublicationCustody::Retired(retired) => &retired.product_branch,
        }
    }

    pub fn product_incarnation(&self) -> ProductBranchIncarnation {
        match &self.custody {
            PublicationCustody::Performed(receipt) => receipt
                .publication()
                .new_product_head()
                .lifecycle_incarnation(),
            PublicationCustody::Retired(retired) => retired.product_incarnation,
        }
    }

    pub fn product_generation(&self) -> ProductBranchReferenceGeneration {
        match &self.custody {
            PublicationCustody::Performed(receipt) => receipt
                .publication()
                .new_product_head()
                .reference_generation(),
            PublicationCustody::Retired(retired) => retired.product_generation,
        }
    }

    pub fn composite_commit(&self) -> &CompositeCommitIdentity {
        match &self.custody {
            PublicationCustody::Performed(receipt) => receipt.publication().commit().identity(),
            PublicationCustody::Retired(retired) => &retired.composite_commit,
        }
    }

    pub fn publication_attempt(&self) -> &CompositePublicationAttemptIdentity {
        match &self.custody {
            PublicationCustody::Performed(receipt) => receipt.publication().attempt_identity(),
            PublicationCustody::Retired(retired) => &retired.publication_attempt,
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
            PublicationCustody::Retired(retired) => &retired.relational_commit,
        }
    }

    pub fn relational_posture(&self) -> CompositeComponentChangePosture {
        match &self.custody {
            PublicationCustody::Performed(receipt) => receipt
                .publication()
                .component_results()
                .relational_posture(),
            PublicationCustody::Retired(retired) => retired.relational_posture,
        }
    }

    pub fn signal_posture(&self) -> CompositeComponentChangePosture {
        match &self.custody {
            PublicationCustody::Performed(receipt) => {
                receipt.publication().component_results().signal_posture()
            }
            PublicationCustody::Retired(retired) => retired.signal_posture,
        }
    }

    pub fn signal_publication(&self) -> Option<&CompositeSignalPublicationIdentity> {
        match &self.custody {
            PublicationCustody::Performed(receipt) => {
                receipt.publication().commit().signal_publication_identity()
            }
            PublicationCustody::Retired(retired) => retired.signal_publication.as_ref(),
        }
    }

    pub fn conditional_definition_generation(&self) -> Option<u64> {
        match &self.custody {
            PublicationCustody::Performed(receipt) => receipt.conditional_definition_generation(),
            PublicationCustody::Retired(retired) => retired.conditional_definition_generation,
        }
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

    #[cfg(feature = "test-primary-graph-faults")]
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

/// The identities the performed receipt compares, so a retired copy equals
/// the publication it was taken from.
impl PartialEq for WorthQueryCommittedProductPublication {
    fn eq(&self, other: &Self) -> bool {
        self.composite_commit() == other.composite_commit()
            && self.publication_attempt() == other.publication_attempt()
            && self.conditional_definition_generation() == other.conditional_definition_generation()
    }
}

impl Eq for WorthQueryCommittedProductPublication {}
