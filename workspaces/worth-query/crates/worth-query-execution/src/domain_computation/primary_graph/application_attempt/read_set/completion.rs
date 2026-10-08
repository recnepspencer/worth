use crate::domain_computation::primary_graph::application_attempt::check_request_live;
use std::marker::PhantomData;
use worth_execution::ExecutionAllocationPolicy;

use super::{
    denial, observation_admission, CompletedHandlerFactBoundary,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationReadAttempt, WorthQueryCompleteApplicationReadSet,
};

impl<Schema, Operation, Input, Scope, Phase>
    WorthQueryApplicationReadAttempt<Schema, Operation, Input, Scope, Phase>
{
    pub fn complete(
        mut self,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryCompleteApplicationReadSet<Schema, Operation, Input, Scope, Phase>,
        WorthQueryApplicationAttemptDenial,
    > {
        check_request_live(
            self.admission.publication_request(),
            self.admission.operation(),
        )?;
        if self
            .expected_facts
            .as_ref()
            .is_some_and(|expected| !self.facts.keys().eq(expected.iter()))
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DecisionDependencyMismatch,
                format!(
                    "{}: projected decision fact keys differ",
                    self.admission.operation()
                ),
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
                format!(
                    "{}: decision facts do not close over installed read scopes",
                    self.admission.operation()
                ),
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
        let operation = self.admission.operation();
        let fact_count = self
            .facts
            .len()
            .checked_add(self.source_facts.len())
            .ok_or_else(|| {
                super::super::retained_decision_facts::StoreDenial::Representability
                    .into_attempt_denial(operation)
            })?;
        let check_authority = || {
            self.admission
                .validate_current_authority()
                .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)
        };
        let installed_read_scopes = super::admit_array(
            self.installed_read_scopes.len(),
            self.installed_read_scopes.into_values(),
            allocation_policy,
            operation,
            check_authority,
        )?;
        let facts = super::admit_array(
            fact_count,
            self.facts
                .into_values()
                .chain(self.source_facts.into_values()),
            allocation_policy,
            operation,
            check_authority,
        )?;
        Ok(WorthQueryCompleteApplicationReadSet {
            admission: self.admission,
            lease: self.lease,
            installed_read_scopes,
            facts,
            consumed_outputs: self.consumed_outputs,
            workflow_authority_binding: None,
            mutation_handler_binding: None,
            workflow_deadline: None,
            new_commit_refusal: None,
            _phase: PhantomData,
        })
    }
}
