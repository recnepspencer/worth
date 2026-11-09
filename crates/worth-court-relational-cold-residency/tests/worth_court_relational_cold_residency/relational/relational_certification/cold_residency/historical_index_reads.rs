//! Original older-source index rows and exact unconsulted envelope target selection.
use super::{
    current_read_basis::main_selected_version,
    index_source::{IndexFixture, IndexRecipe},
};
use std::sync::atomic::Ordering;
use worth_relational::facade::{
    indexes::DerivedIndexBuildRequest, transactions::RelationalExecutionDenialCause as Execution,
};
use worth_testkit_relational::durable_medium::immutable_members::ColdRead;
pub(super) fn build(
    fixture: &IndexFixture,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), Execution> {
    let expected = &fixture.generation;
    assert!(expected.applicability.version_id < main_selected_version(&fixture.runtime));
    let result = fixture.runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: expected.source_commit_id,
            branch_id: expected.source_branch_id.clone(),
            index_ids: vec![fixture.index],
        },
    );
    if let Some(denial) = result.execution_denial {
        let worth_relational::facade::indexes::DerivedIndexExecutionDenialKind::Cause(cause) =
            denial.kind;
        return Err(cause);
    }
    assert!(
        result.failed_indexes.is_empty(),
        "the real historical source must build: {result:?}"
    );
    let actual = result
        .generations
        .iter()
        .find(|generation| generation.index_id == fixture.index)
        .unwrap();
    assert_eq!(actual.source_commit_id, expected.source_commit_id);
    assert_eq!(
        actual.entries, expected.entries,
        "a later current value must never escape version filtering"
    );
    assert!(actual.generation_id > expected.generation_id);
    assert!(
        actual.generation_id.0 < expected.generation_id.0 + 16,
        "generation growth has headroom"
    );
    Ok(())
}
pub(super) fn assert_initial_no_reads(fixture: &IndexFixture) {
    assert_eq!(fixture.source.reads.load(Ordering::Relaxed), 0);
}
#[test]
fn historical_index_reconstruction_uses_the_old_value_and_full_cold_history() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = IndexRecipe::new(execution, true).reopen(execution);
    assert_initial_no_reads(&fixture);
    let journal = fixture.runtime.history().latest_patch_stream_position();
    let offered = fixture.medium.offered().len();
    build(&fixture, execution).unwrap();
    assert_eq!(
        fixture.runtime.history().latest_patch_stream_position(),
        journal
    );
    assert_eq!(fixture.medium.offered().len(), offered);
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Subject {
    Head,
    History,
    HistorySymbols,
    Envelope,
    Index,
}
impl Subject {
    pub(super) const ALL: [Self; 5] = [
        Self::Head,
        Self::History,
        Self::HistorySymbols,
        Self::Envelope,
        Self::Index,
    ];
    pub(super) fn target(
        self,
        fixture: &IndexFixture,
    ) -> (
        ColdRead,
        worth_relational::facade::storage::RelationalStorageSubject,
    ) {
        use worth_relational::facade::storage::RelationalStorageSubject as Storage;
        use ColdRead;
        match self {
            Self::Envelope => {
                let entry = fixture
                    .directory
                    .commits
                    .iter()
                    .find(|entry| entry.commit.commit_id == fixture.generation.source_commit_id)
                    .unwrap();
                (
                    ColdRead::Commit(entry.position),
                    Storage::Envelope(entry.position),
                )
            }
            Self::Index => (
                ColdRead::Member(fixture.member),
                Storage::Index(fixture.index),
            ),
            Self::HistorySymbols => {
                let symbol = fixture.history_symbol.unwrap();
                let page = fixture
                    .directory
                    .symbols
                    .iter()
                    .find(|page| {
                        page.first <= u64::from(symbol.0) && u64::from(symbol.0) <= page.last
                    })
                    .unwrap();
                (
                    ColdRead::Member(page.member),
                    Storage::Symbols {
                        first: worth_relational::facade::symbols::Symbol(
                            page.first.try_into().unwrap(),
                        ),
                        last: worth_relational::facade::symbols::Symbol(
                            page.last.try_into().unwrap(),
                        ),
                    },
                )
            }
            Self::Head | Self::History => {
                let view = fixture
                    .runtime
                    .read_truth()
                    .try_project_historical_version(fixture.generation.applicability.version_id)
                    .unwrap();
                assert_eq!(
                    view.version_id(),
                    fixture.generation.applicability.version_id
                );
                // This one-branch fixture carries one actual MVCC root. Older
                // versions read its retained interval; no guessed old root exists.
                let [root] = fixture.directory.roots.as_slice() else {
                    panic!("historical fixture must carry its single actual branch root");
                };
                let (_, basis) = fixture
                    .runtime
                    .observe_branch(&fixture.runtime.main_branch_identity())
                    .unwrap();
                assert_eq!(Some(root.commit_id), basis.observation().commit_id());
                drop(view);
                let region = root
                    .regions
                    .iter()
                    .find(|region| region.partition_id == fixture.entity.partition_id)
                    .unwrap();
                (
                    ColdRead::Member(if matches!(self, Self::Head) {
                        region.head
                    } else {
                        region.history
                    }),
                    Storage::Region(region.partition_id),
                )
            }
        }
    }
}
pub(super) fn target_reads(fixture: &IndexFixture, target: ColdRead) -> usize {
    match target {
        ColdRead::Member(target) => fixture
            .source
            .member_reads
            .lock()
            .unwrap()
            .iter()
            .filter(|member| **member == target)
            .count(),
        ColdRead::Commit(target) => fixture
            .source
            .commit_reads
            .lock()
            .unwrap()
            .iter()
            .filter(|position| **position == target)
            .count(),
    }
}
