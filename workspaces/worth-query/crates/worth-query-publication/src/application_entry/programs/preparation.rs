use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryProgramAdoptionRequirements};

use super::WorthQueryApplicationProgramsRequest;

pub use worth_query_execution::facade::primary_graph::WorthQueryPreparedBranchAdoption;

#[derive(Debug)]
pub enum WorthQueryApplicationProgramAdoptionPreparationDenial {
    ProductSelection(
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial,
    ),
    Adoption(
        worth_query_execution::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial,
    ),
}

/// Adoption with owner-computed requirements bound, but no effects performed.
/// Only `prepare` can mint the move-only value whose `publish` method exists.
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
pub struct WorthQueryApplicationProgramAdoptionRequestWithRequirements<
    'application,
    'principal,
    'scope,
    'target,
    'requirements,
    Schema,
> {
    pub(super) programs:
        WorthQueryApplicationProgramsRequest<'application, 'principal, 'scope, Schema>,
    pub(super) target: &'target ApplicationProgramRevision,
    pub(super) requirements: &'requirements WorthQueryProgramAdoptionRequirements,
}

impl<'application, 'principal, 'scope, 'target, 'requirements, Schema>
    WorthQueryApplicationProgramAdoptionRequestWithRequirements<
        'application,
        'principal,
        'scope,
        'target,
        'requirements,
        Schema,
    >
where
    Schema: ApplicationSchema,
{
    pub fn prepare(
        self,
        maximum_selection_work: usize,
    ) -> Result<
        WorthQueryPreparedBranchAdoption,
        WorthQueryApplicationProgramAdoptionPreparationDenial,
    > {
        self.programs
            .application
            .on_branch(self.programs.branch)
            .select()
            .map_err(WorthQueryApplicationProgramAdoptionPreparationDenial::ProductSelection)?
            .prepare_branch_adoption(
                self.target,
                self.requirements,
                maximum_selection_work,
                self.programs.scope,
            )
            .map_err(WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption)
    }
}
