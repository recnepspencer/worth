//! Public projection scans must guard their exact or historical source themselves.
use super::*;
use crate::execution::RelationalExecutionDenialCause;
use crate::visibility::materialization::read_records::ProjectionAspectScope;
#[derive(Clone, Copy, Debug)]
pub(super) enum ProjectionScanDoor {
    EntityKind,
    RelationKind,
    EntityRecords,
    RelationRecords,
}
impl ProjectionScanDoor {
    pub(super) fn run(
        self,
        historical: bool,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::runtime::RelationalRuntime,
    ) -> Result<(), RelationalExecutionDenialCause> {
        let view = if historical {
            runtime
                .read_truth()
                .try_project_historical_version(runtime.current_version_id())
                .unwrap()
        } else {
            let (_, basis) = runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap();
            runtime
                .read_truth()
                .project_observation(&basis.observation())
                .unwrap()
        };
        match self {
            Self::EntityKind => view
                .bounded_entities_of_kind(execution, KindId(1), 1_000_000)
                .map(|answer| {
                    assert!(
                        answer.is_ok(),
                        "the actual small scan fits its declared bound"
                    );
                }),
            Self::RelationKind => view
                .bounded_relations_of_kind(execution, KindId(2), 1_000_000)
                .map(|answer| {
                    assert!(
                        answer.is_ok(),
                        "the actual small scan fits its declared bound"
                    );
                }),
            Self::EntityRecords => view
                .entity_records_with_projection_scope(
                    execution,
                    KindId(1),
                    ProjectionAspectScope::empty(),
                    |_| Some(true),
                )
                .map(|records| {
                    assert!(!records.is_empty(), "the producer has a live entity");
                }),
            Self::RelationRecords => view
                .relation_records_with_projection_scope(
                    execution,
                    KindId(2),
                    ProjectionAspectScope::empty(),
                    |_| Some(true),
                )
                .map(|_| ()),
        }
    }
}
