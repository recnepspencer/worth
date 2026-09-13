use super::{observe_artifact_at_path, wal_topology, DigestBuilder, WalFacts};

pub(super) struct WalEvidence {
    pub(super) segment_count: u64,
    pub(super) valid_bytes: u64,
    pub(super) observed_bytes: u64,
    pub(super) frame_count: u64,
    pub(super) first_lsn: Option<u64>,
    pub(super) last_lsn: Option<u64>,
    pub(super) digest: [u8; 32],
}

pub(super) fn derive(files: &[(String, Vec<u8>)]) -> Result<WalEvidence, String> {
    let mut observations = Vec::<WalFacts>::new();
    let mut segment_count = 0;
    let mut valid_bytes = 0;
    let mut observed_bytes = 0;
    let mut frame_count = 0;
    let mut first_lsn = None;
    let mut last_lsn = None;
    let mut digest = DigestBuilder::new(b"worth.store.recovery-observer.wal-prefix.v1");
    for (path, bytes) in files {
        if let Some(value) = observe_artifact_at_path(path, bytes).wal {
            observations.push(value);
            segment_count += 1;
            valid_bytes += value.valid_bytes;
            observed_bytes += value.observed_bytes;
            frame_count += value.frames;
            first_lsn = min_option(first_lsn, value.first);
            last_lsn = max_option(last_lsn, value.last);
            digest.record(&value.digest);
        }
    }
    wal_topology::validate(&observations)?;
    Ok(WalEvidence {
        segment_count,
        valid_bytes,
        observed_bytes,
        frame_count,
        first_lsn,
        last_lsn,
        digest: digest.finish().digest(),
    })
}

fn min_option(current: Option<u64>, value: Option<u64>) -> Option<u64> {
    value.map_or(current, |value| {
        Some(current.map_or(value, |current| current.min(value)))
    })
}

fn max_option(current: Option<u64>, value: Option<u64>) -> Option<u64> {
    value.map_or(current, |value| {
        Some(current.map_or(value, |current| current.max(value)))
    })
}
