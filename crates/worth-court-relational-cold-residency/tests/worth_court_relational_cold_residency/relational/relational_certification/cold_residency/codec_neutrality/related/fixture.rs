//! One real installed publication creates all three outputs and both relations.
use super::super::fixture::{self, *};
use super::super::publication_fixture;
use worth_foundational::facade::*;
use worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority;
use worth_relational::facade::identity::{EntityId, KindId, PartitionId, RelationId};
use worth_relational::facade::indexes::*;
use worth_relational::facade::mvcc::{RelationalPublicationOutcome, RelationalTransactionIntent};
use worth_relational::facade::runtime::{
    RelationalInitialSchemaInstallationReceipt, RelationalRuntime,
};
use worth_relational::facade::schema::*;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::*;

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
    registry_with_relation_integrity(version, RelationIntegrityDeclarations::default())
}

pub(super) fn registry_with_relation_integrity(
    version: u8,
    integrity: RelationIntegrityDeclarations,
) -> RelationalSchemaRegistry {
    let original = fixture::registry(version);
    let mut registry = RelationalSchemaRegistry::default();
    for id in [KindId(1), KindId(2)] {
        registry = registry
            .register_entity_kind(original.entity_registration(id).unwrap().clone())
            .unwrap();
    }
    let mut derived = original.entity_registration(KindId(3)).unwrap().clone();
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
            cross_context_policy: worth_relational::facade::config::CrossContextPolicy::Forbid,
            cascade_delete_policy:
                worth_relational::facade::config::CascadeDeletePolicy::CascadeDeleteRelations,
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
            relation_integrity: integrity,
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
    let (world, boundary, publisher) = build_world(execution, runtime, receipt, true);
    (world, boundary.map(|edge| (edge, publisher)))
}

pub(super) fn world_with_publication(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: RelationalRuntime,
    receipt: RelationalInitialSchemaInstallationReceipt,
) -> (RelatedWorld, ProducerPublication) {
    let (world, _, publisher) = build_world(execution, runtime, receipt, false);
    (world, publisher)
}

fn build_world(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: RelationalRuntime,
    receipt: RelationalInitialSchemaInstallationReceipt,
    boundary: bool,
) -> (RelatedWorld, Option<RelationId>, ProducerPublication) {
    let authored = create(execution, &runtime, 1, "authored", SOURCE_VALUE, None);
    let activation = create(execution, &runtime, 2, "activation", "program.v1", None);
    let (outputs, relations, edge, publisher) = if boundary {
        let (outputs, relations, edge, custody) = produce_with_boundary(
            execution,
            &runtime,
            &receipt,
            1,
            authored,
            "native.closed.producer",
        );
        (outputs, relations, Some(edge), custody)
    } else {
        let (outputs, relations, custody) =
            production::produce_with_publication(execution, &runtime, &receipt, 1);
        (outputs, relations, None, custody)
    };
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
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
        })
        .expect("index fixture has identity capacity");
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
        edge,
        publisher,
    )
}

pub(super) fn assert_order(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entities: &[EntityId; 3],
    relations: &[RelationId; 2],
) {
    let generation = runtime
        .index_access()
        .read_latest_generation(execution, ORDER, basis(runtime).descriptor().branch_id())
        .unwrap()
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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    outputs: &[EntityId; 3],
    expected: [&str; 3],
) {
    let selected = basis(runtime);
    let snapshot = runtime
        .snapshots()
        .snapshot_for_observation(&selected.observation())
        .unwrap();
    let view = runtime
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .unwrap()
        .unwrap();
    for (id, expected) in outputs.iter().zip(expected) {
        let record = view.get_entity(*id).unwrap();
        let value = record
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
    runtime.snapshots().release_snapshot(&snapshot).unwrap();
}

pub(super) fn recover(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &RelatedWorld,
) -> (
    RelationalRuntime,
    RecoveredRelationalRuntimeAuthority,
    RelationalInitialSchemaInstallationReceipt,
) {
    let (mut target, receipt) = installed(2);
    let (_, recovered) = crate::relational::relational_certification::cold_residency::codec_neutrality::fixture::restore_codec_transition(&mut target, execution, &capture(execution, &world.runtime))
        .unwrap();
    (target, recovered, receipt)
}

pub(super) fn candidate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    target: &mut RelationalRuntime,
    recovered: &mut RecoveredRelationalRuntimeAuthority,
    activation_id: EntityId,
    control: worth_relational::facade::mvcc::RelationalOperationControl,
) -> Result<
    worth_relational::facade::mvcc::PreparedRelationalCommitCandidate,
    worth_relational::facade::runtime::RelationalSchemaTransitionAdmissionDenial,
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
    let transaction = target
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
    Ok(target
        .prepare_branch_transaction(execution, transaction)
        .unwrap())
}
