//! Genuine public producer/source recipes for original-input certification courts.
use crate::{durable_medium::TestDurableMedium, durable_publication};
use std::sync::Arc;
use worth_execution::{ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_relational::facade::{
    durability::*, runtime::RelationalRuntime, schema::RelationalSchemaRegistry,
};
mod schema_history;
pub use schema_history::schema_transition_history;

pub fn additions() -> RelationalSchemaRegistry {
    use worth_relational::facade::{config::*, schema::*};
    let unused = RelationKindRegistration {
        kind_id: worth_relational::facade::identity::KindId(901),
        kind_name: "home-original-input".into(),
        schema_id: SchemaId("test".into()),
        schema_version_id: SchemaVersionId(1),
        aspect_contract_declarations: Default::default(),
        relation_integrity: Default::default(),
        cross_context_policy: CrossContextPolicy::AllowExplicit,
        cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
    };
    RelationalSchemaRegistry::new()
        .register_relation_kind(unused)
        .unwrap()
}
pub fn initialized(
    request: &ExecutionRequest<'_, '_>,
) -> (RelationalRuntime, Arc<TestDurableMedium>) {
    let medium = TestDurableMedium::new();
    let mut runtime = durable_publication::durable_runtime(&medium);
    let permission = runtime.begin_fresh_home_initialization(request).unwrap();
    runtime
        .prepare_fresh_home_schema_installation(request, permission)
        .unwrap()
        .install_with_managed_sources(request, additions(), vec![], vec![])
        .unwrap();
    (runtime, medium)
}
pub fn receiver(medium: &Arc<TestDurableMedium>) -> RelationalRuntime {
    let mut runtime = durable_publication::durable_runtime(medium);
    runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(additions())
        .unwrap();
    runtime
}
pub fn capture(
    runtime: &RelationalRuntime,
    request: &ExecutionRequest<'_, '_>,
) -> CompleteRelationalNativeCheckpoint {
    runtime
        .complete_checkpoint_service()
        .prepare(request, RelationalCheckpointArtifactPolicy::Embedded)
        .unwrap()
}
pub fn home_input(
    medium: &Arc<TestDurableMedium>,
    artifact: RelationalCompleteSourceArtifact,
    request: &ExecutionRequest<'_, '_>,
) -> Result<RelationalDurableRecoverySource, RelationalInitialInputsRefusal> {
    let covered = artifact.covered();
    let through = artifact.covered_transitions();
    let source: Arc<dyn RelationalDurableColdSource> = medium.clone();
    let cut = RelationalDurableReadProtection::capture(&source, request).unwrap();
    let bounds = cut.bounds();
    let mut canonical =
        ExecutionRequestArrayBuilder::allocate(*request, (bounds.canonical.0 - covered.0) as usize)
            .unwrap();
    for position in covered.0 + 1..=bounds.canonical.0 {
        canonical
            .push(
                cut.read_commit_owned_at(
                    &source,
                    request,
                    worth_relational::facade::publication::PatchStreamPosition(position),
                    1 << 30,
                )
                .unwrap(),
            )
            .unwrap();
    }
    let mut transitions = ExecutionRequestArrayBuilder::allocate(
        *request,
        (bounds.transitions.0 - through.0) as usize,
    )
    .unwrap();
    for sequence in through.0 + 1..=bounds.transitions.0 {
        transitions
            .push(
                cut.read_transition_owned_at(
                    &source,
                    request,
                    worth_relational::facade::branch::RelationalBranchTransitionSequence(sequence),
                    1 << 30,
                )
                .unwrap(),
            )
            .unwrap();
    }
    let initial = cut
        .read_initial_inputs_owned(&source, request, 1 << 30)
        .unwrap();
    RelationalDurableRecoverySource::from_home_original_bytes(
        Some(RelationalDurableCheckpointSource::from_complete_artifact(
            artifact,
        )?),
        initial,
        canonical.seal().unwrap(),
        transitions.seal().unwrap(),
        bounds,
        source,
        request,
    )
}
