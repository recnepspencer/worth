mod dispositions;
mod migration;
mod selection;
mod state;
mod workflow;

pub use dispositions::{
    WorthQueryProgramCustodyDisposition, WorthQueryProgramCustodyDispositionInventory,
    WorthQueryProgramCustodyDispositionKind,
};
pub use migration::{
    WorthQueryAdmittedProgramMigration, WorthQueryPreparedProgramMigration,
    WorthQueryProgramMigrationDescription, WorthQueryProgramMigrationPreparationDenial,
};

use worth_query_declaration::facade::application_program::ApplicationProgramMigrationAssessmentRequirement;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryProgramAdoptionRequirements,
    WorthQueryProgramAdoptionRequirementsDenial, WorthQueryProgramCustodyInventoryRequirement,
};

use super::super::WorthQuerySelectedProductOperation;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryBranchAdoptionActivationDenial {
    CapacityExhausted,
    AllocationRejected,
    UnknownProductBranch,
    RegistryUnavailable,
    GateUnavailable,
    PublicationInProgress,
    ProgramSupportUnavailable,
}

#[derive(Debug)]
pub enum WorthQueryBranchAdoptionPreparationDenial {
    ProgramSupportUnavailable,
    ProgramActivationUnavailable,
    ProgramActivationUnreadable,
    ProgramActivationUnrostered,
    ProgramSupportRetirementInProgress,
    Requirements(WorthQueryProgramAdoptionRequirementsDenial),
    RequirementsChanged,
    MigrationAssessmentRequired(ApplicationProgramMigrationAssessmentRequirement),
    MigrationTargetMismatch,
    MigrationSourceChanged,
    CustodyDispositionUnsupported(WorthQueryProgramCustodyInventoryRequirement),
    /// The branch holds workflow facts and no dispositions were supplied.
    /// Decide each occurrence of the carried inventory and prepare again.
    WorkflowDispositionRequired {
        inventory: Box<
            crate::domain_computation::primary_graph::workflow::WorthQueryWorkflowAdoptionInventory,
        >,
    },
    /// Owner truth moved after the dispositions were decided. Decide the
    /// carried inventory and prepare again.
    WorkflowInventoryChanged {
        inventory: Box<
            crate::domain_computation::primary_graph::workflow::WorthQueryWorkflowAdoptionInventory,
        >,
    },
    WorkflowDispositionRejected(
        crate::domain_computation::primary_graph::workflow::WorthQueryWorkflowDispositionDenial,
    ),
    WorkflowInventoryUnreadable {
        entity: worth_relational::facade::identity::EntityId,
    },
    WorkflowInventoryRelationUnreadable {
        partition_id: worth_relational::facade::identity::PartitionId,
        slot: usize,
    },
    UnknownEntityScope {
        entity: String,
    },
    RelationScopeUnsupported {
        relation: String,
    },
    SelectionLimitExceeded {
        maximum_work_units: usize,
        consumed_work_units: usize,
    },
    /// The selected branch's root could not be read for adoption.
    BranchBasisUnavailable(worth_relational::facade::branch::RelationalBranchBasisDenial),
    TransactionAdmission(
        worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial,
    ),
    TransactionStaging(worth_relational::facade::mvcc::RelationalTransactionStagingDenial),
    TargetRuleRejected {
        identity: worth_relational::facade::transactions::CustomInvariantSemanticIdentity,
    },
    RelationalPreparation(worth_relational::facade::transactions::TransactionCommitError),
    WorldPreparation(worth_runtime_world::facade::NoEffectCompositePublication),
    ProductActivation(WorthQueryBranchAdoptionActivationDenial),
}

impl From<crate::domain_computation::execution_runtime::product_world::activation::WorthQueryProductActivationDenial>
    for WorthQueryBranchAdoptionActivationDenial
{
    fn from(
        denial: crate::domain_computation::execution_runtime::product_world::activation::WorthQueryProductActivationDenial,
    ) -> Self {
        use crate::domain_computation::execution_runtime::product_world::activation::WorthQueryProductActivationDenial;
        match denial {
            WorthQueryProductActivationDenial::CapacityExhausted => Self::CapacityExhausted,
            WorthQueryProductActivationDenial::AllocationRejected => Self::AllocationRejected,
            WorthQueryProductActivationDenial::UnknownProductBranch => Self::UnknownProductBranch,
            WorthQueryProductActivationDenial::RegistryUnavailable => Self::RegistryUnavailable,
            WorthQueryProductActivationDenial::GateUnavailable => Self::GateUnavailable,
            WorthQueryProductActivationDenial::PublicationInProgress => Self::PublicationInProgress,
            WorthQueryProductActivationDenial::ProgramSupportUnavailable => {
                Self::ProgramSupportUnavailable
            }
        }
    }
}

