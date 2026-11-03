//! A merge's field updates obey the unique law. Application kinds declare
//! no merge policy yet, so this world is a raw kind whose unique `label`
//! merges last-writer-wins; the lookup reads any kind the same way, so the
//! update arm runs here exactly as it would for an application kind.

use std::collections::BTreeMap;

use worth_foundational::facade::AspectContract;
use worth_foundational::facade::AspectContractRevision;
use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::facade::AspectIdentity;
use worth_foundational::facade::AspectKey;
use worth_foundational::facade::AspectValue;
use worth_foundational::facade::CanonicalFieldPath;
use worth_foundational::facade::FieldKey;
use worth_foundational::facade::InternedString;
use worth_foundational::facade::LocatorAuthority;
use worth_foundational::facade::ScalarAspectType;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::indexes::DerivedIndexBuildRequest;
use worth_relational::facade::indexes::DerivedIndexDefinition;
use worth_relational::facade::indexes::DerivedIndexId;
use worth_relational::facade::indexes::DerivedIndexKind;
use worth_relational::facade::merge::AspectMergePolicyDeclaration;
use worth_relational::facade::merge::AspectMergePolicyKind;
use worth_relational::facade::merge::IdentityBasisDeclaration;
use worth_relational::facade::merge::IdentityBasisKind;
use worth_relational::facade::merge::IdentityBasisScope;
use worth_relational::facade::merge::MergeExecutionRequest;
use worth_relational::facade::merge::MergeIntent;
use worth_relational::facade::merge::PreparedMergeExecution;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_relational::facade::schema::AspectBinding;
use worth_relational::facade::schema::DeclaredAspectContractBinding;
use worth_relational::facade::schema::EntityKindRegistration;
use worth_relational::facade::schema::KindAspectContractDeclarations;
use worth_relational::facade::schema::RelationalSchemaRegistry;
use worth_relational::facade::schema::SchemaId;
use worth_relational::facade::schema::SchemaVersionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::CreateIntent;
use worth_relational::facade::transactions::CreatedEntityRef;
use worth_relational::facade::transactions::EntityMutationIntent;
use worth_relational::facade::transactions::EntitySpec;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;

use worth_query_execution::domain_computation::primary_graph::merge_unique_values::admit_merge_unique_values;
use worth_query_execution::domain_computation::primary_graph::schema_layout::unique_fields::WorthQueryUniqueFieldFixture;
use worth_query_execution::facade::integration::WorthQueryMergeUniqueValueDenialKind as Kind;

pub const KIND: KindId = KindId(1);
pub const FEATURE: &str = "feature";
pub const NOTE: &str = "note";

#[test]
fn a_merge_that_updates_to_a_value_the_target_holds_is_denied() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut world = UpdateWorld::new();
    world.create(execution, "ada", "taken");
    let bob = world.create(execution, "bob", "bob-label");
    world.fork(execution);
    world.update(execution, bob, "bob", "taken");
    assert_eq!(world.admit(execution,), Err(Kind::ValueTaken));
}

#[test]
fn a_merge_that_updates_to_a_free_value_is_admitted() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut world = UpdateWorld::new();
    world.create(execution, "ada", "taken");
    let bob = world.create(execution, "bob", "bob-label");
    world.fork(execution);
    world.update(execution, bob, "bob", "free");
    assert_eq!(world.admit(execution,), Ok(()));
}

/// An entity the merge updates keeps its own unique label: holding the
/// value itself is not a collision.
#[test]
fn a_merge_that_keeps_the_updated_entitys_own_value_is_admitted() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut world = UpdateWorld::new();
    world.create(execution, "ada", "taken");
    let bob = world.create(execution, "bob", "bob-label");
    world.fork(execution);
    world.update_note(execution, bob, "bob", "bob-label", "edited");
    let prepared = world.prepared_merge(execution);
    assert!(
        prepared.merged_intents().iter().any(|intent| matches!(
            intent,
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(update))
                if update.entity_id == bob
        )),
        "the merge updates bob: {:?}",
        prepared.merged_intents()
    );
    assert_eq!(world.admit_prepared(execution, &prepared), Ok(()));
}

/// A raw runtime whose one kind has an identity `name` and a unique `label`
/// with an equality index and a last-writer-wins merge policy.
pub struct UpdateWorld {
    pub runtime: RelationalRuntime,
    pub index: DerivedIndexId,
}

