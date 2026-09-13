use super::{observe_artifact_at_path, DigestBuilder};

pub(super) struct ResidueEvidence {
    pub(super) count: u64,
    pub(super) bytes: u64,
    pub(super) digest: [u8; 32],
}

pub(super) fn derive(files: &[(String, Vec<u8>)]) -> ResidueEvidence {
    let mut count = 0;
    let mut bytes = 0;
    let mut digest = DigestBuilder::new(b"worth.store.recovery-observer.residue.v1");
    for (path, bytes_value) in files {
        let facts = observe_artifact_at_path(path, bytes_value);
        for value in [facts.residue, facts.wal_residue].into_iter().flatten() {
            count += 1;
            bytes += value.len;
            digest.record(&value.digest);
        }
    }
    ResidueEvidence {
        count,
        bytes,
        digest: digest.finish().digest(),
    }
}
