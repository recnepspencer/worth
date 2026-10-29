//! Historical builds must filter a genuine older version and admit its Full source.
use super::cold_index_fixture::{reopened_historical, IndexFixture};
use super::*;
use crate::execution::RelationalExecutionDenialCause as Execution;
use crate::indexes::data::DerivedIndexBuildRequest;

fn build(
    fixture: &IndexFixture,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), Execution> {
    let expected = &fixture.generation;
    assert!(expected.applicability.version_id < fixture.runtime.current_version_id());
    let result = fixture.runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: expected.source_commit_id,
            branch_id: expected.source_branch_id.clone(),
            index_ids: vec![fixture.index],
        },
    );
    if let Some(denial) = result.execution_denial {
        let crate::indexes::data::DerivedIndexExecutionDenialKind::Cause(cause) = denial.kind;
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
fn assert_cold(fixture: &IndexFixture) {
    let symbol = fixture.history_symbol.unwrap();
    assert!(
        fixture
            .runtime
            .services
            .symbols
            .interner_snapshot()
            .resolve(symbol)
            .is_none(),
        "the historical-only meaning is cold at reopen"
    );
    assert!(
        fixture
            .runtime
            .history
            .canonical_envelope(fixture.generation.source_commit_id)
            .is_none(),
        "the selected nonhead envelope body is cold"
    );
    assert_eq!(fixture.source.reads.load(Ordering::Relaxed), 0);
}
#[test]
fn historical_index_reconstruction_uses_the_old_value_and_full_cold_history() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = reopened_historical(execution);
    assert_cold(&fixture);
    let journal = fixture.runtime.history().latest_patch_stream_position();
    let offered = attached_test_medium(&fixture.runtime).offered().len();
    build(&fixture, execution).unwrap();
    assert_eq!(
        fixture
            .runtime
            .services
            .symbols
            .interner_snapshot()
            .resolve(fixture.history_symbol.unwrap()),
        Some("paged-view-value-0300")
    );
    assert_eq!(
        fixture.runtime.history().latest_patch_stream_position(),
        journal
    );
    assert_eq!(
        attached_test_medium(&fixture.runtime).offered().len(),
        offered
    );
}

#[derive(Clone, Copy, Debug)]
enum Subject {
    Head,
    History,
    HistorySymbols,
    Envelope,
    Index,
}
impl Subject {
    const ALL: [Self; 5] = [
        Self::Head,
        Self::History,
        Self::HistorySymbols,
        Self::Envelope,
        Self::Index,
    ];
    fn target(
        self,
        fixture: &IndexFixture,
    ) -> (
        super::cold_source::ColdRead,
        crate::storage::data::RelationalStorageSubject,
    ) {
        use super::cold_source::ColdRead;
        use crate::storage::data::RelationalStorageSubject as Storage;
        match self {
            Self::Envelope => {
                let entry = fixture
                    .manifest
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
                    .manifest
                    .symbols
                    .iter()
                    .find(|page| {
                        page.first <= u64::from(symbol.0) && u64::from(symbol.0) <= page.last
                    })
                    .unwrap();
                (
                    ColdRead::Member(page.member),
                    Storage::Symbols {
                        first: crate::symbols::data::Symbol(page.first.try_into().unwrap()),
                        last: crate::symbols::data::Symbol(page.last.try_into().unwrap()),
                    },
                )
            }
            Self::Head | Self::History => {
                let view = fixture
                    .runtime
                    .read_truth()
                    .try_project_historical_version(fixture.generation.applicability.version_id)
                    .unwrap();
                let root_commit = view.selected_root().unwrap().commit_id().unwrap();
                let root = fixture
                    .manifest
                    .roots
                    .iter()
                    .find(|root| root.commit_id == root_commit)
                    .unwrap();
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
fn target_reads(fixture: &IndexFixture, target: super::cold_source::ColdRead) -> usize {
    match target {
        super::cold_source::ColdRead::Member(target) => fixture
            .source
            .member_reads
            .lock()
            .unwrap()
            .iter()
            .filter(|member| **member == target)
            .count(),
        super::cold_source::ColdRead::Commit(target) => fixture
            .source
            .commit_reads
            .lock()
            .unwrap()
            .iter()
            .filter(|position| **position == target)
            .count(),
    }
}

#[cfg(test)]
#[path = "cold_historical_index_matrix/exhaustive.rs"]
mod exhaustive;
