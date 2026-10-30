pub(crate) use {
    crate::phase_eight_process::history::parent_oracle::checkpoint_evidence,
    crate::phase_eight_process::history::parent_oracle::identity_evidence,
    crate::phase_eight_process::history::parent_oracle::manifest_evidence,
    crate::phase_eight_process::history::parent_oracle::page_evidence,
    crate::phase_eight_process::history::parent_oracle::residue_evidence,
    crate::phase_eight_process::history::parent_oracle::selector_evidence,
    crate::phase_eight_process::history::parent_oracle::wal_evidence,
};
pub(crate) use {
    crate::phase_eight_process::history::parent_oracle::evidence::ParentPhysicalEvidence,
    crate::phase_eight_process::history::parent_oracle::evidence::ParentPhysicalEvidenceParts,
};

pub(crate) fn derive(files: &[(String, Vec<u8>)]) -> Result<ParentPhysicalEvidence, String> {
    let identity =
        crate::phase_eight_process::history::parent_oracle::identity_evidence::derive(files);
    let selectors =
        crate::phase_eight_process::history::parent_oracle::selector_evidence::derive(files);
    let checkpoints =
        crate::phase_eight_process::history::parent_oracle::checkpoint_evidence::derive(files);
    let wal = crate::phase_eight_process::history::parent_oracle::wal_evidence::derive(files)?;
    let pages = crate::phase_eight_process::history::parent_oracle::page_evidence::derive(files);
    let manifests =
        crate::phase_eight_process::history::parent_oracle::manifest_evidence::derive(files);
    let residue =
        crate::phase_eight_process::history::parent_oracle::residue_evidence::derive(files);
    Ok(crate::phase_eight_process::history::parent_oracle::evidence::ParentPhysicalEvidence::from_parts(
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
