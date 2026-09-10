use super::super::SignalGraph;
impl SignalGraph {
    pub(crate) fn prepare_invalidation_performed_work(
        &self,
        binding: &crate::data::proof::invalidation::progression::InvalidationWorkBindingAxes,
        work: &mut crate::logic::evaluation::EvaluationWork<'_>,
    ) -> Result<Option<super::PreparedPerformedWorkCapture>, crate::data::error::SignalError> {
        use crate::logic::evaluation::EvaluationWork;
        let ledger = self.arena.retained_node_ledger.as_ref();
        match work {
            EvaluationWork::Ordinary if ledger.is_some() => {
                let mut allowance = crate::data::retained_storage::RetainedStoragePreparation::new(
                    self.installed_runtime_policy()
                        .conditional_evaluation_budget()
                        .maximum_attempt_visits,
                );
                self.invalidation_performed_work.prepare(
                    binding,
                    ledger,
                    &mut EvaluationWork::Conditional(&mut allowance),
                )
            }
            _ => self
                .invalidation_performed_work
                .prepare(binding, ledger, work),
        }
    }

    pub(crate) fn snapshot_invalidation_performed_targets(
        &self,
        captures_work: bool,
        work: &mut crate::logic::evaluation::EvaluationWork<'_>,
    ) -> Result<super::PerformedTargetSnapshot, crate::data::error::SignalError> {
        use crate::logic::evaluation::EvaluationWork;
        let ledger = self.arena.retained_node_ledger.as_ref();
        let snapshot = |work: &mut EvaluationWork<'_>| {
            if captures_work {
                self.invalidation_performed_work
                    .snapshot_targets(ledger, work)
            } else {
                super::PerformedTargetSnapshot::empty(ledger, work)
            }
        };
        match work {
            EvaluationWork::Ordinary if ledger.is_some() => {
                let mut allowance = crate::data::retained_storage::RetainedStoragePreparation::new(
                    self.installed_runtime_policy()
                        .conditional_evaluation_budget()
                        .maximum_attempt_visits,
                );
                snapshot(&mut EvaluationWork::Conditional(&mut allowance))
            }
            _ => snapshot(work),
        }
    }
}
