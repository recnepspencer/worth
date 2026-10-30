//! Independent domain input and retained native publication owners for both scale axes.
use super::{
    super::{
        host::{self, Host},
        live_at,
    },
    measurement,
};
use std::time::Instant;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationMutationOutcome,
    primary_graph::WorthQueryApplicationCommitReceipt,
};

const SAMPLES: u64 = 8;

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
        println!(
            "cell setup live={live} retained_steps={history} elapsed_us={} retained_receipts={}",
            setup.elapsed().as_micros(),
            receipts.len()
        );
        Self {
            host,
            live,
            receipts,
            initial,
        }
    }

    pub fn measure(&self) -> measurement::WarmPublication {
        let before = measurement::capture(&self.host);
        let mut micros = Vec::new();
        let mut decision_facts = 0;
        for sample in 0..SAMPLES {
            let start = Instant::now();
            let receipt = committed(&self.host, self.live, 200_000 + sample, 1_000_000 + sample);
            decision_facts += receipt
                .mutation_work()
                .expect("the original commit retains native mutation work")
                .decision_fact_count();
            micros.push(start.elapsed().as_micros());
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
        assert_eq!(
            live_at::current(&self.host, 1)
                .iter()
                .map(|row| (row.key, row.value))
                .collect::<Vec<_>>(),
            current
        );
        assert_eq!(
            live_at::current(&self.host, 2)
                .iter()
                .map(|row| (row.key, row.value))
                .collect::<Vec<_>>(),
            [(30_000, 29)]
        );
        warm
    }
}

fn committed(host: &Host, cell: u64, value: u64, key: u64) -> WorthQueryApplicationCommitReceipt {
    match live_at::edit(host, 1, cell, value, key) {
        WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => receipt,
        other => panic!("the ordinary cell-bearing step must commit: {other:?}"),
    }
}
