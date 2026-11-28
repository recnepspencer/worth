use super::*;
use crate::source::{
    BridgeAsyncClassifiedDeniedCompletion, BridgeAsyncCompletionSupersessionEvidence,
};

/// A completion that can enter writeback or an already classified denial.
pub enum BridgeAsyncWritebackSubmission<'a> {
    Admitted(BridgeAsyncWritebackAdmissionRequest),
    Superseded(&'a BridgeAsyncClassifiedDeniedCompletion),
}

#[derive(Debug)]
pub enum BridgeAsyncWritebackOutcome {
    Evaluated(BridgeAsyncWritebackCommitReport),
    Superseded(BridgeAsyncCompletionSupersessionEvidence),
}

impl RuntimeBridge {
    /// Stages and commits an admitted completion or returns supersession evidence
    /// without staging a mutation. Admitted effects still pass loop prevention.
    pub fn writeback_async_completion(
        &self,
        submission: BridgeAsyncWritebackSubmission<'_>,
    ) -> Result<BridgeAsyncWritebackOutcome, BridgeAsyncWritebackRejection> {
        let request = match submission {
            BridgeAsyncWritebackSubmission::Admitted(request) => request,
            BridgeAsyncWritebackSubmission::Superseded(classification) => {
                self.require_writeback_completion_owner(
                    classification.denied_completion().request_identity(),
                )?;
                return Ok(BridgeAsyncWritebackOutcome::Superseded(
                    classification.evidence().clone(),
                ));
            }
        };
        let admitted = self.admit_async_writeback(request)?;
        let staged = self.stage_async_writeback_effect(&admitted)?;
        Ok(BridgeAsyncWritebackOutcome::Evaluated(
            self.commit_async_writeback(&staged),
        ))
    }

    fn require_writeback_completion_owner(
        &self,
        request: &AdmittedBridgeAsyncRequestIdentity,
    ) -> Result<(), BridgeAsyncWritebackRejection> {
        if request.bridge_runtime_key() != self.signal_runtime_key {
            return Err(BridgeAsyncWritebackRejection::new(
                BridgeAsyncWritebackRejectionKind::CurrentAuthorityDrifted,
                "async completion belongs to a different Bridge runtime",
            ));
        }
        Ok(())
    }

    /// Admits one async completion into the authoritative writeback family
    /// boundary without yet staging or executing relational mutation.
    pub fn admit_async_writeback(
        &self,
        request: BridgeAsyncWritebackAdmissionRequest,
    ) -> Result<AdmittedBridgeAsyncWriteback, BridgeAsyncWritebackRejection> {
        self.require_writeback_completion_owner(request.completion().request_identity())?;
        AdmittedBridgeAsyncWriteback::admit(
            request,
            self.policy.diagnostics_tier(),
            self.policy.allow_replay_artifacts(),
            self.policy.record_route_artifacts(),
        )
    }

    /// Produces the staged writeback effect proof over the generic writeback
    /// engine for one already-admitted async completion writeback request.
    pub fn stage_async_writeback_effect(
        &self,
        writeback: &AdmittedBridgeAsyncWriteback,
    ) -> Result<StagedBridgeAsyncWritebackEffect, BridgeAsyncWritebackRejection> {
        self.require_writeback_completion_owner(writeback.request_identity())?;
        StagedBridgeAsyncWritebackEffect::stage(self, writeback)
    }

    /// Commits one staged async writeback effect and returns explicit commit,
    /// noop, or rejection artifacts.
    pub fn commit_async_writeback(
        &self,
        staged: &StagedBridgeAsyncWritebackEffect,
    ) -> BridgeAsyncWritebackCommitReport {
        BridgeAsyncWritebackCommitReport::commit(self, staged)
    }
}
