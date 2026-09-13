use sha2::{Digest, Sha256};

use super::{observe_artifact_at_path, DigestBuilder};

pub(super) struct IdentityEvidence {
    pub(super) artifact_digest: [u8; 32],
    pub(super) generation_link_count: u64,
    pub(super) generation_link_digest: [u8; 32],
}

pub(super) fn derive(files: &[(String, Vec<u8>)]) -> IdentityEvidence {
    let mut identity = DigestBuilder::new(b"worth.store.recovery-observer.artifact-identity.v1");
    let mut generation_digest =
        DigestBuilder::new(b"worth.store.recovery-observer.generation-link.v1");
    let mut generation_link_count = 0;
    for (path, bytes) in files {
        let content_digest: [u8; 32] = Sha256::digest(bytes).into();
        let mut record = Vec::with_capacity(path.len() + 48);
        record.extend_from_slice(&(path.len() as u64).to_le_bytes());
        record.extend_from_slice(path.as_bytes());
        record.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
        record.extend_from_slice(&content_digest);
        identity.record(&record);

        let facts = observe_artifact_at_path(path, bytes);
        generation_link_count += u64::from(facts.generation);
        if facts.generation_links.observations() > 0 {
            let mut record = Vec::with_capacity(path.len() + 32);
            record.extend_from_slice(path.as_bytes());
            record.extend_from_slice(&facts.generation_links.digest());
            generation_digest.record(&record);
        }
    }
    let identity = identity.finish();
    let generation_digest = generation_digest.finish();
    IdentityEvidence {
        artifact_digest: identity.digest(),
        generation_link_count,
        generation_link_digest: generation_digest.digest(),
    }
}
