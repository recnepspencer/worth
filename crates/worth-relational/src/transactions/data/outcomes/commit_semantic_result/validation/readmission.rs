use super::*;

impl RelationalInvariantExecutionMeaning {
    pub(crate) fn is_consistent(&self) -> bool {
        if self.checks.len() != self.decisions.len()
            || (self.metadata.disposition != InvariantExecutionDisposition::Executed
                && !self.checks.is_empty())
        {
            return false;
        }
        self.checks
            .iter()
            .zip(&self.decisions)
            .all(|(check, decision)| {
                check.execution_point == self.metadata.execution_point
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
                        custom.observation_kind == self.metadata.observation_kind
                            && custom.version == self.metadata.version
                            && custom.current_version == self.metadata.current_version
                    })
            })
    }
}
