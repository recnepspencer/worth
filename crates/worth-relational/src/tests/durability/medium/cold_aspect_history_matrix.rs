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

#[cfg(test)]
#[path = "cold_aspect_history_matrix/exhaustive.rs"]
mod exhaustive;
