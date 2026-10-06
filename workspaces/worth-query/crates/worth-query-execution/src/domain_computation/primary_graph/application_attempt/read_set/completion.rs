use std::marker::PhantomData;

use super::{
    denial, observation_admission, CompletedHandlerFactBoundary, SealedComputationFacts,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationReadAttempt, WorthQueryCompleteApplicationReadSet,
};

impl<Schema, Operation, Input, Scope, Phase>
    WorthQueryApplicationReadAttempt<Schema, Operation, Input, Scope, Phase>
{
    pub fn complete(
        mut self,
    ) -> Result<
        WorthQueryCompleteApplicationReadSet<Schema, Operation, Input, Scope, Phase>,
        WorthQueryApplicationAttemptDenial,
    > {
        if self.facts.len().saturating_add(self.source_facts.len())
            > self
                .admission
                .allowed_graph_contract()
                .decision_fact_budget()
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,
                self.admission.operation(),
            ));
        }
        if self
            .expected_facts
            .as_ref()
            .is_some_and(|expected| !self.facts.keys().eq(expected.iter()))
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DecisionDependencyMismatch,
                self.admission.operation(),
            ));
        }
        let installed_scopes_close_over_facts = self
            .installed_read_scopes
            .iter()
            .zip(self.facts.keys())
            .all(|((scope_key, scope), fact_key)| {
                scope_key == fact_key
                    && observation_admission::graph_read_scope_matches_key(scope, fact_key)
                    && self
                        .admission
                        .allowed_graph_contract()
                        .graph_reads()
                        .roles()
                        .iter()
                        .flat_map(|role| role.read_scopes())
                        .any(|installed| installed == scope)
            });
        if self.installed_read_scopes.len() != self.facts.len()
            || !installed_scopes_close_over_facts
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DecisionDependencyMismatch,
                self.admission.operation(),
            ));
        }
        if self.expected_facts.is_none()
            && !observation_admission::graph_reads_exactly_cover_fact_keys(
                self.admission.allowed_graph_contract().graph_reads(),
                self.facts.keys(),
            )
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::IncompleteDecisionReadSet,
                self.admission.operation(),
            ));
        }
        self.admission
            .mutation_preconditions()
            .validate_observations(&self.facts)
            .map_err(|()| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::MutationPreconditionMismatch,
                    self.admission.operation(),
                )
            })?;
        let (computation_facts, retained) = match self.computation_reads {
            Some((reads, deposit)) => {
                let facts = SealedComputationFacts::at_seal(reads, &self.facts);
                let completed = deposit.and_then(|deposit| {
                    deposit
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .take()
                });
                // A partition carried from the last run is that run's result
                // only if every fact it read is the fact this attempt sealed.
                let retained = match completed {
                    Some(completed) => completed.seal(facts.clone()).map_err(|()| {
                        denial(
                            WorthQueryApplicationAttemptDenialKind::DecisionDependencyMismatch,
                            self.admission.operation(),
                        )
                    })?,
                    None => None,
                };
                (Some(facts), retained)
            }
            None => (None, None),
        };
        self.admission.record_completed_handler_facts(
            CompletedHandlerFactBoundary::from_completed_read(self.facts.len()),
            retained,
        );
        Ok(WorthQueryCompleteApplicationReadSet {
            admission: self.admission,
            lease: self.lease,
            installed_read_scopes: self.installed_read_scopes.into_values().collect(),
            facts: self.facts.into_values().chain(self.source_facts).collect(),
            consumed_outputs: self.consumed_outputs,
            computation_facts,
            workflow_authority_binding: None,
            mutation_handler_binding: None,
            workflow_deadline: None,
            new_commit_refusal: None,
            _phase: PhantomData,
        })
    }
}

impl<Schema, Operation, Input, Scope, Phase>
    WorthQueryCompleteApplicationReadSet<Schema, Operation, Input, Scope, Phase>
{
    /// The facts the owner calls read and which calls read each, when the
    /// handler ran one partitioned computation.
    // Retention takes its own copy at seal; tests read this one.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::domain_computation::primary_graph) fn computation_facts(
        &self,
    ) -> Option<&SealedComputationFacts> {
        self.computation_facts.as_ref()
    }

    /// The facts the decision read, for tests outside the attempt.
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn decision_facts(
        &self,
    ) -> &[super::WorthQueryApplicationObservedFact] {
        &self.facts
    }
}
