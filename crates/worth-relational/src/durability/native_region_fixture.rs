//! Author an exact region image without constructing a runtime.
use worth_relational_storage::internal_port::storage::partition::adjacency::AdjacencySetFixturePort as _;
use worth_relational_storage::internal_port::storage::substrate::record_arena::arena::slot_lifecycle::metadata::RecordArenaFixturePort as _;
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistryFieldsPort as _;
use worth_relational_contracts::internal_port::indexes::data::entry_map::DerivedIndexEntryMapOwnerPort as _;
use worth_relational_storage::internal_port::durability::medium::member_custody::RelationalDurableMemberCustodyOwnerPort as _;
use crate::branch::root::schema::RelationalBranchRootSchemaAuthority;
use crate::config::data::{
    AdjacencyBackend, AdjacencyPolicy, CascadeDeletePolicy, CrossContextPolicy,
};
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId, VersionId};
use crate::schema::data::{
    AspectBinding, DeclaredAspectContractBinding, DescriptorSemanticsVersion,
    EntityKindRegistration, KindAspectContractDeclarations, RelationKindRegistration,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::{
    EntityArena, EntityExtra, RelationArena, RelationEndpoints, RelationExtra, SlotInit,
};
use crate::symbols::data::StringInterner;
use std::sync::Arc;
use worth_foundational::facade::{
    admit_authoritative_record_aspect_state, validate_aspect_value, AbsenceLaw, AspectContract,
    AspectContractRevision, AspectEvolutionPolicy, AspectIdentity, AspectKey, AspectValue,
    ContractValidationInput, FieldDeclaration, FieldKey, FieldRequirement, InternedString,
    ScalarAspectType, StructAspectShape, StructAspectValue,
};
use worth_proof::TransitionOutcome;

pub(crate) struct HistoryPartition {
    pub(crate) partition: PartitionState,
    pub(crate) entity: EntityId,
    pub(crate) relation: RelationId,
    pub(crate) schema: Arc<RelationalBranchRootSchemaAuthority>,
    pub(crate) symbols: StringInterner,
}

pub(crate) fn populated_partition() -> HistoryPartition {
    let fields = ["classification", "name", "source_document"].map(|field| {
        FieldDeclaration::new(
            FieldKey::new(field).unwrap(),
            ScalarAspectType::String,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            AspectEvolutionPolicy::AdditiveFieldsAllowed,
        )
        .unwrap()
    });
    let contract = AspectContract::struct_aspect(
        AspectKey::new("name").unwrap(),
        AspectIdentity(1),
        AspectContractRevision(1),
        StructAspectShape::new(fields).unwrap(),
    );
    let partition_id = PartitionId::main();
    let entity_kind = KindId(1);
    let relation_kind = KindId(2);
    let entity = EntityId::new(partition_id, 0, 1);
    let relation = RelationId::new(partition_id, 0, 1);
    let schema_id = SchemaId("native-resident-history".into());
    let schema_version_id = SchemaVersionId(1);
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: entity_kind,
            kind_name: "entity".into(),
            schema_id: schema_id.clone(),
            schema_version_id,
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                DeclaredAspectContractBinding {
                    binding: AspectBinding::EntityField {
                        field: FieldKey::new("name").unwrap(),
                    },
                    contract: contract.clone(),
                },
            ]),
        })
        .unwrap()
        .register_relation_kind(RelationKindRegistration {
            kind_id: relation_kind,
            kind_name: "relation".into(),
            schema_id,
            schema_version_id,
            cross_context_policy: CrossContextPolicy::AllowExplicit,
            cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
            aspect_contract_declarations: Default::default(),
            relation_integrity: Default::default(),
        })
        .unwrap();
    let schema = RelationalBranchRootSchemaAuthority::capture(
        1,
        &registry,
        &registry.authority_snapshot(),
        DescriptorSemanticsVersion::default(),
        None,
    )
    .unwrap();
    let mut symbols = StringInterner::default();
    let mut partition = PartitionState {
        partition_id,
        adjacency_policy: AdjacencyPolicy {
            backend: AdjacencyBackend::CompressedFanoutAdjacency,
            small_degree_inline_capacity: 4,
        },
        relation_overlay_is_sparse: false,
        entity_arena: EntityArena::with_capacity(3),
        relation_arena: RelationArena::with_capacity(1),
        adjacency: Default::default(),
        reverse_adjacency: Default::default(),
    };
    for (slot, label) in ["cold-first", "cold-target", "cold-other-target"]
        .into_iter()
        .enumerate()
    {
        partition
            .entity_arena
            .write_reserved_slot_for_test(
                SlotInit {
                    partition_id,
                    kind_id: entity_kind,
                    version_id: VersionId(1 + 4 * slot as u64),
                    extra: named_extra(&contract, &mut symbols, label),
                },
                slot,
                1,
            )
            .unwrap();
        partition
            .adjacency
            .ensure(slot, &partition.adjacency_policy);
        partition
            .reverse_adjacency
            .ensure(slot, &partition.adjacency_policy);
    }
    for (offset, label) in ["cold-second", "cold-third", "cold-fourth"]
        .into_iter()
        .enumerate()
    {
        partition
            .entity_arena
            .apply_extra_update_for_test(
                0,
                named_extra(&contract, &mut symbols, label),
                VersionId(2 + offset as u64),
            )
            .unwrap();
    }
    let mut previous_target = None;
    for (offset, slot) in [1, 2, 1].into_iter().enumerate() {
        let version = VersionId(10 + offset as u64);
        let target = EntityId::new(partition_id, slot as u64, 1);
        let extra = RelationExtra::from_unbudgeted_parts(
            Some(RelationEndpoints {
                source: entity,
                target,
            }),
            None,
        );
        if let Some(previous) = previous_target {
            partition
                .reverse_adjacency
                .get_mut(previous)
                .unwrap()
                .remove_adjacent_id_for_test(relation_kind, &relation);
            partition
                .relation_arena
                .apply_extra_update_for_test(0, extra, version)
                .unwrap();
        } else {
            partition
                .relation_arena
                .write_reserved_slot_for_test(
                    SlotInit {
                        partition_id,
                        kind_id: relation_kind,
                        version_id: version,
                        extra,
                    },
                    0,
                    1,
                )
                .unwrap();
            partition
                .adjacency
                .get_mut(0)
                .unwrap()
                .insert_adjacent_id_at_for_test(relation_kind, relation, version);
        }
        partition
            .reverse_adjacency
            .get_mut(slot)
            .unwrap()
            .insert_adjacent_id_at_for_test(relation_kind, relation, version);
        previous_target = Some(slot);
    }
    HistoryPartition {
        partition,
        entity,
        relation,
        schema,
        symbols,
    }
}

