//! Full read completion over the original borrowed admission and read state.
use super::complete_read_set_core::{
    CompleteReadSetCore, CompletedReadFacts, PreparedReadCompletion,
};
use super::mutation_read_attempt_core::MutationReadAttemptCore;
use super::{
    denial, observation_admission, CompletedHandlerFactBoundary, SealedComputationFacts,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryCompleteApplicationReadSet,
};
use worth_execution::ExecutionAllocationPolicy;

impl MutationReadAttemptCore<'_, '_, '_> {
    pub(super) fn complete_full(
        mut self,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<PreparedReadCompletion, WorthQueryApplicationAttemptDenial> {
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
                    super::super::fact::WorthQueryApplicationFactKey::ComponentSelection(key),
                    super::super::fact::WorthQueryApplicationObservedFact::ComponentSelection {
                        observation,
                    },
                ) = (fact_key, fact)
                    && !self
                        .expected_facts
                        .as_ref()
                        .and_then(|expected| {
                            expected.component(
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
                if let (
                    super::super::fact::WorthQueryApplicationFactKey::Aggregate(key),
                    super::super::fact::WorthQueryApplicationObservedFact::Aggregate {
                        observation,
                    },
                ) = (fact_key, fact)
                    && !self
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
                if let (
                    super::super::fact::WorthQueryApplicationFactKey::StructuralWalk(key),
                    super::super::fact::WorthQueryApplicationObservedFact::StructuralWalk {
                        observation,
                    },
                ) = (fact_key, fact)
                    && !self
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
                if let (super::InstalledScopePosition::SourceQuery,
                    super::super::fact::WorthQueryApplicationFactKey::SourceReadAuthorization(key),
                    super::super::fact::WorthQueryApplicationObservedFact::SourceReadAuthorization { observation },
                ) = (scope, fact_key, fact) {
                    return self.expected_facts.as_ref().and_then(|expected| expected.source_authorization(
                        key, self.admission.admission_identity(), &self.admission.retain_graph_contract(), self.lease.snapshot(),
                    )).is_some_and(|issued| std::sync::Arc::ptr_eq(issued, observation));
                }
                if let (
                    super::InstalledScopePosition::SourceQuery,
                    super::super::fact::WorthQueryApplicationFactKey::ManagedIndexSource(key),
                    super::super::fact::WorthQueryApplicationObservedFact::ManagedIndexSource {
                        observation,
                    },
                ) = (scope, fact_key, fact)
                {
                    return self
                        .expected_facts
                        .as_ref()
                        .and_then(|expected| {
                            expected.managed_source(
                                key,
                                self.admission.admission_identity(),
                                &self.admission.retain_graph_contract(),
                                self.lease.snapshot(),
                            )
                        })
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
            .validate_observations(self.facts)
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
        let (reads, mut completed) = self.take_computation_reads();
        let computation_fact_ordinals: std::sync::Arc<[usize]> = reads.ordinals(self.facts).into();
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
            std::mem::take(self.installed_read_scopes).into_values(),
            allocation_policy,
            operation,
            check_authority,
        )?;
        let facts = super::admit_completed_facts(
            fact_count,
            std::mem::take(self.facts).into_values().chain(
                std::mem::replace(self.source_facts, super::SourceFacts::Admitted(Vec::new()))
                    .into_values(),
            ),
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
        Ok(PreparedReadCompletion::Full(CompleteReadSetCore {
            effect_routes,
            read: CompletedReadFacts {
                postcondition_carry: None,
                application_projection: Some(application_projection),
                original_read: None,
                installed_read_scopes,
                facts,
                content,
                #[cfg(test)]
                computation_facts,
                computation_fact_ordinals,
            },
        }))
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