/// Exact prepared adoption. Its private World reservation and Relational
/// candidate make it impossible to construct from a target revision alone.
#[must_use = "a prepared adoption owns a reserved publication attempt"]
pub struct WorthQueryPreparedBranchAdoption {
    pub(super) source: ApplicationProgramRevision,
    pub(super) target: ApplicationProgramRevision,
    pub(super) requirements: WorthQueryProgramAdoptionRequirements,
    pub(super) selected_entity_count: usize,
    pub(super) selection_work_units: usize,
    pub(super) migration: Option<WorthQueryProgramMigrationDescription>,
    pub(super) custody: WorthQueryProgramCustodyDispositionInventory,
    pub(super) publication: crate::domain_computation::execution_runtime::product_world::WorthQueryPreparedProductPublication,
    pub(super) recovery: worth_runtime_world::facade::RuntimeWorldRecoveryPort,
    pub(super) disposition:
        crate::domain_computation::primary_graph::WorthQueryUnpublishedIdempotencyDisposition,
    pub(super) support_custody:
        crate::domain_computation::primary_graph::program_occurrence::WorthQueryProgramSupportCustody,
}

impl WorthQueryPreparedBranchAdoption {
    pub fn source(&self) -> &ApplicationProgramRevision {
        &self.source
    }

    pub fn target(&self) -> &ApplicationProgramRevision {
        &self.target
    }

    pub fn requirements(&self) -> &WorthQueryProgramAdoptionRequirements {
        &self.requirements
    }

    pub const fn selected_entity_count(&self) -> usize {
        self.selected_entity_count
    }

    pub const fn selection_work_units(&self) -> usize {
        self.selection_work_units
    }

    pub const fn migration(&self) -> Option<&WorthQueryProgramMigrationDescription> {
        self.migration.as_ref()
    }

    pub const fn custody(&self) -> &WorthQueryProgramCustodyDispositionInventory {
        &self.custody
    }
}

pub(super) fn prepare<Schema: ApplicationSchema>(
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    target: &ApplicationProgramRevision,
    expected_requirements: &WorthQueryProgramAdoptionRequirements,
    migration: Option<WorthQueryPreparedProgramMigration>,
    workflow: Option<
        crate::domain_computation::primary_graph::workflow::WorthQueryWorkflowDispositions,
    >,
    maximum_selection_work: usize,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Result<WorthQueryPreparedBranchAdoption, WorthQueryBranchAdoptionPreparationDenial> {
    state::prepare(
        selected,
        target,
        expected_requirements,
        migration,
        workflow,
        maximum_selection_work,
        request,
    )
}

pub(super) fn workflow_inventory<Schema: ApplicationSchema>(
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    target: &ApplicationProgramRevision,
    expected_requirements: &WorthQueryProgramAdoptionRequirements,
    maximum_work_units: usize,
) -> Result<
    crate::domain_computation::primary_graph::workflow::WorthQueryWorkflowAdoptionInventory,
    WorthQueryBranchAdoptionPreparationDenial,
> {
    workflow::inventory(selected, target, expected_requirements, maximum_work_units)
}

pub(super) fn requirements<Schema: ApplicationSchema>(
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    target: &ApplicationProgramRevision,
) -> Result<WorthQueryProgramAdoptionRequirements, WorthQueryBranchAdoptionPreparationDenial> {
    state::requirements(selected, target).map(|(_, requirements)| requirements)
}

#[cfg(test)]
mod tests {
    use super::WorthQueryBranchAdoptionActivationDenial as PublicDenial;
    use crate::domain_computation::execution_runtime::product_world::activation::WorthQueryProductActivationDenial as OwnerDenial;

    #[test]
    fn activation_denials_preserve_the_owner_cause() {
        let cases = [
            (
                OwnerDenial::CapacityExhausted,
                PublicDenial::CapacityExhausted,
            ),
            (
                OwnerDenial::AllocationRejected,
                PublicDenial::AllocationRejected,
            ),
            (
                OwnerDenial::UnknownProductBranch,
                PublicDenial::UnknownProductBranch,
            ),
            (
                OwnerDenial::RegistryUnavailable,
                PublicDenial::RegistryUnavailable,
            ),
            (OwnerDenial::GateUnavailable, PublicDenial::GateUnavailable),
            (
                OwnerDenial::PublicationInProgress,
                PublicDenial::PublicationInProgress,
            ),
            (
                OwnerDenial::ProgramSupportUnavailable,
                PublicDenial::ProgramSupportUnavailable,
            ),
        ];

        for (owner, expected) in cases {
            assert_eq!(PublicDenial::from(owner), expected);
        }
    }
}
