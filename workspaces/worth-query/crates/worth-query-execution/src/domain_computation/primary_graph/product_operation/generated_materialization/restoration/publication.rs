use super::*;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(super) fn publish_restoration(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        publication: WorthQueryProductPublicationBinding,
        branch: crate::basis::WorthQueryProductBranch,
        correspondence: std::sync::Arc<WorthQueryApplicationOutputCorrespondence>,
        producer: super::super::ProducerQualification,
        candidate: worth_relational::facade::mvcc::PreparedRelationalCommitCandidate,
        completion: RelationalRematerializationCompletion,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<WorthQueryRestoredGeneratedOutput, WorthQueryGeneratedOutputRestorationFailure>
    {
        let publication_execution = match phase.execution_request_for(&self.product_runtime) {
            Ok(execution) => execution,
            Err(cause) => {
                return Err(restoration_failure(
                    suspended_from_completion(
                        publication,
                        branch,
                        correspondence,
                        producer,
                        completion,
                    ),
                    WorthQueryGeneratedOutputRestorationFailureCause::advancement(cause.into()),
                ))
            }
        };
        let prepared = match publication.prepare_relational_candidate(candidate, request, true) {
            Ok(prepared) => prepared,
            Err(no_effect) => {
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
        match prepared.execute(publication_execution) {
            RuntimeWorldPublicationOutcome::Performed(performed) => {
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
            RuntimeWorldPublicationOutcome::NoEffect(no_effect) => Err(restoration_failure(
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
            )),
            RuntimeWorldPublicationOutcome::ProductUnpublished(effects) => {
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
