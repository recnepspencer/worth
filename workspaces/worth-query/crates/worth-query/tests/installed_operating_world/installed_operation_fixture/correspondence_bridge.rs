use std::{collections::BTreeMap, sync::Arc};

use worth_foundational::facade::{
    AspectBinding, AspectContract, AspectFieldLocator, AspectValue, CanonicalFieldPath, FieldKey,
    LocatorAuthority,
};
use worth_relational::facade::bridge::RuntimeBridgeRelationalSource;
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_relational::facade::mvcc::{
    BranchBoundRelationalTransaction, RelationalTransactionIntent,
};
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_relational::facade::schema::{
    DeclaredAspectContractBinding, EntityKindRegistration, KindAspectContractDeclarations,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, EntityMutationIntent, EntitySpec, MutationIntent, RecordRef,
    UpdateEntityFieldsIntent, WorkerIntentBatch,
};
use worth_runtime_bridge::facade::{
    AspectKeySelector, BridgeAspectRegistration, BridgeAspectRegistrationId, BridgeDeliveryReceipt,
    BridgeMappingId, BridgeMappingRegistration, BridgeSemanticCorrespondenceRegistration,
    BridgeSemanticLocality, CoarseRoutingMode, InvalidationSink, MappingSelector,
    RelationalCommittedPatchRequest, RuntimeBridge, RuntimeBridgeBuilder, SignalBridgeSinkError,
    SignalInvalidationScope, SliceWideningPolicy, SnapshotReadContract, SubscriptionSliceKind,
    TruthCommitIdentity, TruthDeltaSurfaceKind, TruthPatchScope, TruthPatchTargetSelector,
};

mod commit_snapshot_closeout;
mod retained_relational_source;

use retained_relational_source::RetainedRelationalSource;

struct CorrespondenceSink;

impl InvalidationSink for CorrespondenceSink {
    fn deliver_invalidation(
        &self,
        delivery: worth_runtime_bridge::facade::BridgeSignalInvalidationDelivery,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}

pub(crate) fn correspondence_bridge(
    registration: BridgeSemanticCorrespondenceRegistration,
) -> (RuntimeBridge, RelationalCommittedPatchRequest) {
    let dependency = registration.dependency();
    let mut relational = RelationalRuntimeApi::builder()
        .schema_registry(
            RelationalSchemaRegistry::new()
                .register_entity_kind(EntityKindRegistration {
                    kind_id: KindId(1),
                    kind_name: "conditional.geometry".into(),
                    schema_id: SchemaId("conditional-geometry".into()),
                    schema_version_id: SchemaVersionId(1),
                    aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                        DeclaredAspectContractBinding {
                            binding: dependency.binding().clone(),
                            contract: dependency.contract().clone(),
                        },
                    ]),
                })
                .expect("conditional correspondence schema should register"),
        )
        .build();
    let field = dependency_field(dependency.binding());
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Planned,
        dependency.contract().key().clone(),
        CanonicalFieldPath::single(field.clone()),
    );
    let mut create = begin_main_transaction(&relational);
    create
        .push_batch(WorkerIntentBatch::new("create-conditional-entity").push(
            MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw("conditional-entity"),
                fields: AspectFieldPatch::new(BTreeMap::from([(
                    locator.clone(),
                    AspectValue::String("before".into()),
                )])),
            })),
        ))
        .expect("test staging stays within configured resource budgets");
    let created = create
        .commit(&relational)
        .expect("conditional entity should commit");
    let entity = created
        .changed_records
        .iter()
        .find_map(|record| match record {
            RecordRef::Entity(entity) => Some(*entity),
            RecordRef::Relation(_) => None,
        })
        .expect("create commit should retain its entity identity");
    commit_snapshot_closeout::release_commit_snapshot(&mut relational, &created);

    let mut update = begin_main_transaction(&relational);
    update
        .push_batch(WorkerIntentBatch::new("update-conditional-identity").push(
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: AspectFieldPatch::from_locator(
                        locator,
                        AspectValue::String("after".into()),
                    ),
                },
            )),
        ))
        .expect("test staging stays within configured resource budgets");
    let updated = update
        .commit(&relational)
        .expect("conditional identity field should commit");
    let branch_identity = relational
        .branch_identity(&updated.commit.branch_id)
        .expect("updated branch identity");
    let (_, updated_basis) = relational
        .observe_branch(&branch_identity)
        .expect("updated owner-admitted branch basis");
    let request = RelationalCommittedPatchRequest::new(
        TruthCommitIdentity::from_relational_commit_id(updated.commit.commit_id.0),
    );
    commit_snapshot_closeout::release_commit_snapshot(&mut relational, &updated);
    let source = match dependency.locality() {
        BridgeSemanticLocality::SourcePartition(role) => {
            RuntimeBridgeRelationalSource::for_graph_partition(
                Arc::new(relational),
                "model",
                PartitionId::main(),
                role.clone(),
            )
        }
        BridgeSemanticLocality::SourceRecord
        | BridgeSemanticLocality::ManagedSourceRecord
        | BridgeSemanticLocality::WholeLogicalGraph => {
            RuntimeBridgeRelationalSource::for_graph_role(Arc::new(relational), "model")
        }
    }
    .expect("model is a valid graph role");
    let locality = match dependency.locality() {
        BridgeSemanticLocality::SourceRecord | BridgeSemanticLocality::ManagedSourceRecord => {
            FixtureLocality::Record
        }
        BridgeSemanticLocality::SourcePartition(_) => FixtureLocality::Partition,
        BridgeSemanticLocality::WholeLogicalGraph => FixtureLocality::Graph,
    };
    let contract = dependency.contract().clone();
    let (source, _) = RetainedRelationalSource::new(source, vec![updated_basis]);
    let bridge = build_bridge(
        source,
        &contract,
        field,
        dependency.projection_mask().is_whole_aspect(),
        locality,
        Some(registration),
        Some(
            worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(
                PartitionId::main().0,
                entity.local_slot_value(),
                entity.generation_value(),
            ),
        ),
    );
    (bridge, request)
}

