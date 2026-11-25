//! Authored-write history through installed branch and read owners.
use std::num::NonZeroUsize;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectFieldLocator, CanonicalFieldPath, LocatorAuthority};
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    change_source::RelationalSelectedCommit,
    history::{
        CommitId, RelationalAuthoredWriteCursor, RelationalAuthoredWritePageLimits,
        RelationalAuthoredWriteScope, RelationalAuthoredWriteSelector,
    },
    identity::EntityId,
    runtime::RelationalRuntime,
    transactions::RecordRef,
};
use worth_testkit_relational::aspect_storage::{aspect_key, field_key};

mod adjacency;
mod aspect_presence;
mod branches;
mod custody;
mod field_writes;
mod indexed;
mod span_seed;

fn selected(
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
) -> RelationalSelectedCommit {
    runtime
        .select_exact_commit(
            &basis.observation(),
            basis.observation().commit_id().unwrap(),
        )
        .into_outcome()
        .unwrap()
}

fn limits(width: usize) -> RelationalAuthoredWritePageLimits {
    RelationalAuthoredWritePageLimits {
        maximum_entries: NonZeroUsize::new(width).unwrap(),
        maximum_source_visits: NonZeroUsize::new(width).unwrap(),
        maximum_owned_bytes: NonZeroUsize::new(usize::MAX).unwrap(),
    }
}

fn collect(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    mut cursor: RelationalAuthoredWriteCursor,
) -> (Vec<CommitId>, usize) {
    let mut commits = Vec::new();
    let mut visits = 0;
    loop {
        let page = cursor.next_page(runtime, execution, limits(2)).unwrap();
        commits.extend(page.entries().map(|entry| entry.commit_id()));
        visits += page.source_visits();
        if page.is_complete() {
            return (commits, visits);
        }
    }
}

fn keys(entity: EntityId) -> [RelationalAuthoredWriteSelector; 3] {
    use RelationalAuthoredWriteScope as Scope;
    [
        Scope::Lifecycle,
        Scope::WholeAspect(aspect_key("name")),
        Scope::Field(
            aspect_key("name"),
            CanonicalFieldPath::single(field_key("name")),
        ),
    ]
    .map(|scope| RelationalAuthoredWriteSelector::Record {
        record: RecordRef::Entity(entity),
        scope,
    })
}

fn cursor(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    original: &AdmittedRelationalBranchBasis,
    target: &AdmittedRelationalBranchBasis,
    entity: EntityId,
) -> RelationalAuthoredWriteCursor {
    runtime
        .authored_write_history_cursor(
            execution,
            original,
            selected(runtime, target),
            &keys(entity),
        )
        .unwrap()
}

fn name_locator() -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        aspect_key("name"),
        CanonicalFieldPath::single(field_key("name")),
    )
}

fn admit_main(
    runtime: &RelationalRuntime,
) -> Result<
    AdmittedRelationalBranchBasis,
    worth_relational::facade::branch::RelationalBranchBasisDenial,
> {
    runtime.admit_branch_basis(&runtime.main_branch_identity())
}
