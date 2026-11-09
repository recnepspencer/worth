//! Original-policy payload ownership; these fixtures issue no runtime authority.
use super::*;
mod historical_selection_funding;
mod wire;
use crate::identity::data::{KindId, PartitionId, VersionId};
use crate::storage::substrate::{
    EntityRecordKind, RecordArena, RecordKind, SharedColumn, SlotInit, StorageAllocation,
    StorageAllocationObservation, StorageAllocationVisitor,
};
use wire::*;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::*;
use worth_proof::TransitionOutcome;

fn request(budget: &SerialMemoryBudget) -> SerialRequest {
    SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None)
}
fn patch(text: &str) -> (AspectContract, AuthoritativeRecordAspectPatch) {
    let contract = AspectContract::scalar(
        AspectKey::new("data").unwrap(),
        AspectIdentity(1),
        AspectContractRevision(1),
        ScalarAspectType::String,
    );
    let TransitionOutcome::Success(value) = validate_aspect_value(
        &contract,
        ContractValidationInput::Scalar(AspectValue::String(InternedString::Raw(text.into()))),
    ) else {
        panic!("admitted source value")
    };
    let TransitionOutcome::Success(patch) =
        AuthoritativeRecordAspectPatch::whole_aspect([value], [])
    else {
        panic!("admitted source patch")
    };
    (contract, patch)
}
fn extra(request: &ExecutionRequest<'_, '_>, value: &str) -> EntityExtra {
    EntityExtra::apply_payload(request, None, &patch(value).1).unwrap()
}
fn expected(extra: &EntityExtra) -> u64 {
    let fixed = std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<EntityExtra>())
        .unwrap()
        .0
        .pad_to_align()
        .size();
    fixed as u64
        + extra
            .authoritative_aspect_state()
            .map_or(0, |state| state.owned_allocation_capacity_bytes() as u64)
}
fn arena(extra: EntityExtra) -> RecordArena<EntityRecordKind> {
    let mut arena = RecordArena::with_capacity(1);
    arena
        .write_reserved_slot(
            SlotInit {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                version_id: VersionId(1),
                extra,
            },
            0,
            1,
        )
        .unwrap();
    arena
}
#[derive(Default)]
struct Observed(Vec<StorageAllocationObservation>);
impl StorageAllocationVisitor for Observed {
    fn visit(&mut self, observation: StorageAllocationObservation) -> bool {
        self.0.push(observation);
        true
    }
}
fn check_delta(current: &RecordArena<EntityRecordKind>, prior: &RecordArena<EntityRecordKind>) {
    let mut introduced = Observed::default();
    let mut removed = Observed::default();
    current.visit_new_authoritative_allocations(prior, &mut introduced);
    prior.visit_new_authoritative_allocations(current, &mut removed);
    let sum = |observed: &Observed| observed.0.iter().map(|entry| entry.bytes).sum::<u64>();
    assert_eq!(
        prior.allocation_inventory().authoritative_bytes - sum(&removed) + sum(&introduced),
        current.allocation_inventory().authoritative_bytes
    );
}
#[test]
fn funded_extra_shares_actual_row_history_and_private_holds_until_last_drop() {
    let budget = SerialMemoryBudget::new(Some(64 * 1024));
    let host = request(&budget);
    let execution = ExecutionRequest::serial(&host);
    let value = extra(&execution, "original");
    let bill = expected(&value);
    let mut current = arena(value);
    let owner = current.retained_extra(0).unwrap();
    let historical = current.metadata_history_at(0).unwrap()[0].clone();
    assert_eq!(
        owner.id(),
        EntityRecordKind::metadata_extra(&historical).id()
    );
    let private = crate::authority::mutation::RecordMutation::EntityMaterializationSuspended {
        entity_id: crate::identity::data::EntityId::new(PartitionId::main(), 0, 1),
        kind_id: KindId(1),
        old_authoritative_aspect_state: Some(owner.clone()),
    };
    let mut observed = Observed::default();
    current.visit_authoritative_allocations(false, &mut observed);
    assert_eq!(
        observed
            .0
            .iter()
            .filter(|entry| entry.id == owner.id())
            .count(),
        1
    );
    assert_eq!(budget.charged_memory_bytes(), bill);
    let before = current.clone();
    current
        .apply_extra_update(0, extra(&execution, "updated"), VersionId(2))
        .unwrap();
    check_delta(&current, &before);
    // Nonconflicting updates in the same commit may have equal versions.
    let middle = current.clone();
    current
        .apply_extra_update(0, extra(&execution, "same-version"), VersionId(2))
        .unwrap();
    check_delta(&current, &middle);
    let no_op = current.clone();
    check_delta(&current, &no_op);
    let before_removal = current.clone();
    current
        .suspend_materialization_with_extra(0, EntityExtra::empty_for_request(&execution).unwrap())
        .unwrap();
    check_delta(&current, &before_removal);
    drop((
        current,
        before,
        middle,
        no_op,
        before_removal,
        observed,
        owner,
    ));
    assert_eq!(budget.charged_memory_bytes(), bill);
    drop(historical);
    assert_eq!(budget.charged_memory_bytes(), bill);
    drop(private);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
#[test]
fn funded_extra_refuses_copy_and_replacement_before_growth() {
    let original = SerialMemoryBudget::new(Some(128 * 1024));
    let host = request(&original);
    let execution = ExecutionRequest::serial(&host);
    let old = extra(&execution, &"large".repeat(2048));
    let old_bill = expected(&old);
    let limited = SerialMemoryBudget::new(Some(0));
    let limited_host = request(&limited);
    let limited_request = ExecutionRequest::serial(&limited_host);
    assert!(matches!(
        EntityExtra::apply_payload(
            &limited_request,
            old.authoritative_aspect_state(),
            &patch("new").1
        ),
        Err(ExtraPayloadDenial::Request(_))
    ));
    assert!(old.copy_for_request(&limited_request).is_err());
    assert_eq!(original.charged_memory_bytes(), old_bill);
    assert_eq!(limited.charged_memory_bytes(), 0);
    let replacement_budget = SerialMemoryBudget::new(Some(4096));
    let replacement_host = request(&replacement_budget);
    let replacement_request = ExecutionRequest::serial(&replacement_host);
    let replacement = EntityExtra::apply_payload(
        &replacement_request,
        old.authoritative_aspect_state(),
        &patch("ok").1,
    )
    .unwrap();
    assert_eq!(
        replacement_budget.charged_memory_bytes(),
        expected(&replacement)
    );
    assert!(replacement_budget.charged_memory_bytes() < 1024);
    // An ordinary deep copy owns independent allocations and no old ticket.
    let independent = replacement.clone();
    drop(replacement);
    assert_eq!(replacement_budget.charged_memory_bytes(), 0);
    assert!(independent.authoritative_aspect_state().is_some());
    drop(old);
    assert_eq!(original.charged_memory_bytes(), 0);
}
#[test]
fn unique_extra_move_and_borrowed_export_preserve_original_funding() {
    let budget = SerialMemoryBudget::new(Some(16 * 1024));
    let host = request(&budget);
    let execution = ExecutionRequest::serial(&host);
    let value = extra(&execution, "move-and-export");
    let bill = expected(&value);
    let mut column =
        SharedColumn::<crate::storage::substrate::StorageAllocation<super::EntityExtra>>::default();
    column.push(value);
    let moved = column.into_vec();
    assert_eq!(
        budget.charged_memory_bytes(),
        bill,
        "the whole unique extra moved, including its footer"
    );
    let catalog = crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog::from_contracts(&[patch("unused").0]).unwrap();
    let exported = crate::durability::checkpoints::aspect_state_images::export_state(
        moved[0].authoritative_aspect_state(),
        Some(&catalog),
    )
    .unwrap();
    assert!(exported.is_some());
    assert_eq!(budget.charged_memory_bytes(), bill);
    drop(moved);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
#[test]
fn extra_and_metadata_wire_preserve_legacy_named_and_compact_fields() {
    eprintln!(
        "target inline bytes old/new: EntityExtra={}/{}, RelationExtra={}/{}, EntityMeta={}/{}, RelationMeta={}/{}",
        std::mem::size_of::<LegacyEntity<AuthoritativeRecordAspectState>>(),
        std::mem::size_of::<EntityExtra>(),
        std::mem::size_of::<LegacyRelation<RelationEndpoints, AuthoritativeRecordAspectState>>(),
        std::mem::size_of::<RelationExtra>(),
        std::mem::size_of::<LegacyMeta<AuthoritativeRecordAspectState>>(),
        std::mem::size_of::<VersionedEntityMetadata>(),
        std::mem::size_of::<LegacyRelationMeta<RelationEndpoints, AuthoritativeRecordAspectState>>(),
        std::mem::size_of::<VersionedRelationMetadata>(),
    );
    let budget = SerialMemoryBudget::new(Some(32 * 1024));
    let host = request(&budget);
    let execution = ExecutionRequest::serial(&host);
    for value in [
        EntityExtra::empty_for_request(&execution).unwrap(),
        extra(&execution, "wire"),
    ] {
        let actual = StorageAllocation::new(value);
        equal(
            actual.as_ref(),
            &LegacyEntity {
                structural_fingerprint: actual.structural_fingerprint,
                lineage_id: actual.lineage_id,
                authoritative_aspect_state: actual.authoritative_aspect_state(),
            },
        );
        let meta =
            EntityRecordKind::metadata_for_create(KindId(1), 1, VersionId(2), actual.clone())
                .unwrap();
        equal(
            &meta,
            &LegacyMeta {
                effective_at: meta.effective_at,
                retired_at: meta.retired_at,
                generation: meta.generation,
                kind_id: meta.kind_id,
                lineage_id: meta.lineage_id,
                authoritative_aspect_state: actual.authoritative_aspect_state(),
            },
        );
        let mut relation = if actual.authoritative_aspect_state().is_some() {
            RelationExtra::apply_payload(&execution, None, &patch("wire").1).unwrap()
        } else {
            RelationExtra::empty_for_request(&execution).unwrap()
        };
        relation.endpoints = Some(RelationEndpoints {
            source: crate::identity::data::EntityId::new(PartitionId::main(), 0, 1),
            target: crate::identity::data::EntityId::new(PartitionId::main(), 1, 1),
        });
        let relation = StorageAllocation::new(relation);
        equal(
            relation.as_ref(),
            &LegacyRelation {
                endpoints: relation.endpoints.as_ref(),
                authoritative_aspect_state: relation.authoritative_aspect_state(),
            },
        );
        let meta = crate::storage::substrate::RelationRecordKind::metadata_for_create(
            KindId(2),
            1,
            VersionId(2),
            relation.clone(),
        )
        .unwrap();
        equal(
            &meta,
            &LegacyRelationMeta {
                effective_at: meta.effective_at,
                retired_at: meta.retired_at,
                generation: meta.generation,
                kind_id: meta.kind_id,
                endpoints: &meta.endpoints,
                authoritative_aspect_state: relation.authoritative_aspect_state(),
            },
        );
        assert!(!format!("{actual:?}").contains("_funding"));
    }
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn escaped_extra_carrier_keeps_only_its_own_bill_after_node_drop() {
    let payload_budget = SerialMemoryBudget::new(Some(65536));
    let payload_host = request(&payload_budget);
    let payload_request = ExecutionRequest::serial(&payload_host);
    let value = extra(&payload_request, "independent escaped extra");
    let bill = expected(&value);
    let node_budget = SerialMemoryBudget::new(Some(65536));
    let node_host = request(&node_budget);
    let node_request = ExecutionRequest::serial(&node_host);
    let mut column = SharedColumn::<StorageAllocation<EntityExtra>>::default();
    column
        .push_shared_for_request(StorageAllocation::new(value), &node_request)
        .unwrap();
    let escaped = column.retained_value(0).unwrap();
    assert_eq!(payload_budget.charged_memory_bytes(), bill);
    assert!(node_budget.charged_memory_bytes() > 0);
    drop(column);
    assert_eq!(node_budget.charged_memory_bytes(), 0);
    assert_eq!(payload_budget.charged_memory_bytes(), bill);
    drop(escaped);
    assert_eq!(payload_budget.charged_memory_bytes(), 0);
}
