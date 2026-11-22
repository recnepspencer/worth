//! Qualified use of a producer part (PR C §1.2 R2): its decision facts are
//! keyed for append-time marking and compared exactly from its own post-own
//! image when a later append marks it.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::fact::touch_key::{
    self, FactTouchKey,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationDecisionFact as Fact;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
};

impl super::super::WorthQueryPrimaryGraphProvider {
    /// The touch keys of every decision fact `session`'s writer read
    /// (`None` for a conservative fact).
    pub(in crate::domain_computation::primary_graph) fn coupled_writer_fact_keys(
        &self,
        session: WorthQueryProviderSessionView<'_>,
    ) -> Vec<Option<Vec<FactTouchKey>>> {
        let attempts = self
            .attempts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let attempt = attempts
            .attempt_state(session)
            .expect("actual appended writer")
            .attempt();
        attempt
            .facts()
            .values()
            .filter_map(|fact| match fact {
                Fact::Application { fact, .. }
                | Fact::ObservedSource { fact }
                | Fact::ApplicationObservedSource { fact, .. } => {
                    Some(touch_key::fact_keys(std::ops::Deref::deref(&fact)))
                }
                Fact::Principal(_) | Fact::Authorization { .. } => None,
            })
            .collect()
    }

    /// Whether a decision fact of `session`'s writer differs between its own
    /// post-own image and `current`, against the edit's C basis.
    pub(in crate::domain_computation::primary_graph) fn coupled_writer_support_changed(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session: WorthQueryProviderSessionView<'_>,
        before: &crate::basis::WorthQueryProductBranchLease,
        current: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    ) -> Result<bool, Denial> {
        self.graph.with_runtime(execution, |runtime| {
            let snapshot =
                crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(
                    runtime,
                    before.relational_basis(),
                )
                .map_err(|_| Denial::new(Kind::StorageUnavailable, "coupled qualification C basis"))?;
            let answer = (|| {
                let attempts = self
                    .attempts
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let attempt = attempts
                    .attempt_state(session)
                    .expect("actual appended writer")
                    .attempt();
                let after_own = attempt
                    .coupled_after_own()
                    .expect("Native completed this writer before continuing");
                for fact in attempt.facts().values() {
                    let fact = match fact {
                        Fact::Application { fact, .. }
                        | Fact::ObservedSource { fact }
                        | Fact::ApplicationObservedSource { fact, .. } => fact,
                        Fact::Principal(_) | Fact::Authorization { .. } => continue,
                    };
                    if !fact.support_equal_after_own_effect(
                        execution, runtime, &snapshot, after_own, current,
                    )? {
                        return Ok(true);
                    }
                }
                Ok(false)
            })();
            crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            answer
        })
    }
}
