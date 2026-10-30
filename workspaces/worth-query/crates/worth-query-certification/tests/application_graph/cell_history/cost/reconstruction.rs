//! Independent public readmission and first-answer costs after source custody ends.
use super::super::{
    checkpoint,
    host::Host,
    query::{CellRow, LiveCells},
};
use crate::operator_identity::{authenticate_operator, request_scope};
use std::time::Instant;
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestExt;

const REPETITIONS: usize = 3;

pub fn measure(bytes: Box<[u8]>, retained_steps: u64, current: &[CellRow], untouched: &[CellRow]) {
    // Expected rows contain only issued weak identities and values already
    // checked against independent authored inputs. No old basis or host enters.
    let mut copies = Vec::with_capacity(REPETITIONS);
    let mut readmissions = Vec::with_capacity(REPETITIONS);
    let mut materializations = Vec::with_capacity(REPETITIONS);
    for sample in 0..REPETITIONS {
        let copy_start = Instant::now();
        let (transport, allocation) = super::peak::measure(|| bytes.clone());
        copies.push(copy_start.elapsed().as_micros());
        super::peak::report("checkpoint-transport-copy", allocation);
        let start = Instant::now();
        let (host, allocation) =
            super::peak::measure(|| checkpoint::restore(transport, retained_steps));
        readmissions.push(start.elapsed().as_micros());
        super::peak::report("checkpoint-readmission-constructor", allocation);
        materializations.push(first_answer(&host, sample, current));
        assert_eq!(
            super::super::live_at::current(&host, 2),
            untouched,
            "the other feature also survives complete public readmission"
        );
        drop(host);
    }
    report("transport_copy", copies);
    report("readmission_constructor", readmissions);
    report(
        "first_public_answer_including_auth_and_owned_rows",
        materializations,
    );
}

fn first_answer(host: &Host, sample: usize, expected: &[CellRow]) -> u128 {
    let start = Instant::now();
    let ((mut rows, reads, truncated), allocation) = super::peak::measure(|| {
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let result = host
            .runtime()
            .request(&principal, &scope)
            .query(LiveCells { feature: 1 })
            .execute()
            .unwrap();
        let work = result.receipt().inspect().read_work();
        (
            result.rows().to_vec(),
            work.reads(),
            work.truncation_count(),
        )
    });
    let elapsed = start.elapsed().as_micros();
    assert_eq!(truncated, 0, "the first cold answer is complete");
    rows.sort_by_key(|row| row.key);
    assert_eq!(
        rows, expected,
        "readmission preserves every value and issued occurrence"
    );
    super::peak::report(
        "first-public-answer-including-auth-and-owned-rows",
        allocation,
    );
    println!("cell cold allocator=instrumented-tracking-system first_public_answer_sample={sample} includes=fresh-scope-authentication-query-owned-row-extraction rows={} elapsed_us={elapsed} query_fields={} query_adjacencies={} query_edges={} query_endpoints={} query_index_candidates={} returned_row_capacity_bytes={}",
        rows.len(), reads.field_reads(), reads.adjacency_lists_read(),
        reads.adjacency_edges_inspected(), reads.endpoint_records_read(),
        reads.index_candidates_examined(), rows.capacity() * std::mem::size_of::<CellRow>());
    elapsed
}

fn report(lane: &str, mut micros: Vec<u128>) {
    micros.sort_unstable();
    println!(
        "cell cold lane={lane} samples={} p50_us={} p95_us={} min_us={} max_us={}",
        micros.len(),
        micros[micros.len() / 2],
        micros[(micros.len() * 95 / 100).min(micros.len() - 1)],
        micros[0],
        micros[micros.len() - 1]
    );
}
