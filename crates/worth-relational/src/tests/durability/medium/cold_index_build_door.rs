//! The two public index construction orchestrators over real selected records.
use super::*;
use crate::execution::RelationalExecutionDenialCause;
use crate::indexes::data::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexEntries, DerivedIndexId,
    DerivedIndexKind,
};

#[derive(Clone, Copy, Debug)]
pub(super) enum IndexBuildDoor {
    Commit,
    Basis,
}
impl IndexBuildDoor {
    pub(super) fn register(runtime: &crate::runtime::RelationalRuntime) -> DerivedIndexId {
        runtime
            .index_authority()
            .register(DerivedIndexDefinition {
                index_id: DerivedIndexId(1001),
                name: "residency-read-matrix-build".into(),
                branch_scoped: true,
                kind: DerivedIndexKind::EntityField {
                    field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
                },
            })
            .index_id
    }
    pub(super) fn read(
        self,
        runtime: &crate::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        index: DerivedIndexId,
    ) -> Result<(), RelationalExecutionDenialCause> {
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let request = DerivedIndexBuildRequest {
            source_commit_id: basis.observation().commit_id().unwrap(),
            branch_id: crate::history::data::BranchId("main".into()),
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
            let crate::indexes::data::DerivedIndexExecutionDenialKind::Cause(cause) = cause.kind;
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
                        _ => false,
                    }),
            "the producer's declared field must yield actual rows"
        );
        Ok(())
    }
}
