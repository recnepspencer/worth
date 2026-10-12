use crate::facade::history::{BranchId, RelationalCommitReceipt};
use crate::facade::indexes::{
    DerivedIndexBuildOutcome, DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId,
    DerivedIndexKind,
};
use crate::tests::support::{aspect_field_locator, aspect_key, field_key};

use super::super::fixture::FintechWorld;

pub(crate) fn register_case_book_index(world: &mut FintechWorld) -> DerivedIndexDefinition {
    world
        .runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "fintech.trade.book".to_string(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("book"), field_key("book")),
            },
            branch_scoped: true,
        })
}

pub(crate) fn build_branch_scoped_case_index(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut FintechWorld,
    index_id: DerivedIndexId,
    branch_id: BranchId,
    source_commit: RelationalCommitReceipt,
) -> DerivedIndexBuildOutcome {
    world.runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: source_commit.commit_id,
            branch_id,
            index_ids: vec![index_id],
        },
    )
}
