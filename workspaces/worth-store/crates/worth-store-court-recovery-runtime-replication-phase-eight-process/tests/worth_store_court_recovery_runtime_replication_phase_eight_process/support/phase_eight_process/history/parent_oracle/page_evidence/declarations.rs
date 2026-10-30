pub(crate) use {
    crate::history::parent_oracle::evidence_digest::DigestBuilder,
    crate::phase_eight_process::history::parent_oracle::artifact_facts::observe_artifact_at_path,
};

pub(crate) struct PageEvidence {
    pub(crate) count: u64,
    pub(crate) minimum: Option<u64>,
    pub(crate) maximum: Option<u64>,
    pub(crate) digest: [u8; 32],
}

pub(crate) fn derive(files: &[(String, Vec<u8>)]) -> PageEvidence {
    let mut count = 0;
    let mut minimum: Option<u64> = None;
    let mut maximum: Option<u64> = None;
    let mut digest = crate::history::parent_oracle::evidence_digest::DigestBuilder::new(
        b"worth.store.recovery-observer.page-lsn.v1",
    );
    for (path, bytes) in files {
        if let Some(value) = observe_artifact_at_path(path, bytes).page {
            count += value.count;
            if let Some(value) = value.minimum {
                minimum = Some(minimum.map_or(value, |current| current.min(value)));
            }
            if let Some(value) = value.maximum {
                maximum = Some(maximum.map_or(value, |current| current.max(value)));
            }
            if value.digest.observations() > 0 {
                digest.record(&value.digest.digest());
            }
        }
    }
    PageEvidence {
        count,
        minimum,
        maximum,
        digest: digest.finish().digest(),
    }
}
