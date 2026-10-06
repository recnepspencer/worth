//! A merge's field updates obey the unique law. Application kinds declare
//! no merge policy yet, so this world is a raw kind whose unique `label`
//! merges last-writer-wins; the lookup reads any kind the same way, so the
//! update arm runs here exactly as it would for an application kind.

use std::collections::BTreeMap;

use worth_foundational::facade::{
    AspectContract, AspectContractRevision, AspectFieldLocator, AspectIdentity, AspectKey,
    AspectValue, CanonicalFieldPath, FieldKey, InternedString, LocatorAuthority, ScalarAspectType,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, KindId, PartitionId};
use worth_relational::facade::indexes::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
};
use worth_relational::facade::merge::{
    AspectMergePolicyDeclaration, AspectMergePolicyKind, IdentityBasisDeclaration,
    IdentityBasisKind, IdentityBasisScope, MergeExecutionRequest, MergeIntent,
    PreparedMergeExecution,
};
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_relational::facade::schema::{
    AspectBinding, DeclaredAspectContractBinding, EntityKindRegistration,
    KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, CreatedEntityRef, EntityMutationIntent, EntitySpec,
    MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
};

use crate::domain_computation::primary_graph::merge_unique_values::{
    admit_merge_unique_values, WorthQueryMergeUniqueValueDenialKind as Kind,
};
use crate::domain_computation::primary_graph::schema_layout::WorthQueryUniqueFieldFixture;

const KIND: KindId = KindId(1);
const FEATURE: &str = "feature";
const NOTE: &str = "note";

#[test]
fn a_merge_that_updates_to_a_value_the_target_holds_is_denied() {
    let mut world = UpdateWorld::new();
    world.create("ada", "taken");
    let bob = world.create("bob", "bob-label");
    world.fork();
    world.update(bob, "bob", "taken");
    assert_eq!(world.admit(), Err(Kind::ValueTaken));
}

#[test]
fn a_merge_that_updates_to_a_free_value_is_admitted() {
    let mut world = UpdateWorld::new();
    world.create("ada", "taken");
    let bob = world.create("bob", "bob-label");
    world.fork();
    world.update(bob, "bob", "free");
    assert_eq!(world.admit(), Ok(()));
}

/// An entity the merge updates keeps its own unique label: holding the
/// value itself is not a collision.
#[test]
fn a_merge_that_keeps_the_updated_entitys_own_value_is_admitted() {
    let mut world = UpdateWorld::new();
    world.create("ada", "taken");
    let bob = world.create("bob", "bob-label");
    world.fork();
    world.update_note(bob, "bob", "bob-label", "edited");
    let prepared = world.prepared_merge();
    assert!(
        prepared.merged_intents().iter().any(|intent| matches!(
            intent,
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(update))
                if update.entity_id == bob
        )),
        "the merge updates bob: {:?}",
        prepared.merged_intents()
    );
    assert_eq!(world.admit_prepared(&prepared), Ok(()));
}

/// A raw runtime whose one kind has an identity `name` and a unique `label`
/// with an equality index and a last-writer-wins merge policy.
struct UpdateWorld {
    runtime: RelationalRuntime,
    index: DerivedIndexId,
}

impl UpdateWorld {
    fn new() -> Self {
        let runtime = RelationalRuntimeApi::builder()
            .schema_registry(registry())
            .build();
        let index = runtime
            .index_authority()
            .register(DerivedIndexDefinition {
                index_id: DerivedIndexId(0),
                name: "label-equality".to_owned(),
                kind: DerivedIndexKind::EntityField {
                    field_locator: locator("label"),
                },
                branch_scoped: false,
            })
            .index_id;
        Self { runtime, index }
    }

