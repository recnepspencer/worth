use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::super::receipt::{EffectExecutionReceipt, EffectReceiptTargetEvidence};

pub(super) fn structural_delta_identities(
    receipt: &EffectExecutionReceipt,
) -> Vec<WorthQueryEvidenceIdentity> {
    match receipt.target_evidence() {
        EffectReceiptTargetEvidence::MutationCommit {
            commit_id,
            version_id,
        } => {
            vec![
                WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
                    .field_shape(
                        WorthQueryEvidenceTag::new("identity_family"),
                        "effect_envelope_structural_delta_v1",
                    )
                    .field_shape(WorthQueryEvidenceTag::new("kind"), "mutation_commit")
                    .field_usize(WorthQueryEvidenceTag::new("commit_id"), commit_id as usize)
                    .field_usize(
                        WorthQueryEvidenceTag::new("version_id"),
                        version_id as usize,
                    )
                    .seal().expect("structural delta grammar has at most three sealed identities or primitive coordinates and two literal labels"),
            ]
        }
        EffectReceiptTargetEvidence::MergeCommit {
            commit_id,
            version_id,
        } => {
            vec![
                WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
                    .field_shape(
                        WorthQueryEvidenceTag::new("identity_family"),
                        "effect_envelope_structural_delta_v1",
                    )
                    .field_shape(WorthQueryEvidenceTag::new("kind"), "merge_commit")
                    .field_usize(WorthQueryEvidenceTag::new("commit_id"), commit_id as usize)
                    .field_usize(
                        WorthQueryEvidenceTag::new("version_id"),
                        version_id as usize,
                    )
                    .seal().expect("structural delta grammar has at most three sealed identities or primitive coordinates and two literal labels"),
            ]
        }
        EffectReceiptTargetEvidence::Writeback {
            outcome_identity,
            authority_receipt_identity,
            execution_receipt_identity,
        } => {
            vec![
                WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
                    .field_shape(
                        WorthQueryEvidenceTag::new("identity_family"),
                        "effect_envelope_structural_delta_v1",
                    )
                    .field_shape(WorthQueryEvidenceTag::new("kind"), "writeback")
                    .field_evidence_identity(
                        WorthQueryEvidenceTag::new("outcome"),
                        &outcome_identity,
                    )
                    .field_evidence_identity(
                        WorthQueryEvidenceTag::new("authority_receipt"),
                        &authority_receipt_identity,
                    )
                    .field_evidence_identity(
                        WorthQueryEvidenceTag::new("execution_receipt"),
                        &execution_receipt_identity,
                    )
                    .seal().expect("structural delta grammar has at most three sealed identities or primitive coordinates and two literal labels"),
            ]
        }
        EffectReceiptTargetEvidence::BatchMutation {
            commit_id,
            version_id,
            component_count,
        } => {
            vec![
                WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
                    .field_shape(
                        WorthQueryEvidenceTag::new("identity_family"),
                        "effect_envelope_structural_delta_v1",
                    )
                    .field_shape(WorthQueryEvidenceTag::new("kind"), "batch_commit")
                    .field_usize(WorthQueryEvidenceTag::new("commit_id"), commit_id as usize)
                    .field_usize(
                        WorthQueryEvidenceTag::new("version_id"),
                        version_id as usize,
                    )
                    .field_usize(
                        WorthQueryEvidenceTag::new("component_count"),
                        component_count,
                    )
                    .seal().expect("structural delta grammar has at most three sealed identities or primitive coordinates and two literal labels"),
            ]
        }
    }
}
