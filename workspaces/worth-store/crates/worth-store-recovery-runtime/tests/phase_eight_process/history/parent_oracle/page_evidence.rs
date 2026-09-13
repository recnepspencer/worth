use super::{observe_artifact_at_path, DigestBuilder};

pub(super) struct PageEvidence {
    pub(super) count: u64,
    pub(super) minimum: Option<u64>,
    pub(super) maximum: Option<u64>,
    pub(super) digest: [u8; 32],
}

pub(super) fn derive(files: &[(String, Vec<u8>)]) -> PageEvidence {
    let mut count = 0;
    let mut minimum: Option<u64> = None;
    let mut maximum: Option<u64> = None;
    let mut digest = DigestBuilder::new(b"worth.store.recovery-observer.page-lsn.v1");
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
