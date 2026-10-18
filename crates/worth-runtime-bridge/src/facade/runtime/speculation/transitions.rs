use super::*;

impl RuntimeBridge {
    /// Discards an active preview session with explicit residue classification.
    pub fn discard_preview_session(
        &self,
        session: BridgePreviewSession<PreviewActive>,
        execution_record: &BridgePreviewExecutionRecord,
        residue_classes: Vec<BridgePreviewResidueClass>,
    ) -> Result<
        (
            BridgePreviewSession<PreviewDiscarded>,
            BridgePreviewDiscardRecord,
        ),
        (BridgePreviewSession<PreviewActive>, BridgeSpeculationError),
    > {
        if let Err(denial) = self
            .ensure_execution_record_matches_active_session(&session, execution_record)
            .and_then(|()| self.ensure_session_not_terminal(session.session_identity()))
        {
            return Err((session, denial));
        }

        let residue_report = BridgePreviewResidueReport::new(residue_classes);
        if residue_report.authoritative_residue_count() > 0 {
            let denial = BridgeSpeculationError::new(
                BridgeSpeculationErrorKind::PreviewResidueClassificationMismatch,
                format!(
                    "Preview session `{}` cannot discard while residue report `{}` still contains authoritative residue.",
                    session.session_identity().as_str(),
                    residue_report.digest(),
                ),
            );
            return Err((session, denial));
        }

        let counters = BridgeSpeculationCounters::for_discard(
            residue_report.destroyable_residue_count(),
            residue_report.destroyable_residue_count(),
            residue_report.retained_non_authoritative_count(),
        );
        let discard_record = BridgePreviewDiscardRecord::from_active_session(
            &session,
            execution_record,
            residue_report,
            counters,
        );
        let discarded = session.discard();
        self.diagnostics
            .record_preview_discard(discard_record.clone());
        Ok((discarded, discard_record))
    }

    /// Promotes an active preview session with an aspect-native admissibility proof.
    pub fn promote_preview_session(
        &self,
        session: BridgePreviewSession<PreviewActive>,
        execution_record: &BridgePreviewExecutionRecord,
        proof: &BridgePromotionAdmissibilityProof,
    ) -> Result<
        (
            BridgePreviewSession<PreviewPromoted>,
            BridgePreviewPromotionRecord,
        ),
        (BridgePreviewSession<PreviewActive>, BridgeSpeculationError),
    > {
        if let Err(denial) = self
            .ensure_execution_record_matches_active_session(&session, execution_record)
            .and_then(|()| self.ensure_session_not_terminal(session.session_identity()))
        {
            return Err((session, denial));
        }

        if !proof.matches_active_session(&session) {
            let denial = BridgeSpeculationError::new(
                BridgeSpeculationErrorKind::PromotionAdmissibilityMismatch,
                format!(
                    "Promotion proof `{}` did not match active preview session `{}`.",
                    proof.proof_identity().as_str(),
                    session.session_identity().as_str(),
                ),
            );
            return Err((session, denial));
        }

        let authority_basis = match BridgePreviewPromotionAuthorityBasis::from_active_session(
            &session,
            execution_record,
            proof,
        ) {
            Ok(basis) => basis,
            Err(denial) => return Err((session, denial)),
        };
        let counters = BridgeSpeculationCounters::for_promotion(9, 1, 2);
        let promotion_record = BridgePreviewPromotionRecord::from_active_session(
            &session,
            execution_record,
            proof,
            &authority_basis,
            counters,
        );
        let promoted = session.promote(proof)?;
        self.diagnostics
            .record_preview_promotion(promotion_record.clone());
        Ok((promoted, promotion_record))
    }
}
