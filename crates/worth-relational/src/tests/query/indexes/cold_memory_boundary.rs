//! A real retained-commit reconstruction at its analytically priced edge.
use super::*;
use crate::facade::identity::EntityId;
use crate::facade::indexes::DerivedIndexMaintenanceBudget;
use crate::facade::transactions::{CreateIntent, MutationIntent, WorkerIntentBatch};
use crate::indexes::data::{DerivedIndexMaintenanceDenialKind as Denial, DerivedIndexRows};
use crate::storage::data::AuthoritativeFieldComparisonKey as Key;
use std::{mem::size_of, sync::Arc};
use worth_execution::ExecutionRequest;
const MEMORY: u64 = 64 * 1024 * 1024;
const FRAMEWORK: u64 = 376;
fn height(mut n: usize) -> usize {
    let mut h = 1;
    while n > 64 {
        n = n.div_ceil(32);
        h += 1;
    }
    h
}
fn node() -> usize {
    64 * size_of::<(Arc<Key>, DerivedIndexRows<EntityId>)>() + 71 * size_of::<usize>()
}
fn nodes(n: usize) -> usize {
    n.div_ceil(31) + 2 + 3 * height(n) + 2
}
fn ordered<K, V>() -> usize {
    11 * size_of::<(K, V)>() + 14 * size_of::<usize>()
}
fn row_bytes() -> usize {
    // One changed-ID node, one queued-key node, four pending edits, one retained
    // key Arc, one retained row Arc, and the canonical eight-byte string key.
    ordered::<EntityId, ()>()
        + ordered::<Key, Vec<(EntityId, bool)>>()
        + 4 * size_of::<(EntityId, bool)>()
        + size_of::<Key>()
        + size_of::<EntityId>()
        + 4 * size_of::<usize>()
        + Key::required_encoded_capacity_bytes(&string_aspect_value("row00000")).unwrap() as usize
}
fn reconstruct(n: usize, expected: Option<(u64, bool)>) -> u64 {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_index_field_aspects();
    let mut committed = None;
    // Populate through ordinary commits within the native 4,096-record limit.
    for start in (0..n).step_by(4096) {
        let mut batch = WorkerIntentBatch::new("cold-memory-source");
        for row in start..n.min(start + 4096) {
            let name = format!("row{row:05}");
            batch = batch.push(MutationIntent::Create(CreateIntent::Entity(
                crate::transactions::data::EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw(&name),
                    fields: name_field_patch(&name),
                },
            )));
        }
        let mut transaction = test_owner_begin_transaction_for_main(&runtime);
        transaction
            .push_batch(batch, execution.allocation_policy())
            .unwrap();
        let next = transaction.commit(&execution, &runtime).unwrap();
        if let Some(old) = committed.take() {
            release_test_commit_snapshot(&runtime, &old);
        }
        committed = Some(next);
    }
    let committed = committed.unwrap();
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "cold-memory.name".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
        })
        .index_id;
    // A newer head makes the source an actual retained target, still held by
    // its exact snapshot. Reconstruction never treats the active head as retained.
    let advanced = test_owner_begin_transaction_for_main(&runtime)
        .commit(&execution, &runtime)
        .unwrap();
    let reconstruction_host = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(MEMORY)),
        worth_execution::CancellationToken::new(),
        None,
    );
    let result = runtime.index_authority().reconstruct_for_commit(
        &ExecutionRequest::serial(&reconstruction_host),
        DerivedIndexBuildRequest {
            source_commit_id: committed.commit.commit_id,
            branch_id: BranchId("main".into()),
            index_ids: vec![index],
        },
        DerivedIndexMaintenanceBudget {
            maximum_work_units: Some(100_000_000),
            maximum_cold_record_slots: Some(1_000_000),
            maximum_derived_rows: Some(1_000_000),
        },
    );
    let charged = crate::indexes::last_allocation_bytes();
    match expected {
        Some((bytes, false)) => {
            let denied = result.unwrap_err();
            let Denial::Execution(
                crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted {
                    requested,
                    admitted,
                    ancestor,
                },
            ) = denied.kind
            else {
                panic!("the first unaffordable memory reservation must refuse: {denied:?}");
            };
            assert_eq!(admitted, MEMORY - FRAMEWORK);
            assert_eq!(ancestor, 0);
            // The reservation order may change without moving the independently
            // derived largest affordable shape or permitting partial publication.
            assert!(requested > admitted && requested <= bytes);
            assert!(runtime
                .index_access()
                .latest_generation(index, &BranchId("main".into()))
                .is_none());
        }
        otherwise => {
            let result = result.unwrap();
            let crate::facade::indexes::DerivedIndexEntries::EntityField(entries) =
                &result.generations[0].entries
            else {
                panic!("field index")
            };
            assert_eq!(entries.len(), n);
            if let Some((bytes, true)) = otherwise {
                assert_eq!(charged, bytes, "named shape prices at {n}");
            }
        }
    }
    release_test_commit_snapshot(&runtime, &advanced);
    release_test_commit_snapshot(&runtime, &committed);
    charged
}
#[test]
fn retained_commit_cold_reconstruction_fits_its_largest_serial_shape() {
    // The single-row run supplies only fixed schema/publication scaffolding;
    // all population growth is independently derived from named allocation prices.
    let first = reconstruct(1, None);
    let quote = |n: usize| {
        first + (nodes(n) - nodes(1)) as u64 * node() as u64 + (n - 1) as u64 * row_bytes() as u64
    };
    reconstruct(64, Some((quote(64), true)));
    let mut low: usize = 1;
    let mut high = 100_000;
    while low < high {
        let mid = (low + high).div_ceil(2);
        if quote(mid) <= MEMORY - FRAMEWORK {
            low = mid
        } else {
            high = mid - 1
        }
    }
    assert!(quote(low) <= MEMORY - FRAMEWORK && quote(low + 1) > MEMORY - FRAMEWORK);
    eprintln!(
        "cold public rows={low}, fixed={first}, per_row={}, node={}, bytes={}, next={}",
        row_bytes(),
        node(),
        quote(low),
        quote(low + 1)
    );
    reconstruct(low, Some((quote(low), true)));
    reconstruct(low + 1, Some((quote(low + 1), false)));
}
