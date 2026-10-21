//! Checked fixed-shape facts reach ordinary completion through a funded patch.
use super::computation_routing::NativeReadRefresh;
use super::*;
use crate::domain_computation::primary_graph::application_attempt::{
    check_request_live,
    retained_decision_facts::{StorageControl, StoreDenial},
};
use worth_execution::ExecutionAllocationPolicy;

impl<Schema, Operation, Input, Scope, Phase>
    WorthQueryApplicationReadAttempt<'_, '_, Schema, Operation, Input, Scope, Phase>
{
    pub(super) fn complete_native(
        mut self,
        refresh: NativeReadRefresh,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryCompleteApplicationReadSet<Schema, Operation, Input, Scope, Phase>,
        WorthQueryApplicationAttemptDenial,
    > {
        self.check_completion_live()?;
        let expected = self
            .expected_facts
            .as_ref()
            .expect("native projected completion");
        if !self.facts.keys().eq(expected.refresh_keys())
            || self.installed_read_scopes.len() != self.facts.len()
            || !self.installed_read_scopes.iter().all(|(key, position)| {
                self.facts.contains_key(key)
                    && position
                        .resolve(self.admission.allowed_graph_contract())
                        .is_some_and(|scope| {
                            observation_admission::graph_read_scope_matches_key(scope, key)
                        })
            })
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DecisionDependencyMismatch,
                self.admission.operation(),
            ));
        }
        self.admission
            .mutation_preconditions()
            .validate_observations_with(|key| self.facts.get(key).or_else(|| refresh.fact(key)))
            .map_err(|()| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::MutationPreconditionMismatch,
                    self.admission.operation(),
                )
            })?;
        let handler_sources = self.source_facts.handler_len();
        let source_count = self.source_facts.len();
        let operation = self.admission.operation();
        let check_authority = || {
            self.admission
                .validate_current_authority()
                .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)
        };
        let source = super::admit_completed_facts(
            source_count,
            self.source_facts.into_values(),
            policy,
            self.admission.publication_request(),
            operation,
            check_authority,
        )?;
        let control = StorageControl::new(policy, Some(self.admission.publication_request()));
        let base = refresh
            .decisions()
            .append_completed(source, control)
            .map_err(|cause| cause.into_attempt_denial(operation))?;
        let changed: im::OrdSet<_> = self
            .facts
            .keys()
            .map(|key| {
                refresh
                    .ordinal(key)
                    .ok_or_else(|| StoreDenial::ConflictingBody.into_attempt_denial(operation))
            })
            .collect::<Result<_, _>>()?;
        let mut patch = base
            .prepare_native_patch(&changed, control)
            .map_err(|cause| cause.into_attempt_denial(operation))?;
        for (key, fact) in &self.facts {
            check_authority()?;
            patch
                .replace(
                    refresh.ordinal(key).expect("checked refresh ordinal"),
                    fact,
                    control,
                )
                .map_err(|cause| cause.into_attempt_denial(operation))?;
        }
        let facts = patch
            .seal(control)
            .map_err(|cause| cause.into_attempt_denial(operation))?;
        drop(base);
        drop(self.expected_facts.take());
        let (inputs, completed) = self.computation_reads;
        drop(inputs);
        let (content, content_work) = match refresh.content() {
            Some(prior) => {
                let decisions = refresh.decisions();
                // Original source suffix is replaced, not confused with the
                // new source authority. Every fresh suffix ordinal is derived.
                let mut content_changed = changed.clone();
                content_changed.extend(decisions.len()..facts.len());
                let (content, work) = prior
                    .refresh(facts.clone(), content_changed, control)
                    .map_err(|cause| cause.into_attempt_denial(operation))?;
                (Some(content), work)
            }
            None => (
                None,
                worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero(),
            ),
        };
        let postcondition_carry = refresh
            .postcondition_carry(&changed, self.lease.snapshot(), control)
            .map_err(|cause| cause.into_attempt_denial(operation))?;
        let application_projection = refresh
            .projection(&facts, &changed, control)
            .map_err(|cause| cause.into_attempt_denial(operation))?;
        let (metadata, installed_read_scopes, computation_fact_ordinals, decision_count) = refresh
            .finish(
                facts.clone(),
                content.clone(),
                application_projection.clone(),
            );
        check_request_live(self.admission.publication_request(), operation)?;
        let retained = completed
            .seal_at(Some(metadata.clone()), self.lease.snapshot())
            .map_err(|()| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::DecisionDependencyMismatch,
                    operation,
                )
            })?;
        let handler_count = decision_count
            .checked_add(handler_sources)
            .ok_or_else(|| StoreDenial::Representability.into_attempt_denial(operation))?;
        let (lease, original_read) = super::CompletedOriginalRead::from_native_completion(
            facts.clone(),
            application_projection.clone(),
            self.lease,
            &self.admission,
            control,
        )
        .map_err(|cause| cause.into_attempt_denial(operation))?;
        self.admission.record_completed_handler_facts(
            CompletedHandlerFactBoundary::from_completed_read(handler_count),
            retained,
        );
        self.admission.retain_execution_canonical_work(content_work);
        let effect_routes = match content.as_ref().and_then(|c| c.effect_routes(&facts)) {
            Some(routes) => routes,
            None => super::super::CompletedFactRoutes::from_facts(
                facts.clone(),
                super::super::retained_decision_facts::StorageControl::new(
                    policy,
                    Some(self.admission.publication_request()),
                ),
            )
            .map_err(|cause| cause.into_attempt_denial(self.admission.operation()))?,
        };
        Ok(WorthQueryCompleteApplicationReadSet {
            postcondition_carry,
            admission: self.admission,
            lease,
            application_projection: Some(application_projection),
            original_read,
            installed_read_scopes,
            facts,
            content,
            effect_routes,
            consumed_outputs: self.consumed_outputs,
            #[cfg(test)]
            computation_facts: Some(metadata),
            computation_fact_ordinals,
            workflow_authority_binding: None,
            mutation_handler_binding: None,
            workflow_deadline: None,
            new_commit_refusal: None,
            signed_creation: None,
            workflow_birth_binding: None,
            workflow_succession_binding: None,
            local_action_settlement: None,
            body_external_emission: None,
            _phase: PhantomData,
        })
    }
}