impl UpdateWorld {
    pub fn new() -> Self {
        let runtime = RelationalRuntimeApi::builder()
            .schema_registry(registry())
            .build()
            .expect("valid native runtime configuration");
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
            .expect("index fixture has identity capacity")
            .index_id;
        Self { runtime, index }
    }

    pub fn create(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        name: &str,
        label: &str,
    ) -> EntityId {
        let created = CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: KIND,
            client_key: ClientKey::raw(name),
        };
        let committed = self.commit(
            execution,
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
        self.close(execution, committed);
        entity
    }

    pub fn update(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        name: &str,
        label: &str,
    ) {
        self.update_note(execution, entity_id, name, label, NOTE);
    }

    pub fn update_note(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        name: &str,
        label: &str,
        note: &str,
    ) {
        let committed = self.commit(
            execution,
            FEATURE,
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                UpdateEntityFieldsIntent {
                    entity_id,
                    fields: noted_fields(name, label, note),
                },
            )),
        );
        self.close(execution, committed);
    }

    pub fn fork(&mut self, execution: &worth_execution::ExecutionRequest<'_, '_>) {
        let (_, source) = self
            .runtime
            .observe_fork_source(&BranchId("main".to_owned()))
            .expect("main exposes a fork source");
        self.runtime
            .fork_branch(execution, BranchId(FEATURE.to_owned()), source)
            .expect("the feature branch forks");
    }

    /// Admits the feature-into-main merge, which must carry the label as a
    /// field update, under the label's unique law.
    pub fn admit(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), Kind> {
        let prepared = self.prepared_merge(execution);
        assert!(
            prepared.merged_intents().iter().any(|intent| matches!(
                intent,
                MutationIntent::Entity(EntityMutationIntent::UpdateFields(update))
                    if update.fields.iter().any(|(field, _)| *field == locator("label"))
            )),
            "the merge writes the label as a field update: {:?}",
            prepared.merged_intents()
        );
        self.admit_prepared(execution, &prepared)
    }

    pub fn admit_prepared(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        prepared: &PreparedMergeExecution,
    ) -> Result<(), Kind> {
        let unique = WorthQueryUniqueFieldFixture::new(KIND, locator("label"), Some(self.index));
        admit_merge_unique_values(unique.fields(), &self.runtime, execution, prepared)
            .map_err(|denial| denial.kind())
    }

    pub fn prepared_merge(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> PreparedMergeExecution {
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
            .prepare_merge_execution(execution, bound)
            .expect("a last-writer-wins source edit prepares")
    }

    pub fn commit(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
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
            .push_batch(
                WorkerIntentBatch::new("unique-update").push(intent),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("the fixture batch stages");
        transaction
            .commit(execution, runtime)
            .expect("the fixture commits")
    }

    /// Builds the label index at the new commit and releases its snapshot.
    pub fn close(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        committed: worth_relational::facade::transactions::CommitResult,
    ) {
        let built = self.runtime.index_authority().build_for_commit(
            execution,
            DerivedIndexBuildRequest {
                source_commit_id: committed.commit.commit_id,
                branch_id: committed.commit.branch_id.clone(),
                index_ids: vec![self.index],
            },
        );
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

pub fn registry() -> RelationalSchemaRegistry {
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

pub fn string_aspect(key: &str, identity: u64) -> DeclaredAspectContractBinding {
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

pub fn fields(name: &str, label: &str) -> AspectFieldPatch {
    noted_fields(name, label, NOTE)
}

pub fn noted_fields(name: &str, label: &str, note: &str) -> AspectFieldPatch {
    AspectFieldPatch::from(BTreeMap::from([
        (locator("name"), string(name)),
        (locator("label"), string(label)),
        (locator("note"), string(note)),
    ]))
}

pub fn locator(key: &str) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Planned,
        aspect_key(key),
        CanonicalFieldPath::single(FieldKey::new(key).expect("field key admits")),
    )
}

pub fn aspect_key(key: &str) -> AspectKey {
    AspectKey::new(key).expect("aspect key admits")
}

pub fn string(value: &str) -> AspectValue {
    AspectValue::String(InternedString::from(value.to_owned()))
}
