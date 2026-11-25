//! Actual control allocations, independently measured, including final Weak.
use super::*;
use crate::config::data::{AdjacencyBackend, AdjacencyPolicy};
use crate::identity::data::PartitionId;
use crate::identity::data::{KindId, VersionId};
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use crate::storage::overlay::PartitionMutationJournal;
use crate::storage::residency::{PreparedResidentRegion, RegionCaptureBasis};
use crate::storage::substrate::{EntityArena, EntityExtra, RelationArena, SlotInit};
use crate::symbols::data::StringInterner;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::{
    admit_authoritative_record_aspect_state, validate_aspect_value, AspectContract,
    AspectContractRevision, AspectIdentity, AspectKey, AspectValue, ContractValidationInput,
    InternedString, ScalarAspectType,
};
use worth_proof::TransitionOutcome;

fn partition() -> PartitionState {
    PartitionState {
        partition_id: PartitionId(1),
        adjacency_policy: crate::config::data::AdjacencyPolicy {
            backend: crate::config::data::AdjacencyBackend::InlineSmallDegreeAdjacency,
            small_degree_inline_capacity: 4,
        },
        relation_overlay_is_sparse: false,
        entity_arena: EntityArena::with_capacity(0),
        relation_arena: RelationArena::with_capacity(0),
        adjacency: Default::default(),
        reverse_adjacency: Default::default(),
    }
}

/// The production constructor's actual kind projection for this empty partition.
fn kind_indexes() -> Arc<crate::storage::residency::kind_incremental::RegionKindIndexes> {
    crate::storage::residency::resident_history::with_reconstruction_fixture(|fixture_lane| {
        Arc::new(
            crate::storage::residency::kind_incremental::RegionKindIndexes::capture(
                fixture_lane,
                &partition(),
            )
            .unwrap(),
        )
    })
}

fn allocation<T>() -> u64 {
    let layout = std::alloc::Layout::new::<(usize, usize)>()
        .extend(std::alloc::Layout::new::<T>())
        .unwrap()
        .0
        .pad_to_align();
    u64::try_from(layout.size()).unwrap()
}

