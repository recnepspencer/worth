pub(crate) use {
    crate::history::parent_oracle::evidence_digest::DigestBuilder,
    crate::phase_eight_process::history::parent_oracle::artifact_facts::observe_artifact_at_path,
};

pub(crate) struct CheckpointEvidence {
    pub(crate) count: u64,
    pub(crate) latest_sequence: u64,
    pub(crate) page_count: u64,
    pub(crate) covered_start: Option<u64>,
    pub(crate) covered_end: Option<u64>,
    pub(crate) redo_lsn: Option<u64>,
    pub(crate) durable_lsn: Option<u64>,
    pub(crate) digest: [u8; 32],
}

pub(crate) fn derive(files: &[(String, Vec<u8>)]) -> CheckpointEvidence {
    let mut count = 0;
    let mut latest_sequence = 0;
    let mut page_count = 0;
    let mut covered_start = None;
    let mut covered_end = None;
    let mut redo_lsn = None;
    let mut durable_lsn = None;
    let mut digest = crate::history::parent_oracle::evidence_digest::DigestBuilder::new(
        b"worth.store.recovery-observer.checkpoint-coverage.v1",
    );
    for (path, bytes) in files {
        if let Some(value) = observe_artifact_at_path(path, bytes).checkpoint {
            count += 1;
            latest_sequence = latest_sequence.max(value.sequence);
            page_count += value.page_count;
            covered_start = min_option(covered_start, value.covered.0);
            covered_end = max_option(covered_end, value.covered.1);
            redo_lsn = min_option(redo_lsn, value.redo);
            durable_lsn = max_option(durable_lsn, value.durable);
            digest.record(&value.digest);
        }
    }
    CheckpointEvidence {
        count,
        latest_sequence,
        page_count,
        covered_start,
        covered_end,
        redo_lsn,
        durable_lsn,
        digest: digest.finish().digest(),
    }
}

pub(crate) fn min_option(current: Option<u64>, value: u64) -> Option<u64> {
    Some(current.map_or(value, |current| current.min(value)))
}

pub(crate) fn max_option(current: Option<u64>, value: u64) -> Option<u64> {
    Some(current.map_or(value, |current| current.max(value)))
}
