use super::*;
use crate::domain_computation::execution_runtime::product_world::publication_denial::{
    WorthQueryProductPublicationDenial as PublicationDenial,
    WorthQueryProductPublicationOutcome as PublicationOutcome,
};
use crate::domain_computation::primary_graph::application_contribution::advancement_denial;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductPublicationBindingInternalPort as _;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    #[allow(clippy::too_many_arguments)]
    pub(super) fn publish_restoration(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        publication: WorthQueryProductPublicationBinding,
        branch: crate::basis::WorthQueryProductBranch,
        correspondence: std::sync::Arc<WorthQueryApplicationOutputCorrespondence>,
        producer: super::super::ProducerQualification,
        proposal: worth_relational::facade::mvcc::ValidatedRelationalProposal,
        pending_completion: worth_relational::facade::branch::PendingRelationalRematerializationCompletion,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<WorthQueryRestoredGeneratedOutput, WorthQueryGeneratedOutputRestorationFailure>
    {
        use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
        use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
        let publication_execution =
            match phase.execution_request_for(self.inner.runtime.product_runtime_ref()) {
                Ok(execution) => execution,
                Err(cause) => {
                    return Err(restoration_failure(
                        WorthQuerySuspendedGeneratedOutput {
                            publication,
                            branch,
                            custody: pending_completion.into_custody(),
                            correspondence,
                            producer,
                        },
                        WorthQueryGeneratedOutputRestorationFailureCause::advancement(cause.into()),
                    ))
                }
            };
        let open = match super::super::durable_publication::open_if_home(
            &publication,
            request,
            execution,
        ) {
            Ok(open) => open,
            Err(super::super::durable_publication::DurableOpenDenial::ClockMismatch) => {
                return Err(restoration_failure(
                    WorthQuerySuspendedGeneratedOutput {
                        publication,
                        branch,
                        custody: pending_completion.into_custody(),
                        correspondence,
                        producer,
                    },
                    WorthQueryGeneratedOutputRestorationFailureCause::advancement(
                        crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::ForeignAdmissionClock),
                    ),
                ))
            }
            Err(super::super::durable_publication::DurableOpenDenial::Stopped(stop)) => {
                return Err(restoration_failure(
                    WorthQuerySuspendedGeneratedOutput {
                        publication,
                        branch,
                        custody: pending_completion.into_custody(),
                        correspondence,
                        producer,
                    },
                    WorthQueryGeneratedOutputRestorationFailureCause::advancement(
                        advancement_denial(stop),
                    ),
                ))
            }
            Err(super::super::durable_publication::DurableOpenDenial::Refused(cause)) => {
                return Err(restoration_failure(
                    WorthQuerySuspendedGeneratedOutput {
                        publication,
                        branch,
                        custody: pending_completion.into_custody(),
                        correspondence,
                        producer,
                    },
                    WorthQueryGeneratedOutputRestorationFailureCause::PublicationNoEffect(
                        WorthQueryGeneratedOutputPublicationNoEffect::from_cause(cause),
                    ),
                ));
            }
        };
        let (outcome, completion) = match open {
            super::super::durable_publication::DurableMaterializationOpen::Memory => {
                let candidate = self
                    .inner
                    .runtime
                    .primary_provider_ref()
                    .graph_ref()
                    .with_runtime(execution, |runtime| {
                        runtime.prepare_validated_proposal(execution, proposal)
                    });
                let candidate = match candidate {
                    Ok(candidate) => candidate,
                    Err(error) => {
                        return Err(restoration_failure(
                            WorthQuerySuspendedGeneratedOutput {
                                publication,
                                branch,
                                custody: pending_completion.into_custody(),
                                correspondence,
                                producer,
                            },
                            super::preparation_failure_cause(
                                &worth_relational::facade::branch::RelationalMaterializationError::Commit(error),
                            ),
                        ));
                    }
                };
                let completion = pending_completion.finish(&candidate);
                let prepared = match publication.prepare_relational_candidate(candidate, request, true) {
                    Ok(prepared) => prepared,
                    Err(PublicationDenial::ClockMismatch) => {
                        return Err(restoration_failure(
                            suspended_from_completion(
                                publication,
                                branch,
                                correspondence,
                                producer,
                                completion,
                            ),
                            WorthQueryGeneratedOutputRestorationFailureCause::advancement(
                                crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::ForeignAdmissionClock),
                            ),
                        ))
                    }
                    Err(PublicationDenial::Stopped(stop)) => {
                        return Err(restoration_failure(
                            suspended_from_completion(
                                publication,
                                branch,
                                correspondence,
                                producer,
                                completion,
                            ),
                            WorthQueryGeneratedOutputRestorationFailureCause::advancement(
                                advancement_denial(stop),
                            ),
                        ))
                    }
                    Err(PublicationDenial::Refused(no_effect)) => {
                        return Err(restoration_failure(
                            suspended_from_completion(
                                publication,
                                branch,
                                correspondence,
                                producer,
                                completion,
                            ),
                            WorthQueryGeneratedOutputRestorationFailureCause::PublicationNoEffect(
                                WorthQueryGeneratedOutputPublicationNoEffect::from_world(no_effect),
                            ),
                        ));
                    }
                };
                (prepared.execute(publication_execution), Some(completion))
            }
            super::super::durable_publication::DurableMaterializationOpen::Retained(effects) => {
                (Ok(PublicationOutcome::ProductUnpublished(effects)), None)
            }
            super::super::durable_publication::DurableMaterializationOpen::Opened {
                opened,
                control,
            } => {
                let mut pending = Some(pending_completion);
                match super::super::durable_publication::prepare_and_execute_opened(
                    self,
                    &publication,
                    opened,
                    &control,
                    request,
                    publication_execution,
                    proposal,
                    |candidate| {
                        pending
                            .take()
                            .expect("pending completion consumed once")
                            .finish(candidate)
                    },
                ) {
                    Ok((outcome, completion)) => (Ok(outcome), Some(completion)),
                    Err((PublicationOutcome::ProductUnpublished(effects), completion)) => {
                        (Ok(PublicationOutcome::ProductUnpublished(effects)), completion)
                    }
                    Err(_) => {
                        return Err(restoration_failure(
                            WorthQuerySuspendedGeneratedOutput {
                                publication,
                                branch,
                                custody: pending
                                    .take()
                                    .expect("custody retained on preparation refusal")
                                    .into_custody(),
                                correspondence,
                                producer,
                            },
                            WorthQueryGeneratedOutputRestorationFailureCause::PublicationNoEffect(
                                WorthQueryGeneratedOutputPublicationNoEffect::from_cause(
                                    WorthQueryGeneratedOutputPublicationNoEffectCause::PreEffectFailure,
                                ),
                            ),
                        ));
                    }
                }
            }
        };
        match outcome {
            Ok(PublicationOutcome::Performed(performed)) => {
                let commit = performed
                    .component_results()
                    .relational_commit_result()
                    .cloned()
                    .expect("a relational-only publication returns its relational result");
                let mut consumed = performed.consume();
                let observation = consumed
                    .take_successor_observation()
                    .expect("the requested restoration successor is retained");
                let restored = completion
                    .expect("performed restoration carries prepared Relational completion")
                    .complete(commit)
                    .expect("World returns the exact prepared relational restoration result");
                self.record_restored_generated_output(
                    execution,
                    &observation,
                    correspondence,
                    producer,
                );
                Ok(WorthQueryRestoredGeneratedOutput {
                    branch,
                    commit: WorthQueryGeneratedOutputRestorationReceipt::new(
                        restored.commit.clone(),
                    ),
                })
            }
            Err(PublicationDenial::ClockMismatch) => Err(restoration_failure(
                suspended_from_completion(
                    publication,
                    branch,
                    correspondence,
                    producer,
                    completion.expect("memory restoration carries prepared completion"),
                ),
                WorthQueryGeneratedOutputRestorationFailureCause::advancement(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::ForeignAdmissionClock)),
            )),