#[test]
fn controlled_resident_cell_keeps_exact_charge_until_last_real_weak() {
    let budget = SerialMemoryBudget::new(Some(4 * 1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let region = allocation::<ResidentRegion>();
    let control =
        allocation::<RelationalRegionResidency>() + allocation::<ExecutionMemoryReservation>();
    let lease = RelationalRegionResidency::resident_for_request(
        partition(),
        PartitionContentCommitment::default(),
        kind_indexes(),
        None,
        &ExecutionRequest::serial(&host),
    )
    .unwrap();
    assert_eq!(budget.charged_memory_bytes(), region + control);
    let weak = RelationalRegionResidency::downgrade(lease.cell());
    let remaining = weak.clone();
    drop(lease);
    assert!(
        weak.upgrade().is_none(),
        "charge never retains operational cell data"
    );
    assert_eq!(budget.charged_memory_bytes(), control);
    drop(weak);
    assert_eq!(budget.charged_memory_bytes(), control);
    drop(remaining);
    assert_eq!(budget.charged_memory_bytes(), 0);
    println!("regionArc={region} cell+grantArc={control} weak={} rawWeak={} cellFooter={} regionFooter={}",
        std::mem::size_of::<RegionCellWeak>(),
        std::mem::size_of::<std::sync::Weak<RelationalRegionResidency>>(),
        std::mem::size_of::<Option<Arc<ExecutionMemoryReservation>>>(),
        std::mem::size_of::<Option<ExecutionMemoryReservation>>());
}

#[test]
fn controlled_resident_cell_refuses_exact_control_before_any_new_arc() {
    let budget = SerialMemoryBudget::new(Some(4 * 1024 * 1024));
    let calibration =
        SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let mut controller = 0;
    ExecutionRequest::serial(&calibration)
        .run_native_with_checkpoint(|_| {
            controller = budget.charged_memory_bytes();
        })
        .unwrap();
    assert_eq!(budget.charged_memory_bytes(), 0);
    let region = allocation::<ResidentRegion>();
    let bill =
        allocation::<RelationalRegionResidency>() + allocation::<ExecutionMemoryReservation>();
    let budget = SerialMemoryBudget::new(Some(controller + region + bill - 1));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let result = RelationalRegionResidency::resident_for_request(
        partition(),
        PartitionContentCommitment::default(),
        kind_indexes(),
        None,
        &ExecutionRequest::serial(&host),
    );
    let Err(RegionContentFailure::Execution(cause)) = result else {
        panic!("entered actual control admission must refuse");
    };
    match cause {
        crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted {
            requested,
            admitted,
            ancestor: 0,
        } => {
            assert_eq!(requested, bill);
            assert_eq!(admitted, bill - 1);
        }
        other => panic!("exact control admission cause required: {other:?}"),
    }
    assert_eq!(budget.charged_memory_bytes(), 0);
    let cancel = CancellationSource::new();
    let budget = SerialMemoryBudget::new(Some(4 * 1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), cancel.token(), None);
    cancel.cancel();
    assert!(matches!(
        RelationalRegionResidency::resident_for_request(
            partition(),
            PartitionContentCommitment::default(),
            kind_indexes(),
            None,
            &ExecutionRequest::serial(&host)
        ),
        Err(RegionContentFailure::Execution(_))
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn actual_successor_supplements_equal_independent_head_and_full_closure_differences() {
    crate::storage::residency::resident_history::with_reconstruction_fixture(|fixture_lane| {
        let host = worth_harness::facade::execution_fixture::serial_request();
        let execution = ExecutionRequest::serial(&host);
        let lane = fixture_lane;
        let contract = AspectContract::scalar(
            AspectKey::new("name").unwrap(),
            AspectIdentity(1),
            AspectContractRevision(1),
            ScalarAspectType::String,
        );
        let symbols = RuntimeSymbolTable::default();
        let mut partition = PartitionState {
            partition_id: PartitionId::main(),
            adjacency_policy: AdjacencyPolicy {
                backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
                small_degree_inline_capacity: 4,
            },
            relation_overlay_is_sparse: false,
            entity_arena: EntityArena::with_capacity(1),
            relation_arena: RelationArena::with_capacity(0),
            adjacency: Default::default(),
            reverse_adjacency: Default::default(),
        };
        partition
            .entity_arena
            .write_reserved_slot(
                SlotInit {
                    partition_id: partition.partition_id,
                    kind_id: KindId(1),
                    version_id: VersionId(1),
                    extra: symbols
                        .with_write(|symbols| named_extra(&contract, symbols, "closure-before")),
                },
                0,
                1,
            )
            .unwrap();
        partition.adjacency.ensure(0, &partition.adjacency_policy);
        partition
            .reverse_adjacency
            .ensure(0, &partition.adjacency_policy);
        let mut previous = PreparedResidentRegion::initial(
            lane,
            &execution,
            partition.clone(),
            &symbols.interner_snapshot(),
        )
        .unwrap();
        let mut journal = PartitionMutationJournal::default();
        journal.entity_slots.insert(0);
        for step in 0..24 {
            partition
                .entity_arena
                .apply_extra_update(
                    0,
                    symbols.with_write(|symbols| {
                        named_extra(&contract, symbols, &format!("closure-step-{step}"))
                    }),
                    VersionId(2 + step),
                )
                .unwrap();
            let current = PreparedResidentRegion::incremental(
                &execution,
                RegionCaptureBasis {
                    retention: &previous.retention,
                    scope: &crate::storage::residency::OperationResidency::new(),
                    directory: &previous.directory,
                    kind_commitment: previous.kind_commitment,
                    inventory: previous.inventory,
                    unavailable_records: previous.unavailable_records,
                },
                &partition,
                &journal,
                || panic!("Full successor needs no cold reconstruction schema"),
                &symbols,
                None,
            )
            .unwrap();
            let before_region = previous
                .retention
                .cell()
                .demand_full(&execution, 1 << 24, None)
                .unwrap();
            let current_region = current
                .retention
                .cell()
                .demand_full(&execution, 1 << 24, None)
                .unwrap();
            let inherited = current_region
                .cell()
                .symbol_inheritance
                .get()
                .expect("actual canonical successor carries predecessor coverage");
            for full in [false, true] {
                let (predecessor, supplement) = inherited.predecessor(full).unwrap();
                assert_eq!(
                    predecessor,
                    if full {
                        before_region.symbol_admission_identity()
                    } else {
                        before_region.head_symbol_admission_identity()
                    }
                );
                let before = crate::storage::residency::required_symbols::collect_counted(
                    lane,
                    before_region.full_view().partition(),
                    full,
                )
                .ids;
                let cold = crate::storage::residency::required_symbols::collect_counted(
                    lane,
                    current_region.full_view().partition(),
                    full,
                )
                .ids;
                assert_eq!(
                    supplement
                        .ids
                        .difference(&before)
                        .copied()
                        .collect::<std::collections::BTreeSet<_>>(),
                    cold.difference(&before).copied().collect()
                );
            }
            previous = current;
        }
    })
}

fn named_extra(contract: &AspectContract, symbols: &mut StringInterner, name: &str) -> EntityExtra {
    let symbol = symbols.intern(name).unwrap();
    let TransitionOutcome::Success(value) = validate_aspect_value(
        contract,
        ContractValidationInput::Scalar(AspectValue::String(InternedString::Symbol(
            worth_foundational::facade::Symbol(symbol.0),
        ))),
    ) else {
        panic!("valid string aspect")
    };
    let TransitionOutcome::Success(state) = admit_authoritative_record_aspect_state([value]) else {
        panic!("valid record aspect state")
    };
    EntityExtra::from_unbudgeted_parts(None, None, Some(state.into_parts().into_parts().0))
}
