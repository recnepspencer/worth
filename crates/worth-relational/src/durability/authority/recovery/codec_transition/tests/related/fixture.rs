//! One real installed publication creates all three outputs and both relations.
use super::super::fixture::{self, *};
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::indexes::data::*;
use crate::mvcc::{RelationalPublicationOutcome, RelationalTransactionIntent};
use crate::runtime::{RelationalInitialSchemaInstallationReceipt, RelationalRuntime};
use crate::schema::data::*;
use crate::symbols::data::ClientKey;
use crate::transactions::data::*;
use worth_foundational::facade::*;

mod production;
pub(super) use production::{
    produce, produce_with_boundary, produce_with_incidental, ProducerPublication,
};

pub(super) const ORDER: DerivedIndexId = DerivedIndexId(1);
pub(super) struct RelatedWorld {
    pub(super) runtime: RelationalRuntime,
    pub(super) authored: EntityId,
    pub(super) activation: EntityId,
    pub(super) outputs: [EntityId; 3],
    pub(super) relations: [RelationId; 2],
}

pub(super) fn registry(version: u8) -> RelationalSchemaRegistry {
    let mut registry = fixture::registry(version);
    let mut derived = registry.entity_kinds.remove(&KindId(3)).unwrap();
    derived.aspect_contract_declarations.aspects.push(binding(
        "order",
        4,
        AspectBinding::EntityField {
            field: FieldKey::new("value").unwrap(),
        },
    ));
    registry
        .register_entity_kind(derived)
        .unwrap()
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(4),
            kind_name: "derived-child".into(),
            schema_id: SchemaId("native-codec-world".into()),
            schema_version_id: SchemaVersionId(1),
            cross_context_policy: crate::config::data::CrossContextPolicy::Forbid,
            cascade_delete_policy: crate::config::data::CascadeDeletePolicy::CascadeDeleteRelations,
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                binding(
                    "edge",
                    5,
                    AspectBinding::RelationField {
                        field: FieldKey::new("value").unwrap(),
                    },
                ),
                endpoint("edge-source", 6, AspectBinding::RelationSourceEndpoint),
                endpoint("edge-target", 7, AspectBinding::RelationTargetEndpoint),
            ]),
            relation_integrity: RelationIntegrityDeclarations::default(),
        })
        .unwrap()
}

fn binding(key: &str, id: u64, binding: AspectBinding) -> DeclaredAspectContractBinding {
    DeclaredAspectContractBinding {
        binding,
        contract: AspectContract::struct_aspect(
            AspectKey::new(key).unwrap(),
            AspectIdentity(id),
            AspectContractRevision(1),
            StructAspectShape::new([FieldDeclaration::new(
                FieldKey::new("value").unwrap(),
                ScalarAspectType::String,
                FieldRequirement::Required,
                AbsenceLaw::Required,
                AspectEvolutionPolicy::AdditiveFieldsAllowed,
            )
            .unwrap()])
            .unwrap(),
        ),
    }
}

fn endpoint(key: &str, id: u64, binding: AspectBinding) -> DeclaredAspectContractBinding {
    DeclaredAspectContractBinding {
        binding,
        contract: AspectContract::reference_entity(
            AspectKey::new(key).unwrap(),
            AspectIdentity(id),
            AspectContractRevision(1),
        ),
    }
}

pub(super) fn installed(
    version: u8,
) -> (
    RelationalRuntime,
    RelationalInitialSchemaInstallationReceipt,
) {
    installed_with(registry(version), Vec::new())
}

pub(super) fn world(execution: &worth_execution::ExecutionRequest<'_, '_>) -> RelatedWorld {
    let (runtime, receipt) = installed(1);
    world_with_installation(execution, runtime, receipt)
}

pub(super) fn world_with_installation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: RelationalRuntime,
    receipt: RelationalInitialSchemaInstallationReceipt,
) -> RelatedWorld {
    build_world(execution, runtime, receipt, false).0
}

pub(super) fn world_with_boundary(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: RelationalRuntime,
    receipt: RelationalInitialSchemaInstallationReceipt,
) -> (RelatedWorld, Option<(RelationId, ProducerPublication)>) {
    build_world(execution, runtime, receipt, true)
}

fn build_world(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: RelationalRuntime,
    receipt: RelationalInitialSchemaInstallationReceipt,
    boundary: bool,
) -> (RelatedWorld, Option<(RelationId, ProducerPublication)>) {
    let authored = create(execution, &runtime, 1, "authored", SOURCE_VALUE, None);
    let activation = create(execution, &runtime, 2, "activation", "program.v1", None);
    let (outputs, relations, owned) = if boundary {
        let (outputs, relations, edge, custody) = produce_with_boundary(
            execution,
            &runtime,
            &receipt,
            1,
            authored,
            "native.closed.producer",
        );
        (outputs, relations, Some((edge, custody)))
    } else {
        let (outputs, relations) = produce(execution, &runtime, &receipt, 1);
        (outputs, relations, None)
    };
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: ORDER,
        name: "unchanged ordered derived children".into(),
        branch_scoped: true,
        kind: DerivedIndexKind::RelatedEntityOrdering {
            relation_kind: KindId(4),
            parent_endpoint: RelatedEntityEndpoint::SourceParent,
            child_kind: KindId(3),
            ordering: vec![RelatedEntityOrderingField::new(
                locator("order"),
                RelatedEntityOrderingDirection::Ascending,
            )],
        },
    });
    assert_eq!(index.index_id, ORDER);
    build_index(execution, &runtime);
    (
        RelatedWorld {
            runtime,
            authored,
            activation,
            outputs,
            relations,
        },
        owned,
    )
}