    fn create(&mut self, name: &str, label: &str) -> EntityId {
        let created = CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: KIND,
            client_key: ClientKey::raw(name),
        };
        let committed = self.commit(
            "main",
            MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: created.partition_id,
                kind_id: KIND,
                client_key: created.client_key.clone(),
                fields: fields(name, label),
            })),
        );
        let entity = committed
            .created_entity(&created)
            .expect("the create issued");
        self.close(committed);
        entity
    }

    fn update(&mut self, entity_id: EntityId, name: &str, label: &str) {
        self.update_note(entity_id, name, label, NOTE);
    }

    fn update_note(&mut self, entity_id: EntityId, name: &str, label: &str, note: &str) {
        let committed = self.commit(
            FEATURE,
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                UpdateEntityFieldsIntent {
                    entity_id,
                    fields: noted_fields(name, label, note),
                },
            )),
        );
        self.close(committed);
    }

    fn fork(&mut self) {
        let (_, source) = self
            .runtime
            .observe_fork_source(&BranchId("main".to_owned()))
            .expect("main exposes a fork source");
        self.runtime
            .fork_branch(BranchId(FEATURE.to_owned()), source)
            .expect("the feature branch forks");
    }

    /// Admits the feature-into-main merge, which must carry the label as a
    /// field update, under the label's unique law.
    fn admit(&mut self) -> Result<(), Kind> {
        let prepared = self.prepared_merge();
        assert!(
            prepared.merged_intents().iter().any(|intent| matches!(
                intent,
                MutationIntent::Entity(EntityMutationIntent::UpdateFields(update))
                    if update.fields.iter().any(|(field, _)| *field == locator("label"))
            )),
            "the merge writes the label as a field update: {:?}",
            prepared.merged_intents()
        );
        self.admit_prepared(&prepared)
    }

    fn admit_prepared(&self, prepared: &PreparedMergeExecution) -> Result<(), Kind> {
        let unique = WorthQueryUniqueFieldFixture::new(KIND, locator("label"), Some(self.index));
        admit_merge_unique_values(unique.fields(), &self.runtime, prepared)
            .map_err(|denial| denial.kind())
    }

    fn prepared_merge(&mut self) -> PreparedMergeExecution {
        let request = MergeExecutionRequest::new(
            BranchId("main".to_owned()),
            BranchId(FEATURE.to_owned()),
            MergeIntent::ReconcileIntoTarget,
        );
        let bound = self
            .runtime
            .bind_merge_execution_request(request)
            .expect("the merge binds");
        self.runtime
            .prepare_merge_execution(bound)
            .expect("a last-writer-wins source edit prepares")
    }

    fn commit(
        &mut self,
        branch: &str,
        intent: MutationIntent,
    ) -> worth_relational::facade::transactions::CommitResult {
        let runtime = &mut self.runtime;
        let identity = runtime
            .branch_identity(&BranchId(branch.to_owned()))
            .expect("the branch exists");
        let basis = runtime
            .admit_branch_basis(&identity)
            .expect("the branch admits");
        let mut transaction = runtime
            .begin_branch_transaction(
                &basis,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("the fixture transaction begins");
        transaction
            .push_batch(WorkerIntentBatch::new("unique-update").push(intent))
            .expect("the fixture batch stages");
        transaction.commit(runtime).expect("the fixture commits")
    }

    /// Builds the label index at the new commit and releases its snapshot.
    fn close(&mut self, committed: worth_relational::facade::transactions::CommitResult) {
        let built = self
            .runtime
            .index_authority()
            .build_for_commit(DerivedIndexBuildRequest {
                source_commit_id: committed.commit.commit_id,
                branch_id: committed.commit.branch_id.clone(),
                index_ids: vec![self.index],
            });
        assert_eq!(
            built.generations.len(),
            1,
            "the label index builds: {built:?}"
        );
        self.runtime
            .snapshots()
            .release_snapshot(&committed.snapshot)
            .expect("the fixture releases its commit snapshot");
    }
}

fn registry() -> RelationalSchemaRegistry {
    let name = aspect_key("name");
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KIND,
            kind_name: "unique.update".to_owned(),
            schema_id: SchemaId("unique-update".to_owned()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                string_aspect("name", 0x5151_0001),
                string_aspect("label", 0x5151_0002),
                string_aspect("note", 0x5151_0003),
            ])
            .with_identity_declarations(vec![IdentityBasisDeclaration {
                scope: IdentityBasisScope::AspectKey(name.clone()),
                basis: IdentityBasisKind::DeclaredKeySet(vec![name].into()),
            }])
            .with_merge_policy_declarations(
                ["label", "note"]
                    .into_iter()
                    .map(|key| AspectMergePolicyDeclaration {
                        aspect_key: aspect_key(key),
                        policy: AspectMergePolicyKind::LastWriterWins,
                    })
                    .collect(),
            ),
        })
        .expect("the fixture schema registers")
}

fn string_aspect(key: &str, identity: u64) -> DeclaredAspectContractBinding {
    DeclaredAspectContractBinding {
        binding: AspectBinding::EntityField {
            field: FieldKey::new(key).expect("field key admits"),
        },
        contract: AspectContract::scalar(
            aspect_key(key),
            AspectIdentity(identity),
            AspectContractRevision(1),
            ScalarAspectType::String,
        ),
    }
}

fn fields(name: &str, label: &str) -> AspectFieldPatch {
    noted_fields(name, label, NOTE)
}

fn noted_fields(name: &str, label: &str, note: &str) -> AspectFieldPatch {
    AspectFieldPatch::from(BTreeMap::from([
        (locator("name"), string(name)),
        (locator("label"), string(label)),
        (locator("note"), string(note)),
    ]))
}

fn locator(key: &str) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Planned,
        aspect_key(key),
        CanonicalFieldPath::single(FieldKey::new(key).expect("field key admits")),
    )
}

fn aspect_key(key: &str) -> AspectKey {
    AspectKey::new(key).expect("aspect key admits")
}

fn string(value: &str) -> AspectValue {
    AspectValue::String(InternedString::from(value.to_owned()))
}
