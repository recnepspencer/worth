//! Genuine old/current symbol pages shared physically, with independent reader meanings.
use super::{
    current_publications::create_entity, immutable_recovery_input::copy_input, selected_directory,
    symbol_text_publications::*,
};
use std::sync::{atomic::Ordering, Arc};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::*, identity::EntityId, runtime::RelationalRuntime, symbols::Symbol,
};
use worth_testkit_relational::{
    cold_recovery::{capture_source, ColdRecoverySourceFixture},
    durable_medium::{immutable_members::ImmutableCheckpointMemberSource, TestDurableMedium},
    durable_publication::durable_runtime,
};
pub(super) struct SymbolViewRecipe {
    original: ColdRecoverySourceFixture,
    pub entity: EntityId,
    pub historical: Symbol,
    pub current: Symbol,
    pub historical_page: RelationalDurableMemberLocator,
}
pub(super) struct SymbolViewFixture {
    pub runtime: RelationalRuntime,
    pub medium: Arc<TestDurableMedium>,
    pub source: Arc<ImmutableCheckpointMemberSource>,
    pub entity: EntityId,
    pub historical: Symbol,
    pub current: Symbol,
    pub historical_page: RelationalDurableMemberLocator,
}
impl SymbolViewRecipe {
    pub fn new(request: &ExecutionRequest<'_, '_>) -> Self {
        let medium = TestDurableMedium::new();
        let producer = durable_runtime(&medium);
        admit_population(request, &producer);
        let (entity, _before, historical) = publish_text(
            request,
            &producer,
            TextMutation::Create,
            "paged-view-value-0300",
        );
        let (_, _after, current) = publish_text(
            request,
            &producer,
            TextMutation::Update(entity),
            "paged-view-value-0600",
        );
        let extra = create_entity(request, &producer, "member-before");
        let original = capture_source(request, producer, medium, extra, false);
        let directory = selected_directory::read(&original.input);
        let page = |symbol: Symbol| {
            directory
                .symbols
                .iter()
                .find(|page| page.first <= u64::from(symbol.0) && u64::from(symbol.0) <= page.last)
                .unwrap()
                .member
        };
        let historical_page = page(historical);
        assert_ne!(
            historical_page,
            page(current),
            "fixture values occupy different actual pages"
        );
        Self {
            original,
            entity,
            historical,
            current,
            historical_page,
        }
    }
    pub fn reopen(&self, request: &ExecutionRequest<'_, '_>) -> SymbolViewFixture {
        let (input, source) = copy_input(&self.original);
        let medium = TestDurableMedium::new();
        let mut runtime = durable_runtime(&medium);
        let plan = runtime
            .durability()
            .recovery_plan_from_source(
                request,
                input,
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap();
        runtime
            .durability_recovery()
            .recover(request, plan)
            .unwrap();
        assert_eq!(
            source.reads.load(Ordering::Relaxed),
            0,
            "reopen reads no dictionary or region member"
        );
        let warmed = runtime.fork().unwrap();
        let (_, basis) = warmed
            .observe_branch(&warmed.main_branch_identity())
            .unwrap();
        assert!(warmed
            .entity_record_at_observation(request, &basis.observation(), self.entity)
            .unwrap()
            .is_some());
        drop(basis);
        drop(warmed);
        SymbolViewFixture {
            runtime,
            medium,
            source,
            entity: self.entity,
            historical: self.historical,
            current: self.current,
            historical_page: self.historical_page,
        }
    }
}
