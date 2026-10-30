//! Independent domain input and retained native publication owners for both scale axes.
use super::{
    measurement,
    model::{
        host::{self, Host},
        live_at,
    },
    peak, reconstruction,
};
use std::time::Instant;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationMutationOutcome,
    primary_graph::WorthQueryApplicationCommitReceipt,
};

const SAMPLES: u64 = 8;

pub fn retained_census(
    history: u64,
) -> worth_relational::facade::inspection::RelationalRetainedStorageObservation {
    let court = CellHistory::build(3, history);
    super::storage::observe(&court.host)
}

pub fn assert_census_walk(
    inventory: &worth_relational::facade::inspection::RelationalRetainedStorageObservation,
    publications: u64,
) {
    let work = &inventory.work;
    // This real workload changes its cell partition and creates one ordinary
    // idempotency partition per step. Each radix replacement owns a 33-node
    // path; each first node visit offers at most two child edges. Root/schema/
    // set metadata and two leaf backing objects supply the remaining granule.
    // A current root still contains registration partitions from publications
    // whose old physical roots have already been reclaimed. Count those real
    // publications independently of the observed retained-root population.
    let regions = publications + work.roots + 1;
    assert!(
        work.region_entries <= regions,
        "regions were re-enumerated before deduplication: {work:?}"
    );
    assert!(
        work.root_allocation_rows <= regions * (2 * 33 + 1) + work.roots * 3,
        "shared radix subtrees were walked before deduplication: {work:?}"
    );
}

pub struct CellHistory {
    host: Host,
    live: u64,
    receipts: Vec<WorthQueryApplicationCommitReceipt>,
    initial: Vec<(u64, u64)>,
}

impl CellHistory {
    pub fn build(live: u64, history: u64) -> Self {
        assert!(live > 0 && history > 0);
        let setup = Instant::now();
        let (court, requested) = peak::measure(|| {
            let initial: Vec<_> = (1..=live).map(|key| (key, key + 7)).collect();
            let mut seeds: Vec<_> = initial
                .iter()
                .map(|&(key, value)| (key, 1, value))
                .collect();
            seeds.push((30_000, 2, 29));
            let host = host::install_cells(&seeds, history + SAMPLES);
            let mut receipts = Vec::new();
            for step in 1..=history {
                receipts.push(committed(&host, live, 1_000 + step, 10_000 + step));
            }
            Self {
                host,
                live,
                receipts,
                initial,
            }
        });
        let elapsed = setup.elapsed().as_micros();
        println!("cell setup allocator=instrumented-tracking-system live={live} retained_steps={history} elapsed_us={elapsed} retained_receipts={}", court.receipts.len());
        peak::report("setup-and-retained-history", requested);
        court
    }

    pub fn measure(self) -> measurement::WarmPublication {
        let occurrence = live_at::current(&self.host, 1)
            .into_iter()
            .find(|row| row.key == self.live)
            .unwrap()
            .occurrence;
        let mut previous_version = self.receipts.last().unwrap().commit_reference().version_id;
        let before = measurement::capture(&self.host);
        let mut micros = Vec::new();
        let mut decision_facts = 0;
        for sample in 0..SAMPLES {
            let start = Instant::now();
            let (receipt, allocation) = peak::measure(|| {
                committed(&self.host, self.live, 200_000 + sample, 1_000_000 + sample)
            });
            micros.push((start.elapsed().as_micros(), allocation));
            super::committed_patch::assert_one_updated_cell(&receipt, occurrence);
            assert_ne!(receipt.commit_reference().version_id, previous_version);
            previous_version = receipt.commit_reference().version_id;
            decision_facts += receipt
                .mutation_work()
                .expect("the original commit retains native mutation work")
                .decision_fact_count();
        }
        let observation = measurement::observe(&self.host, &before);
        assert!(
            observation.world_history_after().installed_commits() >= self.receipts.len(),
            "the history axis must remain genuinely retained"
        );
        let warm = measurement::report_warm(
            &observation,
            self.live,
            self.receipts.len() as u64,
            micros,
            decision_facts,
        );
        let mut first = self.initial.clone();
        first.last_mut().unwrap().1 = 1_001;
        measurement::historical(&self.host, &self.receipts[0], "first", &first);
        if self.receipts.len() > 2 {
            let middle_index = self.receipts.len() / 2;
            let mut middle = self.initial.clone();
            middle.last_mut().unwrap().1 = 1_001 + middle_index as u64;
            measurement::historical(&self.host, &self.receipts[middle_index], "middle", &middle);
        }
        let mut last = self.initial.clone();
        last.last_mut().unwrap().1 = 1_000 + self.receipts.len() as u64;
        if self.receipts.len() > 1 {
            measurement::historical(&self.host, self.receipts.last().unwrap(), "last", &last);
        }
        let mut current = self.initial.clone();
        current.last_mut().unwrap().1 = 200_000 + SAMPLES - 1;
        let current_rows = live_at::current(&self.host, 1);
        assert_eq!(
            current_rows
                .iter()
                .map(|row| (row.key, row.value))
                .collect::<Vec<_>>(),
            current
        );
        let untouched_rows = live_at::current(&self.host, 2);
        assert_eq!(
            untouched_rows
                .iter()
                .map(|row| (row.key, row.value))
                .collect::<Vec<_>>(),
            [(30_000, 29)]
        );
        // Explicit owner walk after warm measurement and answer checks. Its
        // allocation and elapsed work are diagnostic, never publication work.
        assert_census_walk(
            &super::storage::observe(&self.host),
            self.receipts.len() as u64 + SAMPLES,
        );
        let start = Instant::now();
        let (bytes, allocation) = peak::measure(|| super::model::checkpoint::capture(&self.host));
        let capture_elapsed = start.elapsed().as_micros();
        peak::report("checkpoint-capture-and-transport", allocation);
        println!("cell cold capture_transport_samples=1 capture_transport_us={} live={} retained_steps={} checkpoint_payload_bytes={}",
            capture_elapsed, self.live, self.receipts.len(), bytes.len());
        let history = self.receipts.len() as u64;
        let Self { host, receipts, .. } = self;
        drop(receipts);
        drop(host);
        reconstruction::measure(bytes, history + SAMPLES, &current_rows, &untouched_rows);
        warm
    }
}

fn committed(host: &Host, cell: u64, value: u64, key: u64) -> WorthQueryApplicationCommitReceipt {
    match live_at::edit(host, 1, cell, value, key) {
        WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => receipt,
        other => panic!("the ordinary cell-bearing step must commit: {other:?}"),
    }
}
