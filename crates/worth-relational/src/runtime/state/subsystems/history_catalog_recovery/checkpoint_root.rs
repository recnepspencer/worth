//! Image roots retain their actual owner; inventory synchronization never relinks them to envelopes.
use super::*;

pub(super) fn is_readmitted_checkpoint_root(
    history: &HistorySubsystem,
    root: &crate::branch::RelationalBranchRoot,
) -> Result<bool, String> {
    let Some(crate::branch::RelationalRootCommitEvidence::Checkpoint(commit)) =
        root.commit_evidence()
    else {
        return Ok(false);
    };
    let artifact = history
        .commit_artifact(commit.index().commit.commit_id)
        .ok_or("readmitted checkpoint root has no actual catalog artifact")?;
    let (entry, _source) = artifact
        .envelope_residency()
        .source_placement()
        .ok_or("readmitted checkpoint root has no actual source placement")?;
    if !commit.links_artifact(&artifact)
        || commit.index() != entry.as_ref()
        || root.descriptor() != Some(artifact.roots())
    {
        return Err(
            "readmitted checkpoint root differs from its original catalog source".to_owned(),
        );
    }
    Ok(true)
}
