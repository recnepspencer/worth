//! Shared fixtures for Store M4.4 `S5` + `M7` historical retained-basis courts.

pub(crate) mod allocation_court;
pub(crate) mod boundary_sealing_fence;
pub(crate) mod lean_conformance_court;
pub(crate) mod m7_mutants_and_compile_fail;
pub(crate) mod s5_consolidation_reopen;
pub(crate) mod s5_crash_cut_recovery;
pub(crate) mod s5_denials_residency_and_m7;
pub(crate) mod s5_manifest_paging;
pub(crate) mod s5_race_and_restart;
pub(crate) mod s5_relational_diamond_fixture;
pub(crate) mod s5_relational_diamond_merge;
pub(crate) mod s5_relational_fork_history;
pub(crate) mod s5_relational_revision;
pub(crate) mod s5_residency_pool;

use worth_execution::{ExecutionRequest, SerialRequest};
use worth_relational::facade::config::ResidencyBudget;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_relational::facade::schema::{SchemaId, SchemaVersionId};
use worth_runtime_world::facade::{
    BranchOriginIdentity, CanonicalKeyDomain, HistoricalBasisRow, HistoricalQueryContract,
    SchemaVersionIdentity, ORDERING_CANONICAL_ASC, PROJECTION_CANONICAL_FULL,
};
use worth_store_application::{HistoricalCursor, ReadmittedApplicationHome};

pub(crate) fn test_schema(version: u32, byte: u8) -> SchemaVersionIdentity {
    SchemaVersionIdentity::new(
        SchemaId("m4.historical.audit".into()),
        SchemaVersionId(version),
        [byte; 32],
    )
    .expect("valid schema identity")
}

pub(crate) fn test_origin(name: &str, byte: u8) -> BranchOriginIdentity {
    BranchOriginIdentity::from_owner(BranchId(name.into()), [byte; 32]).expect("valid origin")
}

pub(crate) fn test_query(schema: &SchemaVersionIdentity) -> HistoricalQueryContract {
    test_query_with_projection(schema, PROJECTION_CANONICAL_FULL)
}

pub(crate) fn test_query_with_projection(
    schema: &SchemaVersionIdentity,
    projection: &str,
) -> HistoricalQueryContract {
    HistoricalQueryContract::new(
        schema.clone(),
        CanonicalKeyDomain::Entity,
        Some(KindId(7)),
        projection,
        ORDERING_CANONICAL_ASC,
    )
}

pub(crate) fn test_budget(head_bytes: u64, historical_bytes: u64) -> ResidencyBudget {
    ResidencyBudget {
        head_working_set_bytes: head_bytes,
        historical_working_set_bytes: historical_bytes,
        max_in_flight_fill_bytes: historical_bytes.max(16 * 1024),
        streaming_window_bytes: 16 * 1024,
    }
}

pub(crate) fn make_rows(
    count: usize,
    tag: &str,
    history_versions_per_row: usize,
    payload_bytes: usize,
) -> Vec<HistoricalBasisRow> {
    let mut rows = Vec::with_capacity(count);
    for i in 0..count {
        let partition_id = PartitionId((i / 32) as u32);
        let logical_key = format!("key-{i:06}").into_bytes();
        let mut payload = format!("{tag}-row-{i:06}-").into_bytes();
        if payload.len() < payload_bytes {
            payload.resize(payload_bytes, b'x');
        }
        let history_versions = (0..history_versions_per_row)
            .map(|v| format!("{tag}-row-{i:06}-hist-{v:03}").into_bytes())
            .collect();
        rows.push(HistoricalBasisRow {
            partition_id,
            domain: CanonicalKeyDomain::Entity,
            kind_id: KindId(7),
            logical_key,
            payload,
            history_versions,
        });
    }
    rows
}

pub(crate) fn drain_cursor(
    cursor: &mut HistoricalCursor<'_, HistoricalBasisRow>,
    execution: &ExecutionRequest<'_, '_>,
    page_size: usize,
) -> Vec<HistoricalBasisRow> {
    let mut out = Vec::new();
    loop {
        let page = cursor.next_page(execution, page_size).expect("next_page");
        out.extend(page.rows().iter().cloned());
        if !page.has_more() {
            break;
        }
    }
    out
}

pub(crate) fn open_durable_home(
    dir: &std::path::Path,
    request: &SerialRequest,
) -> ReadmittedApplicationHome {
    let execution = ExecutionRequest::serial(request);
    crate::durable_run::open(dir, &execution).expect("open durable application home")
}

pub(crate) fn close_durable_home(home: ReadmittedApplicationHome, request: &SerialRequest) {
    let execution = ExecutionRequest::serial(request);
    assert!(
        matches!(
            home.close(&execution),
            worth_store_application::ApplicationHomeCloseOutcome::Closed { .. }
        ),
        "expected clean close of durable home"
    );
}

pub(crate) fn canonical_logical_key(kind_id: u32, part: u32, slot: u64, gen_num: u32) -> Vec<u8> {
    let mut key = Vec::with_capacity(20);
    key.extend_from_slice(&kind_id.to_be_bytes());
    key.extend_from_slice(&part.to_be_bytes());
    key.extend_from_slice(&slot.to_be_bytes());
    key.extend_from_slice(&gen_num.to_be_bytes());
    key
}

pub(crate) fn build_history_versions<F>(
    entries: Vec<worth_relational::facade::history::AspectHistoryEntry>,
    v1_ver: worth_relational::facade::identity::VersionId,
    head_payload: &[u8],
    mut load_payload: F,
) -> Vec<Vec<u8>>
where
    F: FnMut(worth_relational::facade::identity::VersionId) -> Vec<u8>,
{
    entries
        .into_iter()
        .filter(|e| e.origin.version_id.0 <= v1_ver.0)
        .map(|e| {
            let vp = if e.origin.version_id == v1_ver {
                head_payload.to_vec()
            } else {
                load_payload(e.origin.version_id)
            };
            worth_store_application::encode_canonical_history_version(
                e.origin.commit_id,
                e.origin.version_id,
                e.origin.structural_change,
                &e.origin.changed_aspects,
                &vp,
            )
        })
        .collect()
}

pub(crate) fn commit_and_release(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    branch: &BranchId,
    batch: worth_relational::facade::transactions::WorkerIntentBatch,
) {
    let completed = crate::workload::commit(runtime, execution, branch, batch)
        .expect("commit real Relational batch");
    runtime
        .snapshots()
        .release_snapshot(&completed.snapshot)
        .expect("release commit snapshot");
}
