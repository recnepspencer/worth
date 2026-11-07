use super::*;

pub(super) fn write_receipt_commit_evidence_identity(
    commit_identity: &WorthQueryCommitIdentity,
) -> WorthQueryEvidenceIdentity {
    commit_identity.evidence_identity()
}

pub(super) fn write_receipt_snapshot_evidence_identity(
    snapshot_identity: &WorthQuerySnapshotIdentity,
) -> WorthQueryEvidenceIdentity {
    snapshot_identity.evidence_identity()
}

pub(super) fn write_receipt_committed_truth_identity(
    receipt: &WorthQueryMutationReceipt,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let delta_descriptors = receipt
        .deltas
        .iter()
        .map(|delta| {
            format!(
                "{}:{}:{}",
                delta.collection(),
                mutation_kind_as_str(delta.kind()),
                terminal_touched_aspect_digest_projections(delta.admitted_touched_aspects())
                    .join("|")
            )
        })
        .collect::<Vec<_>>();
    Ok(
        crate::evidence_identity::WorthQueryEvidenceIdentity::compose(
            crate::evidence_identity::WorthQueryEvidenceScope::JournalReplayOutcome,
        )
        .field_evidence_identity(
            crate::evidence_identity::WorthQueryEvidenceTag::new("committed_write_identity"),
            &receipt.commit_identity.evidence_identity(),
        )
        .field_value_sequence(
            crate::evidence_identity::WorthQueryEvidenceTag::new("committed_delta_descriptor"),
            delta_descriptors.iter().map(String::as_str),
        )
        .seal()?,
    )
}

fn terminal_touched_aspect_digest_projections(touches: &[WorthQueryAspectTouch]) -> Vec<String> {
    touches
        .iter()
        .map(WorthQueryAspectTouch::admitted_touch_digest_part)
        .collect()
}

fn mutation_kind_as_str(kind: &WorthQueryMutationKind) -> &'static str {
    match kind {
        WorthQueryMutationKind::Created => "created",
        WorthQueryMutationKind::Updated => "updated",
        WorthQueryMutationKind::Deleted => "deleted",
    }
}
