use super::*;

impl PulseExecutableWorld<Published<FinalRecovered>> {
    pub(crate) fn evidence(
        &self,
    ) -> &crate::adjudication::ExecutableSchemaTransitionEvidence<StatusSchemaRecoverySourceDelta>
    {
        &self.state.stage.evidence
    }

    pub(crate) fn stopped_evidence(
        &self,
    ) -> &crate::adjudication::ExecutableSchemaTransitionEvidence<RevisionSchemaSourceDelta> {
        &self.state.stage.stopped.evidence
    }

    pub(crate) fn rebase_snapshot_evidence(
        &self,
    ) -> &crate::adjudication::ExecutableVisualSnapshotEvidence {
        &self.state.stage.rebase_snapshot
    }

    pub(crate) fn stopped_snapshot_evidence(
        &self,
    ) -> &crate::adjudication::ExecutableVisualSnapshotEvidence {
        &self.state.stage.stopped.snapshot
    }

    pub(crate) fn source_action_count(&self) -> u32 {
        let green = &self.state.stage.stopped.recovered.preserved.green;
        green.initial.prior.action.action_count()
            + green.initial.visual().initial().action.action_count()
            + green.evidence.action().action_count()
            + self
                .state
                .stage
                .stopped
                .recovered
                .preserved
                .evidence
                .action()
                .action_count()
            + self
                .state
                .stage
                .stopped
                .recovered
                .evidence
                .action()
                .action_count()
            + self
                .state
                .stage
                .stopped
                .evidence
                .replacement()
                .action()
                .action_count()
            + self
                .state
                .stage
                .evidence
                .replacement()
                .action()
                .action_count()
    }
}
