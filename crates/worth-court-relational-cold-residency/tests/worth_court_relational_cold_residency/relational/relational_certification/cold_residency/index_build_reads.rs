//! The two public index construction orchestrators over real selected records.
use worth_relational::facade::indexes::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexEntries, DerivedIndexId,
    DerivedIndexKind,
};
use worth_relational::facade::transactions::RelationalExecutionDenialCause;
use worth_testkit_relational::aspect_storage::{aspect_field_locator, aspect_key, field_key};

#[derive(Clone, Copy, Debug)]
pub(super) enum IndexBuildDoor {
    Commit,
    Basis,
}
impl IndexBuildDoor {
    pub(super) fn register(
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> DerivedIndexId {
        runtime
            .index_authority()
            .register(Self::definition())
            .expect("index fixture has identity capacity")
            .index_id
    }
    pub(super) fn definition() -> DerivedIndexDefinition {
        DerivedIndexDefinition {
            index_id: DerivedIndexId(1001),
            name: "residency-read-matrix-build".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
        }
    }
    pub(super) fn read(
        self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        index: DerivedIndexId,
    ) -> Result<(), RelationalExecutionDenialCause> {
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let request = DerivedIndexBuildRequest {
            source_commit_id: basis.observation().commit_id().unwrap(),
            branch_id: worth_relational::facade::history::BranchId("main".into()),
            index_ids: vec![index],
        };
        let outcome = match self {
            Self::Commit => runtime
                .index_authority()
                .build_for_commit(execution, request),
            Self::Basis => runtime
                .index_authority()
                .build_for_basis(execution, request, &basis),
        };
        if let Some(cause) = outcome.execution_denial {
            let worth_relational::facade::indexes::DerivedIndexExecutionDenialKind::Cause(cause) =
                cause.kind;
            return Err(cause);
        }
        assert!(
            outcome.failed_indexes.is_empty(),
            "actual funded index build: {outcome:?}"
        );
        assert!(
            outcome
                .generations
                .iter()
                .any(|generation| generation.index_id == index
                    && match &generation.entries {
                        DerivedIndexEntries::EntityField(entries) => !entries.is_empty(),
                        DerivedIndexEntries::RelationField(_)
                        | DerivedIndexEntries::RelatedEntityOrdering(_)
                        | DerivedIndexEntries::RelationJoin(_) => false,
                    }),
            "the producer's declared field must yield actual rows"
        );
        Ok(())
    }
}
