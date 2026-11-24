//! Actual allocation bindings and retained metadata for relation-birth readmission.
use crate::branch::basis_admission_tests::observed_empty_branch;
use crate::history::data::canonical_commit_envelope::required_output_origin::fixture_birth::describe_birth;
use crate::history::data::canonical_commit_envelope::tests::envelope_with_patch_at;
use crate::history::data::{
    BranchId, CanonicalCommitEnvelope, CanonicalCommitEnvelopeBody, CanonicalRecordAllocation,
    CanonicalRequiredOutputRelation, CommitId, RelationalCommitReceipt,
};
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId, VersionBound, VersionId};
use crate::publication::patch::data::{
    PatchDetail, PublishedAuthoritativePatch, PublishedAuthoritativePatchOperation,
    PublishedAuthoritativePatchValue, PublishedAuthoritativeRecordPatch, PublishedRelationBirth,
    RecordStructuralChange,
};
use crate::schema::data::{
    DeclaredAspectContractBinding, KindAspectContractDeclarations, RelationKindRegistration,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use crate::storage::overlay::{small_relation_partition::relation_partition, PartitionState};
use crate::storage::substrate::{HistoricalMetadata, RelationEndpoints, RelationExtra, SlotInit};
use crate::symbols::data::{ClientKey, StringInterner};
use crate::transactions::data::outcomes::{
    created_entity_bindings::CommitCreatedEntityBindings,
    created_relation_bindings::CommitCreatedRelationBindings,
};
use crate::transactions::data::primitives::{
    CreatedEntityRef, CreatedRelationRef, EntityReference, RecordRef,
};
use std::{collections::BTreeMap, sync::Arc};
use worth_foundational::facade::*;
use worth_proof::TransitionOutcome;

pub(super) struct CommittedImages {
    pub(super) edition: BTreeMap<PartitionId, PartitionState>,
    pub(super) id: RelationId,
    pub(super) owned_id: RelationId,
    pub(super) output_id: EntityId,
    pub(super) right: EntityId,
    pub(super) birth: Arc<CanonicalCommitEnvelope>,
    pub(super) write: Arc<CanonicalCommitEnvelope>,
}

pub(super) struct BirthRelationInputs<'a> {
    pub(super) body: &'a CanonicalCommitEnvelopeBody,
    pub(super) allocated_entities: &'a CommitCreatedEntityBindings,
    pub(super) allocated_relations: &'a CommitCreatedRelationBindings,
    pub(super) edition: &'a BTreeMap<PartitionId, PartitionState>,
    pub(super) output_id: EntityId,
    pub(super) registry: &'a RelationalSchemaRegistry,
}

pub(super) fn committed_images(
    capture: impl FnOnce(BirthRelationInputs<'_>) -> Vec<CanonicalRequiredOutputRelation>,
) -> CommittedImages {
    let partition_id = PartitionId::main();
    let (mut partition, entities, _) = relation_partition(partition_id, 3, &[]);
    let [left, right, output_id] = entities[..] else {
        unreachable!("the partition has two external endpoints and one output");
    };
    let birth_version = VersionId(3);
    let write_version = VersionId(4);
    let source_contract = endpoint_contract("source", 1, AspectBinding::RelationSourceEndpoint);
    let target_contract = endpoint_contract("target", 2, AspectBinding::RelationTargetEndpoint);
    let insert = |partition: &mut PartitionState, source, target| {
        let (slot, generation, _) = partition.relation_arena.push_slot(SlotInit {
            partition_id,
            kind_id: KindId(2),
            version_id: birth_version,
            extra: relation_extra(&source_contract, &target_contract, source, target),
        });
        RelationId::new(partition_id, slot as u64, generation)
    };
    let id = insert(&mut partition, left, right);
    let owned_id = insert(&mut partition, output_id, left);
    let output = CreatedEntityRef {
        partition_id,
        kind_id: KindId(1),
        client_key: ClientKey::raw("output"),
    };
    let incidental = CreatedRelationRef {
        partition_id,
        kind_id: KindId(2),
        client_key: ClientKey::raw("incidental"),
        source: EntityReference::Existing(left),
        target: EntityReference::Existing(right),
    };
    let owned = CreatedRelationRef {
        client_key: ClientKey::raw("owned"),
        source: EntityReference::Created(output.clone()),
        target: EntityReference::Existing(left),
        ..incidental.clone()
    };
    let symbols = StringInterner::default();
    let allocated_entities = CommitCreatedEntityBindings::from_owner_map(
        BTreeMap::from([(output, output_id)]),
        &symbols,
    );
    let allocated_relations = CommitCreatedRelationBindings::from_owner_map(
        BTreeMap::from([(incidental, id), (owned, owned_id)]),
        &symbols,
    );
    let registry = RelationalSchemaRegistry::new()
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(2),
            kind_name: "test.relation".into(),
            schema_id: SchemaId("test".into()),
            schema_version_id: SchemaVersionId(1),
            cross_context_policy: crate::config::data::CrossContextPolicy::AllowExplicit,
            cascade_delete_policy: crate::config::data::CascadeDeletePolicy::CascadeDeleteRelations,
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                source_contract.clone(),
                target_contract.clone(),
            ]),
            relation_integrity: Default::default(),
        })
        .unwrap();
    let (history, basis) = observed_empty_branch(1);
    let branch = BranchId("main".into());
    let receipt = |version: VersionId| RelationalCommitReceipt {
        commit_id: CommitId(version.0),
        version_id: version,
        branch_id: branch.clone(),
        parents: Vec::new(),
    };
    let mut birth =
        envelope_with_patch_at(RecordRef::Relation(id), output_id, receipt(birth_version));
    let body = birth.body_mut_for_test();
    body.schema_authority = registry.authority_snapshot();
    body.branch_cell_checkpoint = Some(
        history
            .branch_cell(&branch)
            .unwrap()
            .checkpoint()
            .in_canonical_frame()
            .unwrap(),
    );
    body.patch.authoritative_record_patches = [(id, left, right), (owned_id, output_id, left)]
        .into_iter()
        .map(|(id, source, target)| {
            birth_patch(id, source, target, &source_contract, &target_contract)
        })
        .collect();
    body.install_record_allocations(
        [
            RecordRef::Entity(output_id),
            RecordRef::Relation(id),
            RecordRef::Relation(owned_id),
        ]
        .into_iter()
        .enumerate()
        .map(|(ordinal, record)| CanonicalRecordAllocation::new(ordinal as u64, record))
        .collect(),
    );
    let mut edition = BTreeMap::from([(partition_id, partition)]);
    let relations = capture(BirthRelationInputs {
        body,
        allocated_entities: &allocated_entities,
        allocated_relations: &allocated_relations,
        edition: &edition,
        output_id,
        registry: &registry,
    });
    body.required_output_origin = Some(Arc::new(describe_birth(
        body.commit.clone(),
        basis.descriptor().in_canonical_frame().unwrap(),
        output_id,
        relations,
    )));
    let partition = edition.get_mut(&partition_id).unwrap();
    partition
        .relation_arena
        .apply_extra_update(
            id.slot_index(),
            relation_extra(&source_contract, &target_contract, output_id, right),
            write_version,
        )
        .unwrap();
    // Retain only metadata that the same history fence keeps in production.
    let fence = VersionBound::new(write_version);
    partition
        .relation_arena
        .metadata_history_at_mut(id.slot_index())
        .unwrap()
        .retain(|entry| {
            entry
                .retired_at()
                .is_none_or(|retired| fence.retains_retired(retired))
        });
    let write = envelope_with_patch_at(RecordRef::Relation(id), output_id, receipt(write_version));
    CommittedImages {
        edition,
        id,
        owned_id,
        output_id,
        right,
        birth: Arc::new(birth),
        write: Arc::new(write),
    }
}

