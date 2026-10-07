use super::*;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

impl<'a> WorthQueryPreviewSession<'a> {
    pub fn compare_to_authoritative(&self) -> WorthQueryPreviewDiff {
        WorthQueryPreviewDiff {
            session_label: self.label.clone(),
            write_count: self.writes.len(),
            changed_entity_count: self
                .writes
                .iter()
                .flat_map(|receipt| receipt.deltas())
                .filter(|delta| {
                    matches!(
                        delta.kind,
                        WorthQueryMutationKind::Created
                            | WorthQueryMutationKind::Updated
                            | WorthQueryMutationKind::Deleted
                    )
                })
                .count(),
        }
    }

    pub fn promote(mut self) -> Result<WorthQueryPreviewOutcome, WorthQueryRuntimeError> {
        let staged_preview_write_count = self.pending_commands.len();
        let promotion_snapshot_identity = self.runtime.current_snapshot_identity()?;
        let residue_snapshot = self.residue_snapshot();
        if !promotion_snapshot_identity.is_same_current_identity_as(&self.basis_snapshot_identity) {
            return Err(WorthQueryRuntimeError::PreviewPromotionStaleBasis(
                WorthQueryPreviewPromotionDenialEvidence::stale_basis(
                    self.effect_policy,
                    &self.basis_admission,
                    &self.basis_snapshot_identity,
                    &promotion_snapshot_identity,
                    staged_preview_write_count,
                    self.handle_bindings.len(),
                ),
            ));
        }
        if staged_preview_write_count > 1 {
            return Err(
                WorthQueryRuntimeError::PreviewPromotionAtomicBatchUnsupported(
                    WorthQueryPreviewPromotionDenialEvidence::atomic_batch_unsupported(
                        self.effect_policy,
                        &self.basis_admission,
                        &self.basis_snapshot_identity,
                        &promotion_snapshot_identity,
                        staged_preview_write_count,
                        self.handle_bindings.len(),
                    ),
                ),
            );
        }
        let crossed_authoritative_residue_count =
            residue_snapshot.crossed_authoritative_residue_count;
        let promotion_rebinding_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::PreviewPromotionRebinding)
                .field_value(
                    WorthQueryEvidenceTag::new("session_label_identity"),
                    self.basis_admission.label_identity().as_str(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("basis_snapshot_identity"),
                    &self.basis_snapshot_identity.evidence_identity(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("promotion_snapshot_identity"),
                    &promotion_snapshot_identity.evidence_identity(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("crossed_authoritative_residue_count"),
                    crossed_authoritative_residue_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("preview_binding_count"),
                    self.handle_bindings.len(),
                )
                .seal();
        if crossed_authoritative_residue_count > 0 {
            return Err(WorthQueryRuntimeError::PreviewPromotionRebindingRequired(
                WorthQueryPreviewPromotionDenialEvidence::rebinding_required(
                    self.effect_policy,
                    &self.basis_admission,
                    &self.basis_snapshot_identity,
                    &promotion_snapshot_identity,
                    staged_preview_write_count,
                    self.handle_bindings.len(),
                    crossed_authoritative_residue_count,
                    promotion_rebinding_digest,
                ),
            ));
        }
        let mut promoted_writes = 0;
        for (index, command) in std::mem::take(&mut self.pending_commands)
            .into_iter()
            .enumerate()
        {
            match self.runtime.write(command) {
                Ok(receipt) => {
                    self.writes.push(receipt);
                    promoted_writes += 1;
                }
                Err(error) => {
                    let reason = error.to_string();
                    let evidence = WorthQueryPreviewPromotionDenialEvidence::write_failed(
                        self.effect_policy,
                        &self.basis_admission,
                        &self.basis_snapshot_identity,
                        &promotion_snapshot_identity,
                        staged_preview_write_count,
                        promoted_writes,
                        index + 1,
                        self.handle_bindings.len(),
                        reason,
                    );
                    return Err(WorthQueryRuntimeError::PreviewPromotionWriteFailed { evidence });
                }
            }
        }
        self.promoted = true;
        let preview_binding_count = self.handle_bindings.len();
        let effect_binding_count = self.effect_binding_count();
        let effect_delivery_residue_count = self.effect_delivery_residue_count();
        let pending_write_intent_residue_count = self.pending_write_intent_residue_count();
        let authoritative_residue_count = self.authoritative_residue_count();
        let closeout_evidence = self.closeout_evidence(
            WorthQueryPreviewCloseoutKind::Promoted,
            staged_preview_write_count,
            promoted_writes,
            residue_snapshot,
            &promotion_snapshot_identity,
            Some(promotion_rebinding_digest),
        );
        Ok(WorthQueryPreviewOutcome {
            session_label: self.label.clone(),
            effect_policy: self.effect_policy,
            promoted: self.promoted,
            discarded: self.discarded,
            write_count: promoted_writes,
            preview_binding_count,
            effect_binding_count,
            effect_delivery_residue_count,
            pending_write_intent_residue_count,
            authoritative_residue_count,
            closeout_evidence,
            source_lane: WorthQueryAuthorityLane::PreviewTruth,
            target_lane: WorthQueryAuthorityLane::AuthoritativeTruth,
        })
    }

    pub fn discard(mut self) -> WorthQueryPreviewOutcome {
        self.discarded = true;
        let staged_preview_write_count = self.pending_commands.len();
        let residue_snapshot = self.residue_snapshot();
        let preview_binding_count = self.handle_bindings.len();
        let effect_binding_count = self.effect_binding_count();
        let effect_delivery_residue_count = self.effect_delivery_residue_count();
        let pending_write_intent_residue_count = self.pending_write_intent_residue_count();
        let authoritative_residue_count = self.authoritative_residue_count();
        let closeout_evidence = self.closeout_evidence(
            WorthQueryPreviewCloseoutKind::Discarded,
            staged_preview_write_count,
            0,
            residue_snapshot,
            &self.basis_snapshot_identity,
            None,
        );
        WorthQueryPreviewOutcome {
            session_label: self.label.clone(),
            effect_policy: self.effect_policy,
            promoted: self.promoted,
            discarded: self.discarded,
            write_count: self.writes.len(),
            preview_binding_count,
            effect_binding_count,
            effect_delivery_residue_count,
            pending_write_intent_residue_count,
            authoritative_residue_count,
            closeout_evidence,
            source_lane: WorthQueryAuthorityLane::PreviewTruth,
            target_lane: WorthQueryAuthorityLane::PreviewTruth,
        }
    }

    pub fn execute_intent(
        &mut self,
        declaration: WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryPreviewIntentReceipt, WorthQueryRuntimeError> {
        self.runtime.admit_facade_family_lane(
            WorthQueryRuntimeFacadeFamily::Intent,
            WorthQueryAuthorityLane::PreviewTruth,
        )?;
        let declaration = declaration
            .with_source_lane(WorthQueryIntentSourceLane::PreviewLocal)
            .with_target_lane(WorthQueryAuthorityLane::PreviewTruth);
        let admission = crate::runtime::intent::admit_preview_intent_declaration(
            &declaration,
            self.effect_policy,
        )
        .map_err(|denial| WorthQueryRuntimeError::IntentCommitDenied {
            intent_name: declaration.name().to_string(),
            stage: denial.stage(),
            message: denial.message().to_string(),
            evidence: WorthQueryIntentDenialEvidence::new(&declaration, &denial, None),
        })?;
        let receipt = WorthQueryPreviewIntentReceipt::new(
            &declaration,
            self.effect_policy,
            &self.basis_admission,
            admission,
        );
        self.intent_receipts.push(receipt.clone());
        self.execution_evidence
            .push(WorthQueryPreviewExecutionEvidence::for_intent_strategy(
                &self.basis_admission,
                declaration.name(),
                receipt.receipt_identity(),
                declaration.strategy_name(),
            ));
        Ok(receipt)
    }

    pub(super) fn admit_operation_effect(
        &self,
        effect: &WorthQueryProgramEffect,
    ) -> Result<(), WorthQueryRuntimeError> {
        match effect {
            WorthQueryProgramEffect::DeclareLiveView { name, .. } => {
                Err(WorthQueryRuntimeError::PreviewOperationEffectDenied {
                    label: self.label.clone(),
                    stage: "effect-admission",
                    message: format!(
                        "preview operations cannot install live view `{name}` into authoritative runtime state; declare the live surface before entering preview or add preview-scoped declaration support"
                    ),
                })
            }
            WorthQueryProgramEffect::DeclareDerivedView(view) => {
                Err(WorthQueryRuntimeError::PreviewOperationEffectDenied {
                    label: self.label.clone(),
                    stage: "effect-admission",
                    message: format!(
                        "preview operations cannot install computed view `{}` into authoritative runtime state; declare the computed surface before entering preview or add preview-scoped declaration support",
                        view.name()
                    ),
                })
            }
            WorthQueryProgramEffect::Write(_)
            | WorthQueryProgramEffect::WriteTemplate(_)
            | WorthQueryProgramEffect::ReadLive { .. }
            | WorthQueryProgramEffect::DrainPatches { .. } => Ok(()),
        }
    }
}
