//! Actual published generation, optional older values, and independent cold readers.
use super::{
    current_publications::create_entity,
    current_read_basis::main_selected_version,
    immutable_recovery_input::copy_input,
    selected_directory::{self, Directory},
    symbol_text_publications::*,
};
use std::sync::{atomic::Ordering, Arc};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::*, history::BranchId, identity::*, indexes::*, runtime::RelationalRuntime,
    symbols::Symbol,
};
use worth_testkit_relational::{
    aspect_storage::{aspect_field_locator, aspect_key, field_key},
    cold_recovery::{capture_source, update_entity, ColdRecoverySourceFixture},
    durable_medium::{immutable_members::ImmutableCheckpointMemberSource, TestDurableMedium},
    durable_publication::durable_runtime,
};
pub(super) struct IndexRecipe {
    original: ColdRecoverySourceFixture,
    pub entity: EntityId,
    pub index: DerivedIndexId,
    pub generation: Arc<DerivedIndexGeneration>,
    /// A second index's generation carried unchanged past the last edit.
    pub carried: Option<Arc<DerivedIndexGeneration>>,
    pub history_symbol: Option<Symbol>,
}
pub(super) struct IndexFixture {
    pub runtime: RelationalRuntime,
    pub source: Arc<ImmutableCheckpointMemberSource>,
    pub medium: Arc<TestDurableMedium>,
    pub entity: EntityId,
    pub index: DerivedIndexId,
    pub generation: Arc<DerivedIndexGeneration>,
    pub carried: Option<Arc<DerivedIndexGeneration>>,
    pub member: RelationalDurableMemberLocator,
    pub directory: Directory,
    pub history_symbol: Option<Symbol>,
}
impl IndexRecipe {
    pub fn new(request: &ExecutionRequest<'_, '_>, historical: bool) -> Self {
        Self::shape(request, historical, false)
    }
    /// The name index advances past a lifecycle index whose selection is carried.
    pub fn carried(request: &ExecutionRequest<'_, '_>) -> Self {
        Self::shape(request, false, true)
    }
    fn shape(request: &ExecutionRequest<'_, '_>, historical: bool, carried: bool) -> Self {
        let medium = TestDurableMedium::new();
        let producer = durable_runtime(&medium);
        let index = producer
            .index_authority()
            .register(DerivedIndexDefinition {
                index_id: DerivedIndexId(0),
                name: "matrix-name".into(),
                branch_scoped: true,
                kind: DerivedIndexKind::EntityField {
                    field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
                },
            })
            .unwrap()
            .index_id;
        let stable_index = carried.then(|| {
            producer
                .index_authority()
                .register(DerivedIndexDefinition {
                    index_id: DerivedIndexId(0),
                    name: "matrix-lifecycle".into(),
                    branch_scoped: true,
                    kind: DerivedIndexKind::EntityField {
                        field_locator: aspect_field_locator(
                            aspect_key("lifecycle"),
                            field_key("lifecycle"),
                        ),
                    },
                })
                .unwrap()
                .index_id
        });
        let (entity, history_symbol, _retained) = if historical {
            admit_population(request, &producer);
            let (entity, before, symbol) = publish_text(
                request,
                &producer,
                TextMutation::Create,
                "paged-view-value-0300",
            );
            let (_, after, _) = publish_text(
                request,
                &producer,
                TextMutation::Update(entity),
                "paged-view-value-0600",
            );
            (entity, Some(symbol), Some((before, after)))
        } else {
            (
                create_entity(request, &producer, "matrix-index-value"),
                None,
                None,
            )
        };
        let branch = BranchId("main".into());
        let (_, basis) = producer
            .observe_branch(&producer.main_branch_identity())
            .unwrap();
        let commit = basis.observation().commit_id().unwrap();
        drop(basis);
        let built = producer.index_authority().build_for_commit(
            request,
            DerivedIndexBuildRequest {
                source_commit_id: commit,
                branch_id: branch.clone(),
                index_ids: std::iter::once(index).chain(stable_index).collect(),
            },
        );
        assert!(
            built.failed_indexes.is_empty(),
            "actual producer builds index"
        );
        let carried = stable_index.map(|stable| {
            producer
                .index_access()
                .read_latest_generation(request, stable, &branch)
                .unwrap()
                .unwrap()
        });
        if carried.is_some() {
            update_entity(request, &producer, entity, "matrix-index-updated");
        }
        let generation = producer
            .index_access()
            .read_latest_generation(request, index, &branch)
            .unwrap()
            .unwrap();
        if let Some(carried) = &carried {
            assert!(carried.applicability.version_id < generation.applicability.version_id);
        } else {
            assert_eq!(generation.source_commit_id, commit);
        }
        if historical {
            update_entity(request, &producer, entity, "matrix-index-after");
        }
        let original = capture_source(request, producer, medium, entity, false);
        let directory = selected_directory::read(&original.input);
        assert!(directory
            .index_definitions
            .iter()
            .any(|definition| definition.index_id == index));
        Self {
            original,
            entity,
            index,
            generation,
            carried,
            history_symbol,
        }
    }
    pub fn reopen(&self, request: &ExecutionRequest<'_, '_>) -> IndexFixture {
        self.reopen_with_source(request, |source| source)
    }
    pub fn reopen_with_source(
        &self,
        request: &ExecutionRequest<'_, '_>,
        wrap: impl FnOnce(Arc<ImmutableCheckpointMemberSource>) -> Arc<dyn RelationalDurableColdSource>,
    ) -> IndexFixture {
        let (input, source) = copy_input(&self.original);
        let directory = selected_directory::read(&input);
        let input = RelationalDurableRecoverySource::new(
            input.checkpoint().cloned(),
            input.tail().to_vec(),
            input.transitions().to_vec(),
            wrap(source.clone()),
        );
        let medium = TestDurableMedium::new();
        let attachment = medium.attachment();
        for bytes in source.commits.values() {
            let artifact =
                RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
            assert!(matches!(
                attachment.publish(&artifact),
                DurablePublicationFate::Committed
            ));
        }
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
        assert_eq!(source.reads.load(Ordering::Relaxed), 0);
        assert!(self.generation.applicability.version_id <= main_selected_version(&runtime));
        IndexFixture {
            runtime,
            source,
            medium,
            entity: self.entity,
            index: self.index,
            generation: self.generation.clone(),
            carried: self.carried.clone(),
            member: directory.derived_indexes,
            directory,
            history_symbol: self.history_symbol,
        }
    }
}
