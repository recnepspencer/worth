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

/// Why the product's program activation refused a branch adoption preparation.
/// Nothing was prepared or published.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryBranchAdoptionActivationDenial {
    /// The runtime's activation capacity is full. Retry later.
    CapacityExhausted,
    /// The activation registry could not allocate what the branch needs.
    AllocationRejected,
    /// The runtime has no activation for this product branch.
    UnknownProductBranch,
    /// The activation registry was unavailable.
    RegistryUnavailable,
    /// The branch's activation gate was unavailable.
    GateUnavailable,
    /// Another publication or reader is active on the branch, or a program
    /// retirement is in progress. Retry later.
    PublicationInProgress,
    /// Program support is required but is not installed, is retired, or is
    /// unavailable.
    ProgramSupportUnavailable,
}

/// Why a branch adoption could not be prepared. Nothing was published.
#[derive(Debug)]
pub enum WorthQueryBranchAdoptionPreparationDenial {
    /// The runtime has no installed program support.
    ProgramSupportUnavailable,
    /// No program activation is published.
    ProgramActivationUnavailable,
    /// The product's program activation record could not be read.
    ProgramActivationUnreadable,
    /// The product's program revision is not on the installed roster.
    ProgramActivationUnrostered,
    /// Program support would not hold the source and target programs because a
    /// retirement is in progress.
    ProgramSupportRetirementInProgress,
    /// Program support could not derive the adoption requirements from the
    /// product's current program to the target.
    Requirements(WorthQueryProgramAdoptionRequirementsDenial),
    /// The adoption requirements differ from the ones supplied. Read the
    /// requirements again and prepare again.
    RequirementsChanged,
    /// The target requires a migration and none was supplied. The carried
    /// requirement names it.
    MigrationAssessmentRequired(ApplicationProgramMigrationAssessmentRequirement),
    /// The supplied migration was sealed for a different target program.
    MigrationTargetMismatch,
    /// The product moved after the migration was sealed. Seal the migration
    /// again.
    MigrationSourceChanged,
    /// No custody disposition exists for the carried requirement.
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
    /// The supplied workflow dispositions do not legally decide the inventory.
    WorkflowDispositionRejected(
        crate::domain_computation::primary_graph::workflow::WorthQueryWorkflowDispositionDenial,
    ),
    /// A workflow entity on the branch could not be read.
    WorkflowInventoryUnreadable {
        entity: worth_relational::facade::identity::EntityId,
    },
    /// A workflow relation slot on the branch could not be read.
    WorkflowInventoryRelationUnreadable {
        partition_id: worth_relational::facade::identity::PartitionId,
        slot: usize,
    },
    /// A validation scope names an entity type the schema does not declare.
    UnknownEntityScope { entity: String },
    /// A validation scope names a relation; adoption revalidates only entity
    /// scopes.
    RelationScopeUnsupported { relation: String },
    /// Selecting what to revalidate would exceed the work limit. The fields
    /// give the limit and the work consumed.
    SelectionLimitExceeded {
        maximum_work_units: usize,
        consumed_work_units: usize,
    },
    /// The selected branch's root could not be read for adoption.
    BranchBasisUnavailable(worth_relational::facade::branch::RelationalBranchBasisDenial),
    /// The branch transaction for the adoption was not admitted.
    TransactionAdmission(
        worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial,
    ),
    /// The adoption's changes could not be staged in the branch transaction.
    TransactionStaging(worth_relational::facade::mvcc::RelationalTransactionStagingDenial),
    /// A custom invariant of the target program rejected the prepared adoption;
    /// `identity` names it.
    TargetRuleRejected {
        identity: worth_relational::facade::transactions::CustomInvariantSemanticIdentity,
    },
    /// The branch transaction could not be prepared.
    RelationalPreparation(worth_relational::facade::transactions::TransactionCommitError),
    /// The product publication could not be prepared.
    WorldPreparation(worth_runtime_world::facade::NoEffectCompositePublication),
    /// The product's program activation refused the preparation.
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
