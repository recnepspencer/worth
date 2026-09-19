//! Asking one occurrence to set a dimension while presenting one program.
//!
//! The presented program is the whole point: the same request, the same
//! occurrence and the same value reach a different answer depending on which
//! rostered program the caller presents and which one the occurrence runs.

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
    WorthQueryApplicationRequestMutationDenial,
};
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;
use worth_query_host::facade::product::WorthQueryProductBranch;

use super::dimension_entry::{
    PartDimensionWritten, SetPartDimensionDenial, SetPartDimensionIntent, PART_IDENTITY,
};
use super::operator_identity::{authenticate_operator, request_scope};
use super::schema::{BoundedDimensionSchema, SetPartDimensionInput};

/// What one presented set-dimension request settled as.
pub type DimensionOutcome =
    WorthQueryApplicationMutationOutcome<SetPartDimensionDenial, PartDimensionWritten>;

/// Issues one set-dimension request on one occurrence through one presented
/// program owner.
pub fn set_dimension<Owner>(
    owner: &Owner,
    branch: WorthQueryProductBranch,
    dimension: u64,
    idempotency: u64,
) -> Result<DimensionOutcome, WorthQueryApplicationRequestMutationDenial>
where
    Owner: WorthQueryProgramOwner<BoundedDimensionSchema>,
{
    let runtime = owner.runtime();
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
        .execute_in_program(owner)
}
