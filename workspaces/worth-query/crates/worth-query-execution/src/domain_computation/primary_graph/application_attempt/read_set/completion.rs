use std::marker::PhantomData;

use super::{
    denial, observation_admission, CompletedHandlerFactBoundary, ComputationFactRouting,
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
        self.admission.record_completed_handler_facts(
            CompletedHandlerFactBoundary::from_completed_read(self.facts.len()),
        );
        let computation_routing = self
            .computation_reads
            .map(|reads| ComputationFactRouting::at_seal(reads, self.facts.keys()));
        Ok(WorthQueryCompleteApplicationReadSet {
            admission: self.admission,
            lease: self.lease,
            installed_read_scopes: self.installed_read_scopes.into_values().collect(),
            facts: self.facts.into_values().chain(self.source_facts).collect(),
            consumed_outputs: self.consumed_outputs,
            computation_routing,
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
    /// What read each handler fact, when the handler ran one partitioned
    /// computation.
    // Nothing routes a mark yet: tests read the table until retention does.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::domain_computation::primary_graph) fn computation_routing(
        &self,
    ) -> Option<&ComputationFactRouting> {
        self.computation_routing.as_ref()
    }
}