fn endpoint_contract(
    key: &str,
    identity: u64,
    binding: AspectBinding,
) -> DeclaredAspectContractBinding {
    DeclaredAspectContractBinding {
        binding,
        contract: AspectContract::reference_entity(
            AspectKey::new(key).unwrap(),
            AspectIdentity(identity),
            AspectContractRevision(1),
        ),
    }
}

fn endpoint_value(entity: EntityId) -> AspectValue {
    AspectValue::EntityRef(worth_foundational::facade::EntityId::new(
        worth_foundational::facade::PartitionId(entity.partition_id.0),
        entity.local_slot_value(),
        entity.generation_value(),
    ))
}

fn relation_extra(
    source_contract: &DeclaredAspectContractBinding,
    target_contract: &DeclaredAspectContractBinding,
    source: EntityId,
    target: EntityId,
) -> RelationExtra {
    let admitted =
        [(source_contract, source), (target_contract, target)].map(|(binding, entity)| {
            let value = ContractValidationInput::Scalar(endpoint_value(entity));
            let TransitionOutcome::Success(value) = validate_aspect_value(&binding.contract, value)
            else {
                panic!("the constructed endpoint is within the declared contract");
            };
            value
        });
    let TransitionOutcome::Success(patch) =
        AuthoritativeRecordAspectPatch::whole_aspect(admitted, [])
    else {
        panic!("each endpoint has its own binding");
    };
    let TransitionOutcome::Success(state) = patch.apply_to_optional(None) else {
        panic!("both endpoint values are admitted");
    };
    RelationExtra::from_unbudgeted_parts(
        Some(RelationEndpoints { source, target }),
        state.map(|state| state.into_parts().into_parts().0),
    )
}

fn birth_patch(
    id: RelationId,
    source: EntityId,
    target: EntityId,
    source_contract: &DeclaredAspectContractBinding,
    target_contract: &DeclaredAspectContractBinding,
) -> PublishedAuthoritativeRecordPatch {
    let operations =
        [(source_contract, source), (target_contract, target)].map(|(declared, id)| {
            PublishedAuthoritativePatchOperation::WholeAspectSet {
                aspect_key: declared.contract.key().clone(),
                aspect_identity: declared.contract.identity(),
                contract_revision: declared.contract.revision(),
                binding: declared.binding.clone(),
                value: PublishedAuthoritativePatchValue::Scalar(endpoint_value(id)),
            }
        });
    PublishedAuthoritativeRecordPatch {
        target: RecordRef::Relation(id),
        structural_change: RecordStructuralChange::Created,
        authoritative_patch: PublishedAuthoritativePatch::new(operations.to_vec()),
        semantic_changes: Vec::new(),
        contains_opaque_aspect: false,
        detail: PatchDetail::DenseBitset(vec![1]),
        relation_birth: Some(PublishedRelationBirth {
            kind: KindId(2),
            source,
            target,
        }),
    }
}
