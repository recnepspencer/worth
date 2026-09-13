use super::evidence::{ParentPhysicalEvidence, ParentPhysicalEvidenceParts};
use super::{
    checkpoint_evidence, identity_evidence, manifest_evidence, page_evidence, residue_evidence,
    selector_evidence, wal_evidence,
};

pub(crate) fn derive(files: &[(String, Vec<u8>)]) -> Result<ParentPhysicalEvidence, String> {
    let identity = identity_evidence::derive(files);
    let selectors = selector_evidence::derive(files);
    let checkpoints = checkpoint_evidence::derive(files);
    let wal = wal_evidence::derive(files)?;
    let pages = page_evidence::derive(files);
    let manifests = manifest_evidence::derive(files);
    let residue = residue_evidence::derive(files);
    Ok(ParentPhysicalEvidence::from_parts(
        ParentPhysicalEvidenceParts {
            artifact_count: files.len() as u64,
            identity,
            selectors,
            checkpoints,
            wal,
            pages,
            manifests,
            residue,
        },
    ))
}
