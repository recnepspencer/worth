use super::*;

impl RelationalInvariantExecutionMeaning {
    pub(crate) fn is_consistent(&self) -> bool {
        // Match the actual request profiles exhaustively. The production
        // proposed-state aggregate retains its committed before/applied view.
        let valid_observation = match (
            self.metadata.execution_point,
            self.metadata.observation_kind,
        ) {
            (
                InvariantExecutionPoint::ProposedState
                | InvariantExecutionPoint::CommitBoundary
                | InvariantExecutionPoint::HarnessAudit,
                InvariantObservationKind::Committed,
            ) => true,
            (
                InvariantExecutionPoint::ProposedState
                | InvariantExecutionPoint::CommitBoundary
                | InvariantExecutionPoint::HarnessAudit,
                InvariantObservationKind::Speculative,
            ) => false,
            (
                InvariantExecutionPoint::MutationSensitive
                | InvariantExecutionPoint::SnapshotPublication
                | InvariantExecutionPoint::CertificationBoundary
                | InvariantExecutionPoint::GraphComposition,
                InvariantObservationKind::Committed | InvariantObservationKind::Speculative,
            ) => true,
        };
        if !valid_observation
            || self.checks.len() != self.decisions.len()
            || (self.metadata.disposition != InvariantExecutionDisposition::Executed
                && !self.checks.is_empty())
        {
            return false;
        }
        self.checks
            .iter()
            .zip(&self.decisions)
            .all(|(check, decision)| {
                let observation = match check.execution_point {
                    InvariantExecutionPoint::MutationSensitive
                    | InvariantExecutionPoint::SnapshotPublication
                        if self.metadata.execution_point
                            == InvariantExecutionPoint::ProposedState =>
                    {
                        InvariantObservationKind::Speculative
                    }
                    InvariantExecutionPoint::CommitBoundary
                    | InvariantExecutionPoint::MutationSensitive
                    | InvariantExecutionPoint::SnapshotPublication
                    | InvariantExecutionPoint::CertificationBoundary
                    | InvariantExecutionPoint::HarnessAudit
                    | InvariantExecutionPoint::GraphComposition => self.metadata.observation_kind,
                    // ProposedState reports an aggregate; it never declares a rule.
                    InvariantExecutionPoint::ProposedState => return false,
                };
                self.metadata
                    .execution_point
                    .includes(check.execution_point)
                    && super::production::references::is_canonical(&check.verdict)
                    && check.execution_point == decision.execution_point
                    && check.failure_effect == decision.failure_effect
                    && check.rule == decision.rule
                    && check.witness == decision.witness
                    && check.groups == decision.groups
                    && check.declared_cost == decision.declared_cost
                    && check.custom_provenance.is_some() == decision.custom_provenance_present
                    && decision.decision == InvariantDecisionKind::from_verdict(&check.verdict)
                    && check.custom_provenance.as_ref().is_none_or(|custom| {
                        // Custom enforcement can name the proposed version while
                        // aggregate metadata retains the original request version.
                        custom.observation_kind == observation
                            && custom.current_version == self.metadata.current_version
                    })
            })
    }
}
