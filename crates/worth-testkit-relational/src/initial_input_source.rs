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
/// Fresh external placement: this destination has no selected members yet.
pub fn capture_external(
    runtime: &RelationalRuntime,
    medium: &Arc<TestDurableMedium>,
    request: &ExecutionRequest<'_, '_>,
) -> CompleteRelationalNativeCheckpoint {
    let source: Arc<dyn RelationalDurableColdSource> = medium.clone();
    runtime
        .complete_checkpoint_service()
        .prepare(
            request,
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: Some(source),
            },
        )
        .unwrap()
}
/// Selected Home recipes supply the actual immutable membership they filed.
pub fn capture_selected_external(
    runtime: &RelationalRuntime,
    medium: &Arc<TestDurableMedium>,
    selected_members: &[RelationalDurableMemberLocator],
    request: &ExecutionRequest<'_, '_>,
) -> CompleteRelationalNativeCheckpoint {
    let source: Arc<dyn RelationalDurableColdSource> = medium.clone();
    runtime
        .complete_checkpoint_service()
        .prepare(
            request,
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: selected_members,
                source: Some(source),
            },
        )
        .unwrap()
}
pub fn home_input(
    medium: &Arc<TestDurableMedium>,
    artifact: RelationalCompleteSourceArtifact,
    request: &ExecutionRequest<'_, '_>,
) -> Result<RelationalDurableRecoverySource, RelationalInitialInputsRefusal> {
    home_checkpoint_input(
        medium,
        RelationalDurableCheckpointSource::from_complete_artifact(artifact)?,
        request,
    )
}

pub fn home_image_input(
    medium: &Arc<TestDurableMedium>,
    capture: &CompleteRelationalNativeCheckpoint,
    request: &ExecutionRequest<'_, '_>,
) -> Result<RelationalDurableRecoverySource, RelationalInitialInputsRefusal> {
    let source = home_image_checkpoint(capture, capture.source_artifact().clone(), request)?;
    home_checkpoint_input(medium, source, request)
}

pub fn home_image_checkpoint(
    capture: &CompleteRelationalNativeCheckpoint,
    artifact: RelationalCompleteSourceArtifact,
    request: &ExecutionRequest<'_, '_>,
) -> Result<RelationalDurableCheckpointSource, RelationalInitialInputsRefusal> {
    let image = capture
        .admission_image()
        .seal_home(
            worth_foundational::facade::HomeIdentity::from_observed_bytes([1; 16]).unwrap(),
            [0; 3],
            [2; 32],
            request,
        )
        .map_err(|cause| {
            RelationalInitialInputsRefusal::Native(RelationalRecoveryStop::AdmissionImage(cause))
        })?;
    let source = RelationalDurableCheckpointSource::from_checkpoint_admission_image(
        artifact, image, request,
    )
    .map_err(|cause| {
        RelationalInitialInputsRefusal::Native(RelationalRecoveryStop::AdmissionImage(cause))
    })?;
    Ok(source)
}

fn home_checkpoint_input(
    medium: &Arc<TestDurableMedium>,
    checkpoint: RelationalDurableCheckpointSource,
    request: &ExecutionRequest<'_, '_>,
) -> Result<RelationalDurableRecoverySource, RelationalInitialInputsRefusal> {
    let covered = checkpoint.covered();
    let through = checkpoint.covered_transitions();
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
        Some(checkpoint),
        initial,
        canonical.seal().unwrap(),
        transitions.seal().unwrap(),
        bounds,
        source,
        request,
    )
}
