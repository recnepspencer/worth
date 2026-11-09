//! A genuine source edit and Native graph selection over a model with fields.
use super::*;
use worth_foundational::facade::{AspectFieldLocator, CanonicalFieldPath, LocatorAuthority};
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    runtime::{
        RelationalFieldRevision, RelationalHistoricalSelectionRead,
        RelationalHistoricalSelectionReadDenial, VisibilityProjectionView,
    },
};
use worth_testkit_relational::aspect_storage::{
    aspect_key, entity_field_aspect, field_key, publications::string_aspect_field_patch,
    AspectSchemaFixture,
};

pub(super) struct SelectedWorld {
    pub runtime: RelationalRuntime,
    pub model: EntityId,
    pub outside: EntityId,
    /// The step navigation selects, before the genuine source edit.
    pub selected: AdmittedRelationalBranchBasis,
    /// The edited current state that navigation replaced.
    pub edited: AdmittedRelationalBranchBasis,
    /// The fresh Native graph-selection commit.
    pub restored: AdmittedRelationalBranchBasis,
}

pub(super) fn note() -> AspectFieldLocator {
    locator("note")
}
pub(super) fn other() -> AspectFieldLocator {
    locator("other")
}
fn locator(name: &str) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        aspect_key(name),
        CanonicalFieldPath::single(field_key(name)),
    )
}

pub(super) fn field_runtime() -> RelationalRuntime {
    let fixture = AspectSchemaFixture {
        entity_aspects: vec![
            entity_field_aspect(aspect_key("note"), field_key("note")),
            entity_field_aspect(aspect_key("other"), field_key("other")),
        ],
        ..AspectSchemaFixture::default()
    };
    let mut runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry_with(fixture))
        .build()
        .unwrap();
    runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install_with_custom_invariants(
            RelationalSchemaRegistry::new(),
            vec![ownership().invariant_registration().unwrap()],
        )
        .unwrap();
    runtime
}

fn patch(values: &[(&str, &str)]) -> AspectFieldPatch {
    string_aspect_field_patch(
        values
            .iter()
            .map(|(name, value)| (aspect_key(name), field_key(name), *value)),
    )
}

fn create(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
) -> EntityId {
    let mut tx = begin(runtime);
    tx.push_batch(
        WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Entity(
            EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw(key),
                fields: patch(&[("note", "first"), ("other", "unrelated")]),
            },
        ))),
        execution.allocation_policy(),
    )
    .unwrap();
    let committed = tx.commit(execution, runtime).unwrap();
    let id = committed
        .changed_records
        .iter()
        .find_map(|record| match record {
            RecordRef::Entity(id) => Some(*id),
            RecordRef::Relation(_) => None,
        })
        .unwrap();
    release_test_commit_snapshot(runtime, &committed);
    id
}

/// An ordinary authored field edit through the public transaction door.
pub(super) fn edit(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity: EntityId,
    name: &str,
    value: &str,
) {
    let mut tx = begin(runtime);
    tx.push_batch(
        WorkerIntentBatch::new("edit").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: patch(&[(name, value)]),
            }),
        )),
        execution.allocation_policy(),
    )
    .unwrap();
    let committed = tx.commit(execution, runtime).unwrap();
    release_test_commit_snapshot(runtime, &committed);
}

impl SelectedWorld {
    /// The model and an unowned outside entity are both edited before the
    /// model navigates back to the selected step.
    pub fn new(execution: &ExecutionRequest<'_, '_>) -> Self {
        let runtime = field_runtime();
        let model = create(execution, &runtime, "model");
        let owned = member(execution, &runtime, "owned-member");
        create_relation(execution, &runtime, model, owned, "owns-member");
        let outside = create(execution, &runtime, "outside");
        let selected = transfer_fixture::basis(&runtime);
        edit(execution, &runtime, model, "note", "edited");
        edit(execution, &runtime, outside, "note", "edited");
        let edited = transfer_fixture::basis(&runtime);
        let committed = transfer_fixture::select(execution, &runtime, model, &selected).unwrap();
        assert!(committed
            .changed_records
            .contains(&RecordRef::Entity(model)));
        assert!(!committed
            .changed_records
            .contains(&RecordRef::Entity(outside)));
        release_test_commit_snapshot(&runtime, &committed);
        let restored = transfer_fixture::basis(&runtime);
        Self {
            runtime,
            model,
            outside,
            selected,
            edited,
            restored,
        }
    }
    pub fn evidence(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalHistoricalSelectionRead, RelationalHistoricalSelectionReadDenial> {
        self.runtime.read_truth().read_historical_selection(
            execution,
            &self.selected.observation(),
            &self.restored.observation(),
        )
    }
    pub fn view(&self, basis: &AdmittedRelationalBranchBasis) -> VisibilityProjectionView {
        self.runtime
            .read_truth()
            .project_observation(&basis.observation())
            .unwrap()
    }
    pub fn current(&self) -> VisibilityProjectionView {
        self.view(&transfer_fixture::basis(&self.runtime))
    }
    pub fn revision(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        entity: EntityId,
        locator: &AspectFieldLocator,
    ) -> RelationalFieldRevision {
        self.view(basis)
            .entity_field_revision(execution, entity, locator)
            .unwrap()
            .unwrap()
    }
    pub fn aspect(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        entity: EntityId,
    ) -> Option<u64> {
        self.view(basis)
            .read_entity_aspect_version(execution, entity, &aspect_key("note"))
            .unwrap()
            .unwrap()
    }
}
