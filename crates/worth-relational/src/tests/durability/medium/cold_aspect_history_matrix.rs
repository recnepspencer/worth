//! Historical operations demand the exact failed subject without partial answers.
use super::*;
use crate::execution::RelationalExecutionDenialCause as Execution;
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailable};
#[derive(Debug, Clone, Copy)]
enum Door {
    Entity,
    EntityTrace,
    Relation,
    RelationTrace,
    Lineage,
    LineageTrace,
}
impl Door {
    const ALL: [Self; 6] = [
        Self::Entity,
        Self::EntityTrace,
        Self::Relation,
        Self::RelationTrace,
        Self::Lineage,
        Self::LineageTrace,
    ];
    fn run(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        fixture: &CurrentFixture,
    ) -> Result<usize, Execution> {
        let history = fixture.runtime.history();
        let branch = BranchId("main".into());
        match self {
            Self::Entity => history
                .entity_aspect_history(execution, &branch, fixture.entity, None)
                .map(|history| history.len()),
            Self::EntityTrace => history
                .entity_aspect_history_with_trace(execution, &branch, fixture.entity, None)
                .map(|history| history.entries.len()),
            Self::Relation => history
                .relation_aspect_history(execution, &branch, fixture.relation, None)
                .map(|history| history.len()),
            Self::RelationTrace => history
                .relation_aspect_history_with_trace(execution, &branch, fixture.relation, None)
                .map(|history| history.entries.len()),
            Self::Lineage => history
                .lineage_entity_aspect_history(execution, &branch, fixture.lineage, None)
                .map(|history| {
                    history
                        .expect("producer lineage has real aspect history")
                        .entries
                        .len()
                }),
            Self::LineageTrace => history
                .lineage_entity_aspect_history_with_trace(execution, &branch, fixture.lineage, None)
                .map(|history| {
                    history
                        .history
                        .expect("producer lineage has real aspect history")
                        .entries
                        .len()
                }),
        }
    }
}
#[derive(Debug, Clone, Copy)]
enum Subject {
    Head,
    History,
    Symbols,
    Envelope,
}
impl Subject {
    const ALL: [Self; 4] = [Self::Head, Self::History, Self::Symbols, Self::Envelope];
    fn targets(self, fixture: &CurrentFixture) -> Vec<ColdRead> {
        let members = match self {
            Self::Head => &fixture.heads,
            Self::History => &fixture.history,
            Self::Symbols => &fixture.symbols,
            Self::Envelope => {
                return vec![ColdRead::Commit(
                    *fixture.source.commits.keys().next().unwrap(),
                )]
            }
        };
        members
            .iter()
            .map(|member| ColdRead::Member(*member))
            .collect()
    }
    fn assert_subject(self, fixture: &CurrentFixture, failure: RelationalStorageUnavailable) {
        match self {
            Self::Head | Self::History => assert_eq!(
                failure.subject,
                RelationalStorageSubject::Region(fixture.entity.partition_id)
            ),
            Self::Symbols => assert!(matches!(
                failure.subject,
                RelationalStorageSubject::Symbols { .. }
            )),
            Self::Envelope => assert_eq!(
                failure.subject,
                RelationalStorageSubject::Envelope(*fixture.source.commits.keys().next().unwrap())
            ),
        }
    }
}
fn attempts(fixture: &CurrentFixture, targets: &[ColdRead]) -> usize {
    fixture
        .source
        .member_reads
        .lock()
        .unwrap()
        .iter()
        .filter(|member| targets.contains(&ColdRead::Member(**member)))
        .count()
        + fixture
            .source
            .commit_reads
            .lock()
            .unwrap()
            .iter()
            .filter(|position| targets.contains(&ColdRead::Commit(**position)))
            .count()
}
#[test]
fn residency_failure_matrix_aspect_history_doors_cover_every_subject_and_class() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for subject in Subject::ALL {
        for door in Door::ALL {
            for (cause, sticky) in FAILURES {
                let fixture = CurrentFixture::reopened(execution);
                let targets = subject.targets(&fixture);
                assert!(!targets.is_empty());
                let frontier = fixture.runtime.history().latest_patch_stream_position();
                let offered = attached_test_medium(&fixture.runtime).offered().len();
                let diagnostics = fixture.runtime.publication.diagnostics.snapshot();
                fixture
                    .source
                    .failures
                    .lock()
                    .unwrap()
                    .extend(targets.iter().map(|target| (*target, cause)));
                let denial = door.run(execution, &fixture).unwrap_err();
                let Execution::StorageUnavailable(failure) = denial else {
                    panic!("{subject:?} x {door:?} x {cause:?}: {denial:?}");
                };
                assert_eq!(failure.cause, cause);
                subject.assert_subject(&fixture, failure);
                fixture.source.failures.lock().unwrap().clear();
                let before = attempts(&fixture, &targets);
                if sticky {
                    assert_eq!(door.run(execution, &fixture), Err(denial));
                    assert_eq!(
                        attempts(&fixture, &targets),
                        before,
                        "{subject:?} x {door:?}: only the failed subject must stay damaged"
                    );
                } else {
                    assert!(
                        door.run(execution, &fixture)
                            .unwrap_or_else(|failure| panic!(
                                "retry {subject:?} x {door:?} x {cause:?}: {failure:?}"
                            ))
                            > 0
                    );
                    assert!(attempts(&fixture, &targets) > before);
                }
                assert_eq!(
                    fixture.runtime.history().latest_patch_stream_position(),
                    frontier
                );
                assert_eq!(
                    attached_test_medium(&fixture.runtime).offered().len(),
                    offered
                );
                assert_eq!(
                    fixture.runtime.publication.diagnostics.snapshot(),
                    diagnostics
                );
            }
        }
    }
}
