//! Existing Native work and actual query receipts; no retained-history byte estimate.
use super::super::{
    host::Host,
    query::{CellRow, LiveCells},
};
use crate::operator_identity::{authenticate_operator, request_scope};
use std::time::Instant;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequestExt,
    primary_graph::WorthQueryApplicationCommitReceipt,
};
use worth_query_replay::facade::{
    WorthQueryCertificationCostObservation, WorthQueryCertificationCostRuntimeExt,
    WorthQueryCertificationCostScope,
};

#[derive(Debug)]
pub struct WarmPublication {
    pub copied_bytes: u64,
    pub hashed_values: u64,
    pub index_path_nodes: u64,
    pub new_authoritative_bytes: u64,
    pub decision_facts: usize,
    pub world_reachability_lookups: u64,
}

pub fn capture(host: &Host) -> WorthQueryCertificationCostScope {
    let scope = request_scope();
    host.runtime()
        .with_application_advancement(&scope, |phase| {
            host.runtime()
                .capture_certification_cost_scope(&phase, host.current_world())
                .unwrap()
        })
        .unwrap()
}

pub fn observe(
    host: &Host,
    before: &WorthQueryCertificationCostScope,
) -> WorthQueryCertificationCostObservation {
    let scope = request_scope();
    host.runtime()
        .with_application_advancement(&scope, |phase| {
            host.runtime()
                .observe_certification_cost(&phase, before)
                .unwrap()
        })
        .unwrap()
}

pub fn report_warm(
    observation: &WorthQueryCertificationCostObservation,
    live: u64,
    history: u64,
    mut micros: Vec<u128>,
    decision_facts: usize,
) -> WarmPublication {
    micros.sort_unstable();
    let delta = observation.relational().sharing_cost_delta();
    assert!(
        delta.copied_truth_bytes > 0,
        "the actual cell edits must copy changed truth"
    );
    assert!(delta.publication_content_values_hashed > 0);
    assert_eq!(
        delta.branch_population_scans, 0,
        "ordinary publication does not scan runtime branches"
    );
    assert_eq!(
        delta.copied_commit_envelopes, 0,
        "ordinary publication shares original commit envelopes"
    );
    let sharing = observation.relational().sharing();
    let world_reachability_lookups = observation.world_history_after().reachability_lookups()
        - observation.world_history_before().reachability_lookups();
    assert!(
        decision_facts > 0,
        "real decision observations reach the committing owner"
    );
    println!("cell warm decision_facts={decision_facts} world_reachability_lookups={world_reachability_lookups}");
    println!("cell warm live={live} retained_steps={history} samples={} p50_us={} p95_us={} min_us={} max_us={} copied_bytes={} new_authoritative_bytes={} hashed_values={} index_path_nodes={} touched_regions={} head_payload_bytes={} head_canonical_bytes={} head_optional_cache_bytes={} held_world_commits={}",
        micros.len(), micros[micros.len()/2], micros[(micros.len()*95/100).min(micros.len()-1)],
        micros[0], micros[micros.len()-1],
        delta.copied_truth_bytes, delta.publication_new_authoritative_bytes,
        delta.publication_content_values_hashed, delta.publication_persistent_index_path_nodes,
        delta.publication_touched_region_count, sharing.unique_physical_partition_payload_bytes(),
        sharing.unique_physical_canonical_commit_bytes(), sharing.unique_optional_cache_bytes(),
        observation.world_history_after().installed_commits());
    WarmPublication {
        copied_bytes: delta.copied_truth_bytes,
        hashed_values: delta.publication_content_values_hashed,
        index_path_nodes: delta.publication_persistent_index_path_nodes,
        new_authoritative_bytes: delta.publication_new_authoritative_bytes,
        decision_facts,
        world_reachability_lookups,
    }
}

pub fn historical(
    host: &Host,
    receipt: &WorthQueryApplicationCommitReceipt,
    label: &str,
    expected: &[(u64, u64)],
) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let start = Instant::now();
    let (mut rows, reads) = host
        .runtime()
        .with_application_advancement(&scope, |phase| {
            let result = host
                .runtime()
                .request(&principal, &scope)
                .at_commit(&phase, receipt, None)
                .unwrap()
                .query(LiveCells { feature: 1 })
                .execute_in_advancement(&phase)
                .unwrap();
            let work = result.receipt().inspect().read_work();
            assert_eq!(
                work.truncation_count(),
                0,
                "the history answer must be complete"
            );
            (result.rows().to_vec(), work.reads())
        })
        .unwrap();
    let micros = start.elapsed().as_micros();
    rows.sort_by_key(|row: &CellRow| row.key);
    let values: Vec<_> = rows.iter().map(|row| (row.key, row.value)).collect();
    assert_eq!(
        values, expected,
        "the historical query must return every independent live cell"
    );
    println!("cell historical selection={label} rows={} elapsed_us={micros} query_fields={} query_adjacencies={} query_edges={} query_endpoints={} query_index_candidates={} returned_row_capacity_bytes={}",
        rows.len(), reads.field_reads(), reads.adjacency_lists_read(),
        reads.adjacency_edges_inspected(), reads.endpoint_records_read(), reads.index_candidates_examined(),
        rows.capacity()*std::mem::size_of::<CellRow>());
}