Err(PublicationDenial::Stopped(stop)) => Err(restoration_failure(
                suspended_from_completion(
                    publication,
                    branch,
                    correspondence,
                    producer,
                    completion.expect("memory restoration carries prepared completion"),
                ),
                WorthQueryGeneratedOutputRestorationFailureCause::advancement(advancement_denial(
                    stop,
                )),
            )),
            Err(PublicationDenial::Refused(no_effect)) => Err(restoration_failure(
                suspended_from_completion(
                    publication,
                    branch,
                    correspondence,
                    producer,
                    completion.expect("memory restoration carries prepared completion"),
                ),
                WorthQueryGeneratedOutputRestorationFailureCause::PublicationNoEffect(
                    WorthQueryGeneratedOutputPublicationNoEffect::from_world(no_effect),
                ),
            )),
            Ok(PublicationOutcome::ProductUnpublished(effects)) => {
                let product = self.unpublished_materialization_from_binding(effects, &publication);
                Err(
                    WorthQueryGeneratedOutputRestorationFailure::ProductUnpublished(
                        WorthQueryUnpublishedGeneratedOutputRestoration {
                            product,
                            retry: recovery::RestorationRetry {
                                publication,
                                branch,
                                correspondence,
                                producer,
                                completion,
                            },
                        },
                    ),
                )
            }
        }
    }
}
