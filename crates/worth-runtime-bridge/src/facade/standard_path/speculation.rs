use super::*;
use crate::error::BridgeSpeculationError;

#[derive(Debug)]
pub struct BridgeSpeculativeSessionHandle {
    runtime: RuntimeBridge,
    session: BridgePreviewSession<PreviewActive>,
    execution_record: BridgePreviewExecutionRecord,
    liveness: BridgePreviewSessionLivenessOwner,
}

impl BridgeSpeculativeSessionHandle {
    pub(crate) fn new(
        runtime: RuntimeBridge,
        session: BridgePreviewSession<PreviewActive>,
        execution_record: BridgePreviewExecutionRecord,
    ) -> Self {
        let liveness = BridgePreviewSessionLivenessOwner::new(session.session_identity().clone());
        Self {
            runtime,
            session,
            execution_record,
            liveness,
        }
    }

    /// Returns the preview session identity.
    pub fn session_identity(&self) -> &BridgePreviewSessionIdentity {
        self.session.session_identity()
    }

    /// Observes whether this exact Bridge-owned preview session remains active.
    pub fn liveness_observer(&self) -> BridgePreviewSessionLivenessObserver {
        self.liveness.observer()
    }

    /// Builds a comparison handle between this speculative session and main.
    pub fn compare_to_main(&self) -> BridgeSpeculativeComparison {
        BridgeSpeculativeComparison::from_active_session(&self.session)
    }

    /// Returns the retained diagnostics artifact facade.
    pub fn diagnostics(&self) -> &BridgeDiagnosticsFacade {
        self.runtime.diagnostics().retained_artifacts()
    }

    /// Returns the standard-path diagnostics wrapper.
    pub fn inspect(&self) -> BridgeDiagnostics<'_> {
        self.runtime.diagnostics()
    }

    /// Returns the retained execution record for this active session.
    pub fn execution_record(&self) -> &BridgePreviewExecutionRecord {
        &self.execution_record
    }

    /// Discards the active session, returning the same owner on refusal.
    pub fn discard(
        self,
        residue_classes: Vec<BridgePreviewResidueClass>,
    ) -> Result<BridgeSpeculativeDiscardOutcome, (Self, BridgeSpeculationError)> {
        let Self {
            runtime,
            session,
            execution_record,
            liveness,
        } = self;
        match runtime.discard_preview_session(session, &execution_record, residue_classes) {
            Ok((session, record)) => Ok(BridgeSpeculativeDiscardOutcome::new(session, record)),
            Err((session, denial)) => Err((
                Self {
                    runtime,
                    session,
                    execution_record,
                    liveness,
                },
                denial,
            )),
        }
    }

    /// Promotes the active session, returning the same owner on refusal.
    pub fn promote(
        self,
    ) -> Result<BridgeSpeculativePromotionOutcome, (Self, BridgeSpeculationError)> {
        let Self {
            runtime,
            session,
            execution_record,
            liveness,
        } = self;
        let proof = session.promotion_admissibility_proof();
        match runtime.promote_preview_session(session, &execution_record, &proof) {
            Ok((session, record)) => Ok(BridgeSpeculativePromotionOutcome::new(session, record)),
            Err((session, denial)) => Err((
                Self {
                    runtime,
                    session,
                    execution_record,
                    liveness,
                },
                denial,
            )),
        }
    }
}
