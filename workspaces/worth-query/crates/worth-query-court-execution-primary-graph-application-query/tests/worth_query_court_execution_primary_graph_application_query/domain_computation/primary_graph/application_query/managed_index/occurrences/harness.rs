//! Real RelationalRuntime + PersistentSourceRows harness for occurrence update courts.
use super::{
    companion::{Companion, ObservedState, SourceTriggerRule},
    schema::{FieldedIndex, LabelField, ScopeKeyField, SelfRefSchema, ZeroFieldIndex},
};
use std::sync::{Arc, Mutex};
use worth_execution::{
    CancellationSource, ExecutionAllocationPolicy, ExecutionRequest, SerialMemoryBudget,
    SerialRequest,
};
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};
use worth_query_execution::{
    court_doors_for_test::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout,
    facade::domain_computation::WorthQueryExecutionRuntimeInstaller,
};
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstalledGraphReadContract, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage,
};
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId, RelationId},
    mvcc::{PublicationCompanionRegistration, RelationalTransactionIntent},
    runtime::{CustomInvariantRegistration, RelationalRuntime, RelationalRuntimeApi},
    schema::RelationalSchemaRegistry,
    symbols::ClientKey,
    transactions::{
        AspectFieldLocator, AspectFieldPatch, CommitResult, CreateIntent, CreatedEntityRef,
        CreatedRelationRef, DeleteEntityIntent, DeleteRelationIntent, EntityMutationIntent,
        EntityReference, EntitySpec, MutationIntent, RelationMutationIntent, RelationSpec,
        UpdateEntityFieldsIntent, WorkerIntentBatch,
    },
};

struct Profile {
    node: &'static str,
    contains: &'static str,
    self_link: &'static str,
    tag_link: &'static str,
    with_label: bool,
}

pub(super) struct Harness {
    pub runtime: RelationalRuntime,
    pub state: Arc<Mutex<ObservedState>>,
    pub scope_kind: KindId,
    pub node_kind: KindId,
    pub tag_kind: KindId,
    pub contains_kind: KindId,
    pub self_link_kind: KindId,
    pub tag_link_kind: KindId,
    pub scope_key_locator: AspectFieldLocator,
    pub label_locator: Option<AspectFieldLocator>,
    pub host: SerialRequest,
    _registration: PublicationCompanionRegistration,
}

fn lower_schema<I: ApplicationManagedIndexBinding<SelfRefSchema>>() -> (
    WorthQueryPrimaryGraphLayout,
    RelationalSchemaRegistry,
    WorthQueryInstalledGraphReadContract,
) {
    let declaration = SelfRefSchema::declaration().unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        SelfRefSchema::OWNER,
        1,
        0,
    ))
    .application_schema(declaration.clone())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("self-ref-test", "local")
        .admit(package)
        .unwrap();
    let (installer_rt, _) = WorthQueryExecutionRuntimeInstaller::new()
        .install(WorthQueryInstallationGeneration::initial(), [admitted])
        .unwrap()
        .into_parts();
    let installed = installer_rt
        .installed_packages()
        .bind_application_schema(declaration)
        .unwrap();
    let graph = installed
        .installed_managed_index::<I>()
        .unwrap()
        .source_read()
        .query()
        .read_graph()
        .clone();
    let (layout, registry) = WorthQueryPrimaryGraphLayout::lower_for_test(
        installed.installed_declaration(),
        installed.native_contracts(),
        &RelationalSchemaRegistry::new(),
        installed.value_bindings(),
    )
    .unwrap();
    (layout, registry, graph)
}

impl Harness {
    pub(super) fn zero_field() -> Self {
        Self::install::<ZeroFieldIndex>(Profile {
            node: "ZeroNode",
            contains: "ContainsZero",
            self_link: "ZeroSelf",
            tag_link: "ZeroTag",
            with_label: false,
        })
    }

    pub(super) fn fielded() -> Self {
        Self::install::<FieldedIndex>(Profile {
            node: "FieldedNode",
            contains: "ContainsFielded",
            self_link: "FieldedSelf",
            tag_link: "FieldedTag",
            with_label: true,
        })
    }

