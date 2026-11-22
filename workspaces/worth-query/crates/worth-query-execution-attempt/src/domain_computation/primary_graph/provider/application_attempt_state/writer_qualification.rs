//! Qualified use of a producer part (PR C §1.2 R2): its decision facts are
//! keyed for append-time marking and compared exactly from its own post-own
//! image when a later append marks it.
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
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
    pub fn coupled_writer_fact_keys(
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
    pub fn coupled_writer_support_changed(
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
                    match fact.support_equal_after_own_effect(
                        execution, runtime, &snapshot, after_own, current,
                    ) {
                        Ok(true) => {}
                        Ok(false) => return Ok(true),
                        // Unavailable at this image: unknown. The writer is
                        // kept; final support compares this fact strictly.
                        Err(cause) if !crate::domain_computation::primary_graph::provider::application_attempt_state::candidate_read_support::is_hard_comparison_stop(&cause) => {}
                        Err(cause) => return Err(cause),
                    }
                }
                Ok(false)
            })();
            crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            answer
        })
    }
}
