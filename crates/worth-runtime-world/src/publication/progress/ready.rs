use super::*;

impl CompositeAttemptProgress {
    pub(crate) fn into_ready_results(
        self,
    ) -> Result<(Self, crate::publication::CompositeOwnerExecutionResults), Self> {
        self.ready_results().ok_or(self)
    }

    /// A read-only projection of settled evidence. The owner registry can keep
    /// the original progress while a linear phase borrows its result view.
    pub(crate) fn ready_results(
        &self,
    ) -> Option<(Self, crate::publication::CompositeOwnerExecutionResults)> {
        if self.owner_effect_count() == 0 {
            return None;
        }
        let relational_result = self.relational.ready_result()?;
        let signal_result = self.signal.ready_result()?;
        let summary = Self {
            relational: RelationalAttemptProgress {
                posture: self.relational.posture,
                evidence: None,
                fork: None,
                fork_successor_basis: None,
            },
            signal: SignalAttemptProgress::summary(self.signal.posture),
        };
        Some((
            summary,
            crate::publication::CompositeOwnerExecutionResults::from_components(
                relational_result,
                signal_result,
            ),
        ))
    }
}

impl RelationalAttemptProgress {
    fn ready_result(&self) -> Option<crate::publication::CompositeRelationalOwnerResult> {
        use crate::publication::CompositeRelationalOwnerResult as ResultEvidence;
        match (
            self.posture,
            &self.evidence,
            &self.fork,
            &self.fork_successor_basis,
        ) {
            (RelationalAttemptProgressPosture::Untouched, None, None, None) => {
                Some(ResultEvidence::retained())
            }
            (RelationalAttemptProgressPosture::Performed, None, Some(fork), Some(basis)) => {
                Some(ResultEvidence::forked(fork.clone(), basis.clone()))
            }
            (
                RelationalAttemptProgressPosture::Settled,
                Some(RelationalProgressEvidence::Settled {
                    commit_identity,
                    successor_basis,
                    result,
                }),
                None,
                None,
            ) => Some(ResultEvidence::settled(
                commit_identity.clone(),
                successor_basis.clone(),
                std::sync::Arc::clone(result),
            )),
            _ => None,
        }
    }
}

impl SignalAttemptProgress {
    fn ready_result(&self) -> Option<crate::publication::CompositeSignalOwnerResult> {
        use crate::publication::CompositeSignalOwnerResult as ResultEvidence;
        match (self.posture, &self.evidence) {
            (SignalAttemptProgressPosture::Untouched, None) => Some(ResultEvidence::retained()),
            (
                SignalAttemptProgressPosture::Performed,
                Some(SignalProgressEvidence::Advanced(outcome)),
            ) => Some(ResultEvidence::advanced(std::sync::Arc::clone(outcome))),
            (
                SignalAttemptProgressPosture::Performed,
                Some(SignalProgressEvidence::Forked(outcome)),
            ) => Some(ResultEvidence::forked(outcome.clone())),
            _ => None,
        }
    }
}
