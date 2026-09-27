//! Asking one occurrence to set a dimension.
//!
//! The request never names a program: the same request, value and host reach
//! a different answer depending on which program the occurrence runs.

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
    WorthQueryApplicationRequestMutationDenial,
};
use worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_host::facade::declaration::application_program::ApplicationProgramDefinition;
use worth_query_host::facade::product::WorthQueryProductBranch;

use super::dimension_entry::{
    PartDimensionWritten, SetPartDimensionDenial, SetPartDimensionIntent, PART_IDENTITY,
};
use super::operator_identity::{authenticate_operator, request_scope};
use super::schema::{BoundedDimensionSchema, SetPartDimensionInput};

/// What one set-dimension request settled as.
pub type DimensionOutcome =
    WorthQueryApplicationMutationOutcome<SetPartDimensionDenial, PartDimensionWritten>;

/// Issues one set-dimension request on one occurrence through the installed
/// owner of the program that occurrence runs.
pub fn set_dimension<Program>(
    application: &WorthQueryProgramApplicationRuntime<BoundedDimensionSchema, Program>,
    branch: WorthQueryProductBranch,
    dimension: u64,
    idempotency: u64,
) -> Result<DimensionOutcome, WorthQueryApplicationRequestMutationDenial>
where
    Program: ApplicationProgramDefinition<BoundedDimensionSchema>,
{
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(SetPartDimensionIntent {
            input: SetPartDimensionInput {
                identity: PART_IDENTITY.to_owned(),
                dimension,
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .execute_in_program(application)
}
