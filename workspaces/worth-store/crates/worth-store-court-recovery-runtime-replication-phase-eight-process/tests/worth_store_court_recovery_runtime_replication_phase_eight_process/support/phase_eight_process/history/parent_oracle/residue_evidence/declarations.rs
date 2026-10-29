pub(crate) use {crate::phase_eight_process::history::parent_oracle::artifact_facts::observe_artifact_at_path, crate::history::parent_oracle::evidence_digest::DigestBuilder};

pub(crate) struct ResidueEvidence {
    pub(crate) count: u64,
    pub(crate) bytes: u64,
    pub(crate) digest: [u8; 32],
}

pub(crate) fn derive(files: &[(String, Vec<u8>)]) -> ResidueEvidence {
    let mut count = 0;
    let mut bytes = 0;
    let mut digest = crate::history::parent_oracle::evidence_digest::DigestBuilder::new(b"worth.store.recovery-observer.residue.v1");
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