    fn install<I: ApplicationManagedIndexBinding<SelfRefSchema>>(profile: Profile) -> Self {
        let (layout, registry, graph) = lower_schema::<I>();
        let program = layout.managed_source_for_test(I::NAME).unwrap().clone();
        let scope_kind = layout.entity_kind_for_test("Scope").unwrap();
        let node_kind = layout.entity_kind_for_test(profile.node).unwrap();
        let tag_kind = layout.entity_kind_for_test("Tag").unwrap();
        let contains_kind = *layout.relation(profile.contains).unwrap().kind_for_test();
        let self_link_kind = *layout.relation(profile.self_link).unwrap().kind_for_test();
        let tag_link_kind = *layout.relation(profile.tag_link).unwrap().kind_for_test();
        let scope_key = ScopeKeyField::at("ScopeKey");
        let scope_key_locator = layout
            .field_locator_for_test(scope_key.entity(), scope_key.aspect(), scope_key.field())
            .unwrap()
            .clone();
        let label_locator = profile.with_label.then(|| {
            let label = LabelField::at("Label");
            layout
                .field_locator_for_test(label.entity(), label.aspect(), label.field())
                .unwrap()
                .clone()
        });
        let mut entity_kinds = vec![scope_kind, node_kind, tag_kind];
        let mut relation_kinds = vec![contains_kind, self_link_kind, tag_link_kind];
        for route in program.projection_routes() {
            entity_kinds.push(route.endpoint_kind());
            relation_kinds.push(route.physical_relation().0);
        }
        entity_kinds.sort_unstable();
        entity_kinds.dedup();
        relation_kinds.sort_unstable();
        relation_kinds.dedup();
        let rule = CustomInvariantRegistration::new(SourceTriggerRule {
            program: program.clone(),
            entity_kinds,
            relation_kinds,
        })
        .unwrap()
        .with_managed_index_sources(vec![program])
        .unwrap();
        let runtime = RelationalRuntimeApi::builder()
            .schema_registry(registry)
            .custom_invariant(rule)
            .build()
            .unwrap();
        let state = Arc::new(Mutex::new(ObservedState::default()));
        let registration = runtime
            .publication_companion_port()
            .begin_required_registration()
            .unwrap()
            .activate(Arc::new(Companion {
                state: Arc::clone(&state),
                graph,
            }))
            .unwrap();
        let cancel = CancellationSource::new();
        let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), cancel.token(), None);
        Self {
            runtime,
            state,
            scope_kind,
            node_kind,
            tag_kind,
            contains_kind,
            self_link_kind,
            tag_link_kind,
            scope_key_locator,
            label_locator,
            host,
            _registration: registration,
        }
    }

    pub(super) fn commit(&self, batch: WorkerIntentBatch) -> CommitResult {
        let execution = ExecutionRequest::serial(&self.host);
        let (_, basis) = self
            .runtime
            .observe_branch(&self.runtime.main_branch_identity())
            .unwrap();
        let mut tx = self
            .runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        tx.push_batch(batch, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        tx.commit(&execution, &self.runtime).unwrap()
    }

    pub(super) fn create_scope(&self, key: &str) -> EntityId {
        let res = self.commit(
            WorkerIntentBatch::new("create-scope").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: self.scope_kind,
                    client_key: ClientKey::raw(key),
                    fields: AspectFieldPatch::from_locator(
                        self.scope_key_locator.clone(),
                        AspectValue::String(key.into()),
                    ),
                }),
            )),
        );
        res.created_entity(&CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: self.scope_kind,
            client_key: ClientKey::raw(key),
        })
        .unwrap()
    }

    pub(super) fn create_node(&self, key: &str, label: Option<&str>) -> EntityId {
        let fields = match (&self.label_locator, label) {
            (Some(locator), Some(text)) => {
                AspectFieldPatch::from_locator(locator.clone(), AspectValue::String(text.into()))
            }
            _ => AspectFieldPatch::default(),
        };
        let res = self.commit(
            WorkerIntentBatch::new("create-node").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: self.node_kind,
                    client_key: ClientKey::raw(key),
                    fields,
                }),
            )),
        );
        res.created_entity(&CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: self.node_kind,
            client_key: ClientKey::raw(key),
        })
        .unwrap()
    }

    pub(super) fn create_tag(&self, key: &str) -> EntityId {
        let res = self.commit(
            WorkerIntentBatch::new("create-tag").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: self.tag_kind,
                    client_key: ClientKey::raw(key),
                    fields: AspectFieldPatch::default(),
                }),
            )),
        );
        res.created_entity(&CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: self.tag_kind,
            client_key: ClientKey::raw(key),
        })
        .unwrap()
    }

    pub(super) fn link(&self, kind: KindId, from: EntityId, to: EntityId, key: &str) -> RelationId {
        let res =
            self.commit(WorkerIntentBatch::new("link").push(self.link_intent(kind, from, to, key)));
        res.created_relation(&CreatedRelationRef {
            partition_id: PartitionId::main(),
            kind_id: kind,
            client_key: ClientKey::raw(key),
            source: EntityReference::Existing(from),
            target: EntityReference::Existing(to),
        })
        .unwrap()
    }

    pub(super) fn link_intent(
        &self,
        kind: KindId,
        from: EntityId,
        to: EntityId,
        key: &str,
    ) -> MutationIntent {
        MutationIntent::Create(CreateIntent::Relation(RelationSpec {
            partition_id: PartitionId::main(),
            kind_id: kind,
            client_key: ClientKey::raw(key),
            source: EntityReference::Existing(from),
            target: EntityReference::Existing(to),
            fields: AspectFieldPatch::default(),
        }))
    }

    pub(super) fn unlink_intent(&self, relation_id: RelationId) -> MutationIntent {
        MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
            relation_id,
        }))
    }

    pub(super) fn update_label_intent(&self, entity_id: EntityId, text: &str) -> MutationIntent {
        MutationIntent::Entity(EntityMutationIntent::UpdateFields(
            UpdateEntityFieldsIntent {
                entity_id,
                fields: AspectFieldPatch::from_locator(
                    self.label_locator.clone().unwrap(),
                    AspectValue::String(text.into()),
                ),
            },
        ))
    }

    pub(super) fn delete_entity_intent(&self, entity_id: EntityId) -> MutationIntent {
        MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
            entity_id,
        }))
    }
}
