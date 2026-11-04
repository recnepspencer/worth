//! Checked fixed-shape facts reach ordinary completion through a funded patch.
use super::complete_read_set_core::{
    CompleteReadSetCore, CompletedReadFacts, NativeReadCompletion, PreparedReadCompletion,
};
use super::computation_routing::NativeReadRefresh;
use super::mutation_read_attempt_core::MutationReadAttemptCore;
use super::*;
use crate::domain_computation::primary_graph::application_attempt::{
    check_request_live,
    retained_decision_facts::{StorageControl, StoreDenial},
};
use worth_execution::ExecutionAllocationPolicy;

impl MutationReadAttemptCore<'_, '_, '_> {
    pub(super) fn complete_native(
        self,
        refresh: NativeReadRefresh,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<PreparedReadCompletion, WorthQueryApplicationAttemptDenial> {
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
        let source_values =
            std::mem::replace(self.source_facts, SourceFacts::Admitted(Vec::new())).into_values();
        let source = super::admit_completed_facts(
            source_count,
            source_values,
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
        for (key, fact) in self.facts.iter() {
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
        let (inputs, completed) = std::mem::replace(self.computation_reads, (
            crate::domain_computation::primary_graph::invariant_projection::ObservedComputationInputs::None,
            crate::domain_computation::primary_graph::application_contribution::CompletedComputationRetention::Absent(
                crate::domain_computation::primary_graph::application_contribution::PriorAbsence::NotProduced),
        ));
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
        Ok(PreparedReadCompletion::Native(NativeReadCompletion {
            read: CompletedReadFacts {
                postcondition_carry,
                application_projection: Some(application_projection),
                original_read: None,
                installed_read_scopes,
                facts,
                content,
                #[cfg(test)]
                computation_facts: Some(metadata),
                computation_fact_ordinals,
            },
            retained,
            handler_count,
            content_work,
        }))
    }
}

impl NativeReadCompletion {
    pub(super) fn finish(
        mut self,
        mut admission: crate::domain_computation::authorization::ReadCompletionAdmission<'_>,
        lease: WorthQueryApplicationSnapshotLease,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        (CompleteReadSetCore, WorthQueryApplicationSnapshotLease),
        WorthQueryApplicationAttemptDenial,
    > {
        let facts = &self.read.facts;
        let application_projection = self
            .read
            .application_projection
            .as_ref()
            .expect("native completion projection");
        let operation = admission.operation();
        let retained = self.retained;
        let handler_count = self.handler_count;
        let content_work = self.content_work;
        let content = &self.read.content;
        let control = StorageControl::new(policy, Some(admission.publication_request()));
        let (lease, original_read) = super::CompletedOriginalRead::from_native_completion(
            facts.clone(),
            application_projection.clone(),
            lease,
            &admission,
            control,
        )
        .map_err(|cause| cause.into_attempt_denial(operation))?;
        admission.record_completed_handler_facts(
            CompletedHandlerFactBoundary::from_completed_read(handler_count),
            retained,
        );
        admission.retain_execution_canonical_work(content_work);
        let effect_routes = match content.as_ref().and_then(|c| c.effect_routes(facts)) {
            Some(routes) => routes,
            None => super::super::CompletedFactRoutes::from_facts(
                facts.clone(),
                super::super::retained_decision_facts::StorageControl::new(
                    policy,
                    Some(admission.publication_request()),
                ),
            )
            .map_err(|cause| cause.into_attempt_denial(admission.operation()))?,
        };
        self.read.original_read = original_read;
        Ok((
            CompleteReadSetCore {
                read: self.read,
                effect_routes,
            },
            lease,
        ))
    }
}
