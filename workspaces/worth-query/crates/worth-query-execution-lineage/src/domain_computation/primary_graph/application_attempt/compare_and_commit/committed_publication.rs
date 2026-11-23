use std::sync::Arc;
use worth_query_execution_runtime::facade::ports::ProductPublicationReceiptInternalPort as _;

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
    /// An actual group member shares the committed endpoint and owns no delivery.
    CoCommitted {
        description: Arc<DescriptiveProductPublication>,
        endpoint: Box<crate::basis::WorthQueryProductBranchReadIdentity>,
    },
    /// Cached Ready publication identity with exact descriptive before/after
    /// endpoints. Retains no World history hold, consumed publication, or Native basis.
    SettledMovement {
        description: Arc<DescriptiveProductPublication>,
        before: Box<crate::basis::WorthQueryProductBranchReadIdentity>,
        after: Box<crate::basis::WorthQueryProductBranchReadIdentity>,
    },
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
    /// Issued only while detaching an actual completed group member's delivery.
    /// The endpoint is descriptive and retains no World or owner resources.
    pub(crate) fn co_committed_member(&self) -> Self {
        let endpoint = self
            .observed_product_identity()
            .expect("actual group member still holds its performed World publication");
        let description = match self.descriptive().custody {
            PublicationCustody::Described(description) => description,
            _ => unreachable!("describing a performed member yields immutable identities"),
        };
        Self {
            custody: PublicationCustody::CoCommitted {
                description,
                endpoint: Box::new(endpoint),
            },
        }
    }
    pub(crate) fn co_committed_at(&self, observation: &ProductBranchObservation) -> bool {
        matches!(&self.custody,PublicationCustody::CoCommitted{endpoint,..} if endpoint.matches_observation(observation))
    }

    pub(crate) fn from_receipt(receipt: WorthQueryProductPublicationReceipt) -> Self {
        Self {
            custody: PublicationCustody::Performed(receipt),
        }
    }

    /// The same identities without the performed publication, once its
    /// commit's history retired, so holding it pins no World or owner state.
    pub(crate) fn retired(&self) -> Self {
        self.descriptive()
    }

    /// Preserve original immutable identities without retaining physical history.
    /// This describes the copy; it does not retire the actual publication.
    pub(crate) fn descriptive(&self) -> Self {
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
            PublicationCustody::Described(_)
            | PublicationCustody::CoCommitted { .. }
            | PublicationCustody::SettledMovement { .. } => None,
        }
    }

    pub(crate) fn take_fresh_product_change(
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
            PublicationCustody::Described(description)
            | PublicationCustody::CoCommitted { description, .. }
            | PublicationCustody::SettledMovement { description, .. } => {
                &description.product_branch
            }
        }
    }

    pub fn product_incarnation(&self) -> ProductBranchIncarnation {
        match &self.custody {
            PublicationCustody::Performed(receipt) => receipt
                .publication()
                .new_product_head()
                .lifecycle_incarnation(),
            PublicationCustody::Described(description)
            | PublicationCustody::CoCommitted { description, .. }
            | PublicationCustody::SettledMovement { description, .. } => {
                description.product_incarnation
            }
        }
    }

    pub fn product_generation(&self) -> ProductBranchReferenceGeneration {
        match &self.custody {
            PublicationCustody::Performed(receipt) => receipt
                .publication()
                .new_product_head()
                .reference_generation(),
            PublicationCustody::Described(description)
            | PublicationCustody::CoCommitted { description, .. }
            | PublicationCustody::SettledMovement { description, .. } => {
                description.product_generation
            }
        }
    }

    pub fn composite_commit(&self) -> &CompositeCommitIdentity {
        match &self.custody {
            PublicationCustody::Performed(receipt) => receipt.publication().commit().identity(),
            PublicationCustody::Described(description)
            | PublicationCustody::CoCommitted { description, .. }
            | PublicationCustody::SettledMovement { description, .. } => {
                &description.composite_commit
            }
        }
    }

    /// Whether `observation` still selects this publication: nothing has been
    /// published on its branch since, so the effect it committed is current
    /// with nothing to compare.
    pub(crate) fn is_selected_at(&self, observation: &ProductBranchObservation) -> bool {
        observation.lifecycle_incarnation() == self.product_incarnation()
            && observation.reference_generation() == self.product_generation()
            && observation.selected_commit() == self.composite_commit()
    }

    pub fn publication_attempt(&self) -> &CompositePublicationAttemptIdentity {
        match &self.custody {
            PublicationCustody::Performed(receipt) => receipt.publication().attempt_identity(),
            PublicationCustody::Described(description)
            | PublicationCustody::CoCommitted { description, .. }
            | PublicationCustody::SettledMovement { description, .. } => {
                &description.publication_attempt
            }
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
            PublicationCustody::Described(description)
            | PublicationCustody::CoCommitted { description, .. }
            | PublicationCustody::SettledMovement { description, .. } => {
                &description.relational_commit
            }
        }
    }

    pub fn relational_posture(&self) -> CompositeComponentChangePosture {
        match &self.custody {
            PublicationCustody::Performed(receipt) => receipt
                .publication()
                .component_results()
                .relational_posture(),
            PublicationCustody::Described(description)
            | PublicationCustody::CoCommitted { description, .. }
            | PublicationCustody::SettledMovement { description, .. } => {
                description.relational_posture
            }
        }
    }

    pub fn signal_posture(&self) -> CompositeComponentChangePosture {
        match &self.custody {
            PublicationCustody::Performed(receipt) => {
                receipt.publication().component_results().signal_posture()
            }
            PublicationCustody::Described(description)
            | PublicationCustody::CoCommitted { description, .. }
            | PublicationCustody::SettledMovement { description, .. } => description.signal_posture,
        }
    }

    pub fn signal_publication(&self) -> Option<&CompositeSignalPublicationIdentity> {
        match &self.custody {
            PublicationCustody::Performed(receipt) => {
                receipt.publication().commit().signal_publication_identity()
            }
            PublicationCustody::Described(description)
            | PublicationCustody::CoCommitted { description, .. }
            | PublicationCustody::SettledMovement { description, .. } => {
                description.signal_publication.as_ref()
            }
        }
    }

    pub fn conditional_definition_generation(&self) -> Option<u64> {
        match &self.custody {
            PublicationCustody::Performed(receipt) => receipt.conditional_definition_generation(),
            PublicationCustody::Described(description)
            | PublicationCustody::CoCommitted { description, .. }
            | PublicationCustody::SettledMovement { description, .. } => {
                description.conditional_definition_generation
            }
        }
    }

    /// Borrow the actual native poststate retained by a performed World owner.
    /// Descriptive identity cannot reconstruct or lend this custody.
    pub(crate) fn retained_relational_basis(
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

    pub(crate) fn take_successor_observation(
        &self,
    ) -> Option<worth_runtime_world::facade::ProductBranchObservation> {
        self.performed()?.take_live_successor_observation()
    }

    pub(crate) fn take_output_demand_observation(
        &self,
    ) -> Option<worth_runtime_world::facade::ProductBranchObservation> {
        self.performed()?.take_output_demand_observation()
    }

    pub(crate) fn take_client_observation(
        &self,
    ) -> Option<worth_runtime_world::facade::ProductBranchObservation> {
        self.performed()?.take_client_observation()
    }

    pub fn has_output_demand_observation_for_test(&self) -> bool {
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
pub mod readmitted;
