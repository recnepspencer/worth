//! Actual canonical authored history, independently of semantic field revisions.
use crate::facade::{
    branch::AdmittedRelationalBranchBasis, history::*, identity::EntityId,
    runtime::RelationalRuntime, transactions::*,
};
use crate::tests::support::*;
use std::num::NonZeroUsize;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectFieldLocator, CanonicalFieldPath, LocatorAuthority};

mod adjacency;
mod aspect_presence;
mod branches;
mod custody;
mod field_writes;
mod indexed;

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

fn selected(
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
) -> crate::change_source::RelationalSelectedCommit {
    runtime
        .select_exact_commit(
            &basis.observation(),
            basis.observation().commit_id().unwrap(),
        )
        .into_outcome()
        .unwrap()
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

fn update_batch(entity: EntityId, value: &str) -> WorkerIntentBatch {
    WorkerIntentBatch::new("authored-update").push(MutationIntent::Entity(
        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id: entity,
            fields: single_string_aspect_field_patch(aspect_key("name"), field_key("name"), value),
        }),
    ))
}

fn name_locator() -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        aspect_key("name"),
        CanonicalFieldPath::single(field_key("name")),
    )
}
