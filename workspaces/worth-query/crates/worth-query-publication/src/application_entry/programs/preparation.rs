use worth_query_installation::facade::{ApplicationSchema, WorthQueryProgramAdoptionRequirements};

use super::WorthQueryApplicationProgramsRequest;

pub use worth_query_execution::facade::primary_graph::{
    WorthQueryPreparedBranchAdoption, WorthQueryPreparedProgramMigration,
    WorthQueryWorkflowDefinitionDisposition, WorthQueryWorkflowDispositions,
    WorthQueryWorkflowInstanceDisposition,
};

/// Why a program adoption did not prepare.
#[derive(Debug)]
pub enum WorthQueryApplicationProgramAdoptionPreparationDenial {
    Authorization(
        worth_query_execution::facade::primary_graph::WorthQueryProgramAdoptionAuthorizationDenial,
    ),
    ProductSelection(
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial,
    ),
    Adoption(
        worth_query_execution::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial,
    ),
}

/// Adoption toward the target its owner-computed requirements name, with no
/// effects performed.
/// Only `prepare` can mint the move-only value whose `publish` method exists.
///
/// A branch holding workflow facts needs an explicit total decision or the
/// native carry-compatible policy. Definition-selected governed reads disclose
/// consumer views; preparation never returns its complete safety inventory.
///
/// ```compile_fail,E0599
/// use worth_query_publication::application_entry::WorthQueryApplicationProgramAdoptionRequest;
/// use worth_query_installation::facade::ApplicationSchema;
/// fn no_complete_disclosure<S: ApplicationSchema>(request: WorthQueryApplicationProgramAdoptionRequest<'_, '_, '_, '_, S>) {
///     request.workflow_inventory(64);
/// }
/// ```
/// ```
/// use worth_query_publication::application_entry::{WorthQueryApplicationProgramAdoptionRequest, WorthQueryWorkflowDispositions};
/// use worth_query_installation::facade::ApplicationSchema;
/// fn explicit_native_intent<S: ApplicationSchema>(request: WorthQueryApplicationProgramAdoptionRequest<'_, '_, '_, '_, S>) {
///     let _owned_request = request.workflow(WorthQueryWorkflowDispositions::carry_compatible());
/// }
/// ```
///
/// ```compile_fail,E0451
/// use worth_query_execution::facade::primary_graph::WorthQueryPreparedBranchAdoption;
///
/// fn callers_cannot_forge_prepared_adoption() {
///     let _forged = WorthQueryPreparedBranchAdoption {
///         source: todo!(),
///         target: todo!(),
///         requirements: todo!(),
///         selected_entity_count: 0,
///         selection_work_units: 0,
///         publication: todo!(),
///     };
/// }
/// ```
pub struct WorthQueryApplicationProgramAdoptionRequest<
    'application,
    'principal,
    'scope,
    'requirements,
    Schema,
> {
    pub(super) programs:
        WorthQueryApplicationProgramsRequest<'application, 'principal, 'scope, Schema>,
    pub(super) requirements: &'requirements WorthQueryProgramAdoptionRequirements,
    pub(super) migration: Option<WorthQueryPreparedProgramMigration>,
    pub(super) workflow: Option<WorthQueryWorkflowDispositions>,
}

impl<'application, 'principal, 'scope, 'requirements, Schema>
    WorthQueryApplicationProgramAdoptionRequest<
        'application,
        'principal,
        'scope,
        'requirements,
        Schema,
    >
where
    Schema: ApplicationSchema,
{
    pub fn migration(mut self, migration: WorthQueryPreparedProgramMigration) -> Self {
        self.migration = Some(migration);
        self
    }

    /// Explicit adoption intent, resolved against complete private fresh safety truth.
    pub fn workflow(mut self, dispositions: WorthQueryWorkflowDispositions) -> Self {
        self.workflow = Some(dispositions);
        self
    }

    pub fn prepare(
        self,
        maximum_selection_work: usize,
    ) -> Result<
        WorthQueryPreparedBranchAdoption,
        WorthQueryApplicationProgramAdoptionPreparationDenial,
    > {
        let request_scope = self.programs.scope.clone();
        let runtime = self.programs.application;
        runtime.with_application_advancement(&request_scope, |phase| {

        let selected = self.programs
            .application
            .on_branch(self.programs.branch)
            .select(&phase, )
            .map_err(WorthQueryApplicationProgramAdoptionPreparationDenial::ProductSelection)?;
        selected.prepare_branch_adoption_in_advancement(
                &phase,
                self.requirements.target(),
                self.requirements,
                self.migration,
                self.workflow,
                maximum_selection_work,
                self.programs.principal,
                self.programs.scope,
            )
            .map_err(WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption)

        }).map_err(|cause| WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(worth_query_execution::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial::ExecutionDenied(cause)))?
    }
}