pub(crate) fn conditional_runtime_bridge_from_source(
    owner: &worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner,
    dependency: &worth_query::facade::domain::WorthQuerySemanticTruthDependency,
    record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
) -> RuntimeBridge {
    let locality = match dependency.locality() {
        worth_query::facade::domain::WorthQuerySemanticLocality::SourceRecord => {
            FixtureLocality::Record
        }
        worth_query::facade::domain::WorthQuerySemanticLocality::SourcePartition(_) => {
            FixtureLocality::Partition
        }
        worth_query::facade::domain::WorthQuerySemanticLocality::WholeLogicalGraph => {
            FixtureLocality::Graph
        }
    };
    let (source, _) = RetainedRelationalSource::new(owner.bridge_source(), Vec::new());
    build_bridge(
        source,
        dependency.contract(),
        dependency_field(dependency.binding()),
        dependency.projection_mask().is_whole_aspect(),
        locality,
        None,
        Some(record),
    )
}
#[derive(Clone, Copy)]
enum FixtureLocality {
    Record,
    Partition,
    Graph,
}

fn build_bridge(
    source: RetainedRelationalSource,
    contract: &AspectContract,
    field: FieldKey,
    whole_aspect: bool,
    locality: FixtureLocality,
    registration: Option<BridgeSemanticCorrespondenceRegistration>,
    record: Option<worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts>,
) -> RuntimeBridge {
    let target = if whole_aspect {
        TruthPatchTargetSelector::authoritative_aspect()
    } else {
        TruthPatchTargetSelector::entity_field(field)
    };
    let truth_surface = if whole_aspect {
        TruthDeltaSurfaceKind::AuthoritativeAspect
    } else {
        TruthDeltaSurfaceKind::EntityField
    };
    let entity_selector = match locality {
        FixtureLocality::Record => MappingSelector::exact(
            record
                .expect("record mapping requires the committed source identity")
                .terminal_projection_for_reporting(),
        ),
        FixtureLocality::Partition | FixtureLocality::Graph => MappingSelector::any(),
    };
    let mapping = BridgeMappingRegistration::new(
        BridgeMappingId::from_stable_name("conditional-identity"),
        TruthPatchScope::new(
            entity_selector,
            AspectKeySelector::exact(contract.key().clone()),
            target,
        ),
        SnapshotReadContract::new(contract.clone()),
        SignalInvalidationScope::from_stable_name("conditional-identity"),
        CoarseRoutingMode::Direct,
    );
    let (slice, widening) = match locality {
        FixtureLocality::Partition => (
            SubscriptionSliceKind::SignalPartition,
            SliceWideningPolicy::RegisteredPartitionWidening,
        ),
        FixtureLocality::Record => (
            SubscriptionSliceKind::SignalField,
            SliceWideningPolicy::Disallow,
        ),
        FixtureLocality::Graph => (
            SubscriptionSliceKind::SignalField,
            SliceWideningPolicy::Disallow,
        ),
    };
    let aspect_mapping = BridgeAspectRegistration::new(
        BridgeAspectRegistrationId::from_stable_name("conditional-identity"),
        mapping.truth_scope().clone(),
        mapping.snapshot_read_contract().clone(),
        truth_surface,
        slice,
        widening,
    );
    let builder = RuntimeBridgeBuilder::new()
        .with_relational_source(source)
        .with_signal_sink(CorrespondenceSink)
        .register_mapping(mapping)
        .register_aspect_mapping(aspect_mapping);
    match registration {
        Some(registration) => builder
            .register_semantic_correspondence(registration)
            .build(),
        None => builder.build(),
    }
    .expect("conditional correspondence bridge should build")
}

fn begin_main_transaction(runtime: &RelationalRuntime) -> BranchBoundRelationalTransaction {
    let main_identity = runtime.main_branch_identity();
    let context = runtime
        .admit_branch_basis(&main_identity)
        .expect("main branch context");
    runtime
        .begin_branch_transaction(&context, RelationalTransactionIntent::ordinary())
        .expect("owner-admitted main basis")
}

fn dependency_field(binding: &AspectBinding) -> FieldKey {
    match binding {
        AspectBinding::EntityField { field } | AspectBinding::RelationField { field } => {
            field.clone()
        }
        _ => FieldKey::new("id").unwrap(),
    }
}
