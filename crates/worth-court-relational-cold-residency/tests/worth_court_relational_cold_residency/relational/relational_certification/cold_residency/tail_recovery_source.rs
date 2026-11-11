//! A real selected checkpoint, older generation and later ordinary journal update.
use super::{
    current_publications::create_entity,
    immutable_recovery_input::copy_input,
    selected_directory::{self, Directory},
    symbol_text_publications::*,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::*, history::BranchId, identity::EntityId, indexes::*, runtime::RelationalRuntime,
    symbols::Symbol,
};
use worth_testkit_relational::{
    aspect_storage::{aspect_field_locator, aspect_key, field_key},
    cold_recovery::{capture_source, ColdRecoverySourceFixture},
    durable_medium::immutable_members::ImmutableCheckpointMemberSource,
    durable_medium::TestDurableMedium,
    durable_publication::durable_runtime,
};
pub(super) struct TailRecipe {
    original: ColdRecoverySourceFixture,
    entity: EntityId,
    symbol: Symbol,
    index: DerivedIndexId,
}
pub(super) struct TailFixture {
    pub recovery: RelationalDurableRecoverySource,
    pub source: Arc<ImmutableCheckpointMemberSource>,
    pub directory: Directory,
    pub entity: EntityId,
    pub symbol: Symbol,
    pub index: DerivedIndexId,
}
impl TailRecipe {
    pub fn new(request: &ExecutionRequest<'_, '_>) -> Self {
        let medium = TestDurableMedium::new();
        let producer = durable_runtime(&medium);
        admit_population(request, &producer);
        let (entity, _, _) = publish_text(
            request,
            &producer,
            TextMutation::Create,
            "paged-view-value-0300",
        );
        let (_, _, symbol) = publish_text(
            request,
            &producer,
            TextMutation::Update(entity),
            "paged-view-value-0600",
        );
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
            .unwrap()
            .index_id;
        let (_, basis) = producer
            .observe_branch(&producer.main_branch_identity())
            .unwrap();
        let built = producer.index_authority().build_for_commit(
            request,
            DerivedIndexBuildRequest {
                source_commit_id: basis.observation().commit_id().unwrap(),
                branch_id: BranchId("main".into()),
                index_ids: vec![index],
            },
        );
        assert!(built.failed_indexes.is_empty());
        drop(basis);
        create_entity(request, &producer, "tail-matrix-unrelated-head");
        let original = capture_source(request, producer, medium, entity, true);
        Self {
            original,
            entity,
            symbol,
            index,
        }
    }
    pub fn fixture(&self) -> TailFixture {
        let (recovery, source) = copy_input(&self.original);
        let directory = selected_directory::read(&recovery);
        TailFixture {
            recovery,
            source,
            directory,
            entity: self.entity,
            symbol: self.symbol,
            index: self.index,
        }
    }
}
pub(super) fn recover(
    request: &ExecutionRequest<'_, '_>,
    runtime: &mut RelationalRuntime,
    input: RelationalDurableRecoverySource,
) -> Result<(), RelationalRecoveryStop> {
    let plan = runtime.durability().recovery_plan_from_source(
        request,
        input,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )?;
    runtime
        .durability_recovery()
        .recover(request, plan)
        .map(|_| ())
}
