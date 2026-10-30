pub(crate) use {
    crate::history::parent_oracle::evidence_digest::DigestBuilder,
    crate::phase_eight_process::history::parent_oracle::artifact_facts::observe_artifact_at_path,
};

pub(crate) struct ManifestEvidence {
    pub(crate) count: u64,
    pub(crate) member_count: u64,
    pub(crate) digest: [u8; 32],
}

pub(crate) fn derive(files: &[(String, Vec<u8>)]) -> ManifestEvidence {
    let mut count = 0;
    let mut member_count = 0;
    let mut digest = crate::history::parent_oracle::evidence_digest::DigestBuilder::new(
        b"worth.store.recovery-observer.manifest-membership.v1",
    );
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