fn named_extra(contract: &AspectContract, symbols: &mut StringInterner, name: &str) -> EntityExtra {
    let fields = [
        ("classification", "native entity"),
        ("name", name),
        ("source_document", "checkpoint history"),
    ]
    .map(|(key, value)| {
        let symbol = symbols.intern(value).unwrap();
        (
            FieldKey::new(key).unwrap(),
            AspectValue::String(InternedString::Symbol(worth_foundational::facade::Symbol(
                symbol.0,
            ))),
        )
    });
    let value = StructAspectValue::from_sorted_fields(Box::new(fields)).unwrap();
    let TransitionOutcome::Success(value) =
        validate_aspect_value(contract, ContractValidationInput::Struct(value))
    else {
        panic!("valid string aspect")
    };
    let TransitionOutcome::Success(state) = admit_authoritative_record_aspect_state([value]) else {
        panic!("valid record aspect state")
    };
    EntityExtra::from_unbudgeted_parts(None, None, Some(state.into_parts().into_parts().0))
}

#[test]
fn partial_update_fixture_has_an_executable_lifecycle_binding() {
    let fixture = populated_partition();
    let mut registry = fixture.schema.registry().clone();
    let mut entity = (*registry.owner_entity_kinds_mut())
        .remove(&KindId(1))
        .unwrap();
    entity
        .aspect_contract_declarations
        .aspects
        .push(DeclaredAspectContractBinding {
            binding: AspectBinding::LifecycleTransition,
            contract: AspectContract::scalar(
                AspectKey::new("lifecycle").unwrap(),
                AspectIdentity(2),
                AspectContractRevision(1),
                ScalarAspectType::String,
            ),
        });
    let registry = registry.register_entity_kind(entity).unwrap();
    let catalog = crate::schema::aspect_plans::lower_aspect_plans(&registry);
    let plan = catalog.entity_plans.get(&KindId(1)).unwrap();
    assert!(plan.executable_bindings.len() >= 2);
    let untouched = plan
        .executable_bindings
        .iter()
        .find(|binding| matches!(binding.target, AspectBinding::LifecycleTransition))
        .unwrap();
    assert_eq!(untouched.aspect_key().as_str(), "lifecycle");
}
