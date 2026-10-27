use crate::domain_computation::primary_graph::application_attempt::check_request_live;
use std::marker::PhantomData;
use worth_execution::ExecutionAllocationPolicy;

use super::{
    denial, observation_admission, CompletedHandlerFactBoundary, SealedComputationFacts,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationReadAttempt, WorthQueryCompleteApplicationReadSet,
};

impl<Schema, Operation, Input, Scope, Phase>
    WorthQueryApplicationReadAttempt<'_, '_, Schema, Operation, Input, Scope, Phase>
{
    pub fn complete(
        mut self,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryCompleteApplicationReadSet<Schema, Operation, Input, Scope, Phase>,
        WorthQueryApplicationAttemptDenial,
    > {
        self.check_completion_live()?;
        if let Some(refresh) = self.native_completion.take() {
            return self.complete_native(refresh, allocation_policy);
        }
        self.complete_full(allocation_policy)
    }

    pub(super) fn check_completion_live(&self) -> Result<(), WorthQueryApplicationAttemptDenial> {
        // No ignored ordinary decode refusal can seal/attribute facts, retain
        // completed input, or advance to an effect/candidate owner.
        self.check_value_decode()?;
        self.execution.consult().map_err(|cause| {
            denial(
                WorthQueryApplicationAttemptDenialKind::ValueDecode(
                    super::super::super::value_decode::WorthQueryValueDecodeDenial::Scope(cause),
                ),
                self.admission.operation(),
            )
        })?;
        check_request_live(
            self.admission.publication_request(),
            self.admission.operation(),
        )?;
        Ok(())
    }

    fn complete_full(
        mut self,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryCompleteApplicationReadSet<Schema, Operation, Input, Scope, Phase>,
        WorthQueryApplicationAttemptDenial,
    > {
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
            .zip(self.facts.iter())
            .all(|((scope_key, scope), (fact_key, fact))| {
                if scope_key != fact_key {
                    return false;
                }
                if let (
                    super::super::fact::WorthQueryApplicationFactKey::Aggregate(key),
                    super::super::fact::WorthQueryApplicationObservedFact::Aggregate {
                        observation,
                    },
                ) = (fact_key, fact)
                {
                    if !self
                        .expected_facts
                        .as_ref()
                        .and_then(|expected| {
                            expected.aggregate(
                                key,
                                self.admission.admission_identity(),
                                &self.admission.retain_graph_contract(),
                                self.lease.snapshot(),
                            )
                        })
                        .is_some_and(|issued| std::sync::Arc::ptr_eq(issued, observation))
                    {
                        return false;
                    }
                }
                if let (
                    super::super::fact::WorthQueryApplicationFactKey::StructuralWalk(key),
                    super::super::fact::WorthQueryApplicationObservedFact::StructuralWalk {
                        observation,
                    },
                ) = (fact_key, fact)
                {
                    if !self
                        .expected_facts
                        .as_ref()
                        .and_then(|expected| {
                            expected.structural_walk(
                                key,
                                self.admission.admission_identity(),
                                &self.admission.retain_graph_contract(),
                                self.lease.snapshot(),
                            )
                        })
                        .is_some_and(|issued| std::sync::Arc::ptr_eq(issued, observation))
                    {
                        return false;
                    }
                }
                if let (
                    super::InstalledScopePosition::SourceQuery,
                    super::super::fact::WorthQueryApplicationFactKey::SourcePublication(read),
                    super::super::fact::WorthQueryApplicationObservedFact::SourcePublication {
                        observation,
                    },
                ) = (scope, fact_key, fact)
                {
                    // The tag/key cannot grant Query authority. Consume only
                    // the exact native read retained by this original reader.
                    return read
                        .admits_observation(self.admission.admission_identity(), observation)
                        && self
                            .expected_facts
                            .as_ref()
                            .and_then(|expected| expected.source_publication(read))
                            .is_some_and(|issued| std::sync::Arc::ptr_eq(issued, observation));
                }
                scope
                    .resolve(self.admission.allowed_graph_contract())
                    .is_some_and(|scope| {
                        observation_admission::graph_read_scope_matches_key(scope, fact_key)
                    })
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
        allocation_policy.check_live().map_err(|cause| {
            WorthQueryApplicationAttemptDenial::allocation_denied(self.admission.operation(), cause)
        })?;
        // The expected-key view and attribution shared the projection's maps.
        // Close the view before ownership moves into retained metadata.
        drop(self.expected_facts.take());
        let (reads, mut completed) = self.computation_reads;
        let computation_fact_ordinals: std::sync::Arc<[usize]> = reads.ordinals(&self.facts).into();
        let attribution = reads.into_attribution();
        let _temporary_index_memory = completed.reserve_fact_index(|| attribution.as_ref().map(|reads| {
            reads.index_bytes().ok_or(
                crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::CapacityOverflow,
            )
        }).transpose(), allocation_policy)
            .map_err(|resource| denial(
                WorthQueryApplicationAttemptDenialKind::ExecutionRequest(
                    crate::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource {
                        denial: resource, partition_identity: None, policy_ancestor: None,
                    },
                ), self.admission.operation(),
            ))?;
        let computation_facts = attribution
            .map(|reads| {
                SealedComputationFacts::prepare(reads, self.facts.keys(), allocation_policy)
            })
            .transpose()
            .map_err(|cause| cause.into_attempt_denial(self.admission.operation()))?;
        let handler_fact_count = self.facts.len() + self.source_facts.handler_len();
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
        let facts = super::admit_completed_facts(
            fact_count,
            self.facts
                .into_values()
                .chain(self.source_facts.into_values()),
            allocation_policy,
            self.admission.publication_request(),
            operation,
            check_authority,
        )?;
        let installed_read_scopes = super::CompletedReadScopes::from_completed(
            self.admission.retain_graph_contract(),
            installed_read_scopes,
        );
        let control = super::super::retained_decision_facts::StorageControl::new(
            allocation_policy,
            Some(self.admission.publication_request()),
        );
        let application_projection =
            super::CompletedFactProjection::build(&facts, installed_read_scopes.clone(), control)
                .map_err(|cause| cause.into_attempt_denial(operation))?;
        let content = if computation_facts.is_some() {
            let control = super::super::retained_decision_facts::StorageControl::new(
                allocation_policy,
                Some(self.admission.publication_request()),
            );
            let (content, work) = super::super::FactContent::build(facts.clone(), control)
                .map_err(|cause| cause.into_attempt_denial(operation))?;
            self.admission.retain_execution_canonical_work(work);
            Some(content)
        } else {
            None
        };
        let computation_facts = computation_facts.map(|metadata| {
            let mut bound = metadata.bind(
                facts.clone(),
                self.admission.retain_graph_contract(),
                installed_read_scopes.clone(),
                self.admission.scope_entity_id(),
                std::sync::Arc::clone(&computation_fact_ordinals),
                content.clone(),
            );
            bound.retain_projection(application_projection.clone());
            bound
        });
        // Original facts and their checked metadata exist before retention seals;
        // postcommit facts are authored later into a distinct carrier.
        let retained = completed
            .seal_at(computation_facts.clone(), self.lease.snapshot())
            .map_err(|()| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::DecisionDependencyMismatch,
                    self.admission.operation(),
                )
            })?;
        self.admission.record_completed_handler_facts(
            CompletedHandlerFactBoundary::from_completed_read(handler_fact_count),
            retained,
        );
        let effect_routes = match content.as_ref().and_then(|c| c.effect_routes(&facts)) {
            Some(routes) => routes,
            None => super::super::CompletedFactRoutes::from_facts(
                facts.clone(),
                super::super::retained_decision_facts::StorageControl::new(
                    allocation_policy,
                    Some(self.admission.publication_request()),
                ),
            )
            .map_err(|cause| cause.into_attempt_denial(self.admission.operation()))?,
        };
        Ok(WorthQueryCompleteApplicationReadSet {
            postcondition_carry: None,
            admission: self.admission,
            lease: self.lease,
            application_projection: Some(application_projection),
            original_read: None,
            installed_read_scopes,
            facts,
            content,
            effect_routes,
            consumed_outputs: self.consumed_outputs,
            #[cfg(test)]
            computation_facts,
            computation_fact_ordinals,
            workflow_authority_binding: None,
            workflow_birth_binding: None,
            workflow_succession_binding: None,
            mutation_handler_binding: None,
            local_action_settlement: None,
            body_external_emission: None,
            signed_creation: None,
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
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn computation_facts(
        &self,
    ) -> Option<&SealedComputationFacts> {
        self.computation_facts.as_ref()
    }

    /// The facts the decision read, for tests outside the attempt.
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn decision_facts(
        &self,
    ) -> Vec<super::WorthQueryApplicationObservedFact> {
        self.facts.iter().cloned().collect()
    }
}
