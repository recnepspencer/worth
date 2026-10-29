//! Recovery stages cold admissions without changing the receiving runtime.
use super::cold_read_matrix::FAILURES;
use super::cold_source::ColdRead;
use super::*;
use crate::indexes::data::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
};
use crate::storage::data::{RelationalStorageSubject as Subject, RelationalStorageUnavailable};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug)]
enum Input {
    Head,
    History,
    Symbols,
    Buckets,
    Nodes,
    Events,
    Index,
    Envelope,
    TailEnvelope,
}
impl Input {
    const ALL: [Self; 9] = [
        Self::Head,
        Self::History,
        Self::Symbols,
        Self::Buckets,
        Self::Nodes,
        Self::Events,
        Self::Index,
        Self::Envelope,
        Self::TailEnvelope,
    ];
}
struct Fixture {
    recovery: RelationalDurableRecoverySource,
    source: Arc<MemberSource>,
    manifest: crate::durability::data::RelationalCheckpointManifest,
    entity: crate::identity::data::EntityId,
    symbol: crate::symbols::data::Symbol,
    index: DerivedIndexId,
}
fn fixture(execution: &worth_execution::ExecutionRequest<'_, '_>) -> Fixture {
    use super::cold_symbols::view_levels::{admit_population, publish_text, TextMutation};
    let producer = durable_runtime_with_test_schema();
    admit_population(execution, &producer);
    let (entity, _) = publish_text(
        execution,
        &producer,
        TextMutation::Create,
        "paged-view-value-0300",
    );
    let (_, _) = publish_text(
        execution,
        &producer,
        TextMutation::Update(entity),
        "paged-view-value-0600",
    );
    let symbol = producer
        .services
        .symbols
        .interner_snapshot()
        .symbol("paged-view-value-0600")
        .unwrap();
    let index = producer
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "tail-matrix-name".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
        })
        .expect("index fixture has identity capacity")
        .index_id;
    let branch = crate::history::data::BranchId("main".into());
    let built = producer.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: producer.history().branch_head(&branch).unwrap().commit_id,
            branch_id: branch,
            index_ids: vec![index],
        },
    );
    assert!(built.failed_indexes.is_empty());
    create_entity(execution, &producer, "tail-matrix-unrelated-head");
    let (recovery, source, entity) = checkpoint_fixture(execution, producer, entity, true);
    source.damaged.store(false, Ordering::Relaxed);
    #[derive(serde::Deserialize)]
    struct Image {
        manifest: crate::durability::data::RelationalCheckpointManifest,
    }
    let manifest = rmp_serde::from_slice::<Image>(
        recovery
            .checkpoint()
            .unwrap()
            .checkpoint()
            .bytes()
            .strip_prefix(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC)
            .unwrap(),
    )
    .unwrap()
    .manifest;
    Fixture {
        recovery,
        source,
        manifest,
        entity,
        symbol,
        index,
    }
}
fn recover(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut crate::runtime::RelationalRuntime,
    recovery: RelationalDurableRecoverySource,
) -> Result<(), RelationalRecoveryStop> {
    let plan = runtime.durability().recovery_plan_from_source(
        execution,
        recovery,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )?;
    runtime
        .durability_recovery()
        .recover(execution, plan)
        .map(|_| ())
}

#[cfg(test)]
#[path = "cold_tail_matrix/exhaustive.rs"]
mod exhaustive;
