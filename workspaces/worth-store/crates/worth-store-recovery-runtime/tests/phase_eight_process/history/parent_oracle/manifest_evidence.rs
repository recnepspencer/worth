use super::{observe_artifact_at_path, DigestBuilder};

pub(super) struct ManifestEvidence {
    pub(super) count: u64,
    pub(super) member_count: u64,
    pub(super) digest: [u8; 32],
}

pub(super) fn derive(files: &[(String, Vec<u8>)]) -> ManifestEvidence {
    let mut count = 0;
    let mut member_count = 0;
    let mut digest = DigestBuilder::new(b"worth.store.recovery-observer.manifest-membership.v1");
    for (path, bytes) in files {
        if let Some(value) = observe_artifact_at_path(path, bytes).manifest {
            count += value.count;
            member_count += value.members;
            digest.record(&value.digest);
        }
    }
    ManifestEvidence {
        count,
        member_count,
        digest: digest.finish().digest(),
    }
}