pub(super) fn assert_order(
    runtime: &RelationalRuntime,
    entities: &[EntityId; 3],
    relations: &[RelationId; 2],
) {
    let generation = runtime
        .indexes
        .latest_generation(ORDER, Some(basis(runtime).descriptor().branch_id()))
        .unwrap();
    let DerivedIndexEntries::RelatedEntityOrdering(entries) = &generation.entries else {
        panic!("actual ordering generation")
    };
    let rows = entries.get(&entities[0]).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows.iter()
            .map(|row| row.child_entity_id())
            .collect::<Vec<_>>(),
        [entities[2], entities[1]]
    );
    assert_eq!(
        rows.iter().map(|row| row.relation_id()).collect::<Vec<_>>(),
        [relations[1], relations[0]]
    );
    assert_eq!(
        rows.iter()
            .map(|row| row.ordering_values()[0].value().clone())
            .collect::<Vec<_>>(),
        [
            AspectValue::String(InternedString::from("a-left")),
            AspectValue::String(InternedString::from("z-right"))
        ]
    );
}

pub(super) fn assert_values(
    runtime: &RelationalRuntime,
    outputs: &[EntityId; 3],
    expected: [&str; 3],
) {
    let selected = basis(runtime);
    for (id, expected) in outputs.iter().zip(expected) {
        let record = selected
            .inner
            .root
            .partition_state(id.partition_id)
            .unwrap()
            .entity_arena
            .get(id)
            .unwrap();
        let value = record
            .extra()
            .authoritative_aspect_state
            .as_ref()
            .unwrap()
            .get(&AspectKey::new("derived").unwrap())
            .unwrap();
        let ContractValidatedAspectValueView::Struct(value) = value.view() else {
            panic!("actual native derived field is a struct carrier")
        };
        assert_eq!(
            value.get(&FieldKey::new("value").unwrap()),
            Some(&AspectValue::String(InternedString::from(expected)))
        );
    }
}

pub(super) fn relation_payload(runtime: &RelationalRuntime, id: RelationId) -> Vec<u8> {
    let basis = basis(runtime);
    let record = basis
        .inner
        .root
        .partition_state(id.partition_id)
        .unwrap()
        .relation_arena
        .get(&id)
        .unwrap();
    rmp_serde::to_vec(&(
        id,
        record.kind_id(),
        record.generation(),
        &record.extra().endpoints,
        &record.extra().authoritative_aspect_state,
    ))
    .unwrap()
}

pub(super) fn recover(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &RelatedWorld,
) -> (
    RelationalRuntime,
    crate::durability::authority::RecoveredRelationalRuntimeAuthority,
    RelationalInitialSchemaInstallationReceipt,
) {
    let (mut target, receipt) = installed(2);
    let (_, recovered) = target
        .durability_recovery()
        .restore_native_checkpoint_for_codec_transition_with_authority(
            execution,
            &capture(&world.runtime),
        )
        .unwrap();
    (target, recovered, receipt)
}

pub(super) fn candidate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    target: &mut RelationalRuntime,
    recovered: &mut crate::durability::authority::RecoveredRelationalRuntimeAuthority,
    activation_id: EntityId,
    control: crate::mvcc::RelationalOperationControl,
    extra: Option<WorkerIntentBatch>,
) -> Result<
    crate::mvcc::PreparedRelationalCommitCandidate,
    crate::runtime::RelationalSchemaTransitionAdmissionDenial,
> {
    let selected = basis(target);
    let (owner, installed) = target
        .durability_recovery()
        .issue_recovered_codec_source_publication_owner(execution, recovered)
        .unwrap();
    let proof = activation_proof(
        execution,
        target,
        recovered,
        &selected,
        &owner,
        activation_id,
    );
    let mut transaction = target
        .durability_recovery()
        .begin_recovered_codec_transition(
            execution,
            recovered,
            &selected,
            control,
            &installed,
            proof,
            activation(activation_id),
        )?;
    if let Some(batch) = extra {
        transaction
            .push_batch(batch, execution.allocation_policy())
            .map_err(crate::runtime::RelationalSchemaTransitionAdmissionDenial::staging)?;
    }
    Ok(target
        .prepare_branch_transaction(execution, transaction)
        .unwrap())
}
