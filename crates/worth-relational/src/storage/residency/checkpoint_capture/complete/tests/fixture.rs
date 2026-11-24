//! Produce declared scalar and struct state plus a real multi-version relation.
use super::*;
use crate::config::data::{
    AdjacencyBackend, AdjacencyPolicy, CascadeDeletePolicy, CrossContextPolicy,
};
use crate::identity::data::{RelationId, VersionId};
use crate::schema::data::{
    DeclaredAspectContractBinding, EntityKindRegistration, KindAspectContractDeclarations,
    RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::{
    EntityArena, EntityExtra, RelationArena, RelationEndpoints, RelationExtra, SlotInit,
};
use worth_foundational::facade::{
    admit_authoritative_record_aspect_state, validate_aspect_value, AbsenceLaw, AspectContract,
    AspectContractRevision, AspectEvolutionPolicy, AspectIdentity, AspectKey, AspectValue,
    ContractValidationInput, FieldDeclaration, FieldKey, FieldRequirement, ScalarAspectType,
    StructAspectShape, StructAspectValue,
};
use worth_proof::TransitionOutcome;

pub(super) fn original_partition() -> (
    PartitionState,
    Arc<crate::branch::root::schema::RelationalBranchRootSchemaAuthority>,
) {
    let field_key = |field: &str| FieldKey::new(field).unwrap();
    let name = AspectContract::scalar(
        AspectKey::new("name").unwrap(),
        AspectIdentity(1),
        AspectContractRevision(1),
        ScalarAspectType::String,
    );
    let summary_shape = StructAspectShape::new([
        FieldDeclaration::new(
            field_key("status"),
            ScalarAspectType::String,
            FieldRequirement::Optional,
            AbsenceLaw::Optional,
            AspectEvolutionPolicy::AdditiveFieldsAllowed,
        )
        .unwrap(),
        FieldDeclaration::new(
            field_key("title"),
            ScalarAspectType::String,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            AspectEvolutionPolicy::AdditiveFieldsAllowed,
        )
        .unwrap(),
    ])
    .unwrap();
    let summary = AspectContract::struct_aspect(
        AspectKey::new("summary").unwrap(),
        AspectIdentity(2),
        AspectContractRevision(1),
        summary_shape,
    );
    let entity_kind = KindId(1);
    let relation_kind = KindId(2);
    let schema_id = SchemaId("complete-wire-fixture".into());
    let schema_version_id = SchemaVersionId(1);
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: entity_kind,
            kind_name: "entity".into(),
            schema_id: schema_id.clone(),
            schema_version_id,
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                DeclaredAspectContractBinding {
                    binding: crate::schema::data::AspectBinding::EntityField {
                        field: field_key("name"),
                    },
                    contract: name.clone(),
                },
                DeclaredAspectContractBinding {
                    binding: crate::schema::data::AspectBinding::EntityField {
                        field: field_key("summary"),
                    },
                    contract: summary.clone(),
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
    let partition_id = PartitionId::main();
    let mut partition = PartitionState {
        partition_id,
        adjacency_policy: AdjacencyPolicy {
            backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
            small_degree_inline_capacity: 4,
        },
        relation_overlay_is_sparse: false,
        entity_arena: EntityArena::with_capacity(3),
        relation_arena: RelationArena::with_capacity(1),
        adjacency: Default::default(),
        reverse_adjacency: Default::default(),
    };
    for (slot, label) in ["original-source", "original-target", "original-later"]
        .into_iter()
        .enumerate()
    {
        let authored_name = ContractValidationInput::Scalar(AspectValue::String(label.into()));
        let authored_summary = ContractValidationInput::Struct(
            StructAspectValue::from_sorted_fields(
                vec![
                    (field_key("status"), AspectValue::String("open".into())),
                    (
                        field_key("title"),
                        AspectValue::String(format!("title-{label}").into()),
                    ),
                ]
                .into_boxed_slice(),
            )
            .unwrap(),
        );
        let values =
            [(&name, authored_name), (&summary, authored_summary)].map(|(contract, value)| {
                let TransitionOutcome::Success(admitted) = validate_aspect_value(contract, value)
                else {
                    panic!("valid declared aspect")
                };
                admitted
            });
        let TransitionOutcome::Success(state) = admit_authoritative_record_aspect_state(values)
        else {
            panic!("valid declared record state")
        };
        partition
            .entity_arena
            .write_reserved_slot(
                SlotInit {
                    partition_id,
                    kind_id: entity_kind,
                    version_id: VersionId(slot as u64 + 1),
                    extra: EntityExtra::from_unbudgeted_parts(
                        None,
                        None,
                        Some(state.into_parts().into_parts().0),
                    ),
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
    let source = EntityId::new(partition_id, 0, 1);
    let relation = RelationId::new(partition_id, 0, 1);
    let mut previous_target = None;
    for (offset, slot) in [1, 2, 1, 2].into_iter().enumerate() {
        let version = VersionId(4 + offset as u64);
        let target = EntityId::new(partition_id, slot as u64, 1);
        let extra =
            RelationExtra::from_unbudgeted_parts(Some(RelationEndpoints { source, target }), None);
        if let Some(previous) = previous_target {
            partition
                .reverse_adjacency
                .get_mut(previous)
                .unwrap()
                .remove(relation_kind, &relation);
            partition
                .relation_arena
                .apply_extra_update(0, extra, version)
                .unwrap();
        } else {
            partition
                .relation_arena
                .write_reserved_slot(
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
                .insert_at(relation_kind, relation, version);
        }
        partition
            .reverse_adjacency
            .get_mut(slot)
            .unwrap()
            .insert_at(relation_kind, relation, version);
        previous_target = Some(slot);
    }
    assert!(
        partition
            .relation_arena
            .metadata_history_at(relation.slot_index())
            .unwrap()
            .len()
            > 2
    );
    let schema = crate::branch::root::schema::RelationalBranchRootSchemaAuthority::capture(
        1,
        &registry,
        &registry.authority_snapshot(),
        crate::schema::data::DescriptorSemanticsVersion::default(),
        None,
    )
    .unwrap();
    (partition, schema)
}
