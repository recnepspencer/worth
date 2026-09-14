use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::branch::{
    RelationalMaterializationError, RelationalRematerializationCompletion,
};
use worth_runtime_world::facade::RuntimeWorldPublicationOutcome;

use super::{WorthQueryCompletedGeneratedOutputReconstruction, WorthQuerySuspendedGeneratedOutput};
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOutputCorrespondence, WorthQueryApplicationProducerBinding,
    WorthQueryPrimaryGraphApplicationRuntime,
};

pub struct WorthQueryRestoredGeneratedOutput {
    branch: crate::basis::WorthQueryProductBranch,
    commit: worth_relational::facade::history::RelationalCommitReceipt,
}

impl WorthQueryRestoredGeneratedOutput {
    pub fn product_branch(&self) -> crate::basis::WorthQueryProductBranch {
        self.branch
    }

    pub fn commit(&self) -> &worth_relational::facade::history::RelationalCommitReceipt {
        &self.commit
    }
}

pub enum WorthQueryGeneratedOutputRestorationFailure {
    Rejected {
        suspended: WorthQuerySuspendedGeneratedOutput,
        cause: WorthQueryGeneratedOutputRestorationFailureCause,
    },
    ProductUnpublished(WorthQueryUnpublishedGeneratedOutputRestoration),
}

impl WorthQueryGeneratedOutputRestorationFailure {
    pub fn cause(&self) -> Option<&WorthQueryGeneratedOutputRestorationFailureCause> {
        match self {
            Self::Rejected { cause, .. } => Some(cause),
            Self::ProductUnpublished(_) => None,
        }
    }

    pub fn into_suspended(self) -> Result<WorthQuerySuspendedGeneratedOutput, Self> {
        match self {
            Self::Rejected { suspended, .. } => Ok(suspended),
            unpublished @ Self::ProductUnpublished(_) => Err(unpublished),
        }
    }
}

#[must_use = "unpublished restoration custody must remain with its World recovery authority"]
pub struct WorthQueryUnpublishedGeneratedOutputRestoration {
    product: crate::domain_computation::WorthQueryProductUnpublishedApplication,
    suspended: WorthQuerySuspendedGeneratedOutput,
}

impl WorthQueryUnpublishedGeneratedOutputRestoration {
    pub fn product(&self) -> &crate::domain_computation::WorthQueryProductUnpublishedApplication {
        &self.product
    }

    pub fn into_parts(
        self,
    ) -> (
        crate::domain_computation::WorthQueryProductUnpublishedApplication,
        WorthQuerySuspendedGeneratedOutput,
    ) {
        (self.product, self.suspended)
    }
}

#[derive(Debug)]
pub enum WorthQueryGeneratedOutputRestorationFailureCause {
    ForeignRuntime,
    WrongProducer,
    StaleProducerVersion,
    StaleOutputLineage,
    ProductActivationUnavailable,
    Preparation(RelationalMaterializationError),
    PublicationNoEffect(worth_runtime_world::facade::NoEffectCompositePublication),
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn restore_generated_output<Producer>(
        &self,
        completed: WorthQueryCompletedGeneratedOutputReconstruction<Schema, Producer>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<WorthQueryRestoredGeneratedOutput, WorthQueryGeneratedOutputRestorationFailure>
    where
        Producer: WorthQueryApplicationProducerBinding<Schema>,
    {
        let gate = match self
            .product_runtime
            .activations
            .gate(completed.suspended.publication.observation().branch_identity())
        {
            Ok(gate) => gate,
            Err(_) => {
                return Err(restoration_failure(
                    completed.suspended,
                    WorthQueryGeneratedOutputRestorationFailureCause::ProductActivationUnavailable,
                ));
            }
        };
        let _publication_admission = match gate.begin_publication() {
            Ok(admission) => admission,
            Err(_) => {
                return Err(restoration_failure(
                    completed.suspended,
                    WorthQueryGeneratedOutputRestorationFailureCause::ProductActivationUnavailable,
                ));
            }
        };
        let WorthQueryCompletedGeneratedOutputReconstruction {
            suspended,
            entities,
            relations,
            marker: _,
        } = completed;
        if !std::sync::Arc::ptr_eq(
            &suspended.publication.root_identity(),
            &self.product_runtime.root_identity(),
        ) || !suspended.matches_runtime(self)
        {
            return Err(restoration_failure(
                suspended,
                WorthQueryGeneratedOutputRestorationFailureCause::ForeignRuntime,
            ));
        }
        if !suspended.matches_producer::<Schema, Producer>() {
            return Err(restoration_failure(
                suspended,
                WorthQueryGeneratedOutputRestorationFailureCause::WrongProducer,
            ));
        }
        if self.installed_producers.provider::<Producer>().is_none() {
            return Err(restoration_failure(
                suspended,
                WorthQueryGeneratedOutputRestorationFailureCause::StaleProducerVersion,
            ));
        }
        if !suspended.matches_retained_lineage::<Schema, Producer>(self) {
            return Err(restoration_failure(
                suspended,
                WorthQueryGeneratedOutputRestorationFailureCause::StaleOutputLineage,
            ));
        }
        let WorthQuerySuspendedGeneratedOutput {
            publication,
            branch,
            custody,
            correspondence,
            producer,
        } = suspended;
        let prepared = self.primary_provider.graph.with_runtime(|runtime| {
            runtime
                .owner_component_services()
                .materialization_port()
                .prepare_generated_rematerialization(
                    publication.observation().basis().relational_basis(),
                    custody,
                    entities,
                    relations,
                )
        });
        let prepared = match prepared {
            Ok(prepared) => prepared,
            Err(failure) => {
                return Err(restoration_failure(
                    WorthQuerySuspendedGeneratedOutput {
                        publication,
                        branch,
                        custody: failure.custody,
                        correspondence,
                        producer,
                    },
                    WorthQueryGeneratedOutputRestorationFailureCause::Preparation(failure.error),
                ));
            }
        };
        let (candidate, completion) = prepared.into_parts();
        self.publish_restoration(
            publication,
            branch,
            correspondence,
            producer,
            candidate,
            completion,
            request,
        )
    }

    fn publish_restoration(
        &self,
        publication: WorthQueryProductPublicationBinding,
        branch: crate::basis::WorthQueryProductBranch,
        correspondence: std::sync::Arc<WorthQueryApplicationOutputCorrespondence>,
        producer: super::ProducerQualification,
        candidate: worth_relational::facade::mvcc::PreparedRelationalCommitCandidate,
        completion: RelationalRematerializationCompletion,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<WorthQueryRestoredGeneratedOutput, WorthQueryGeneratedOutputRestorationFailure>
    {
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
                        no_effect,
                    ),
                ));
            }
        };
        match prepared.execute() {
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
                let commit = completion
                    .complete(commit)
                    .expect("World returns the exact prepared relational restoration result");
                self.primary_provider
                    .graph
                    .output_lineage
                    .lock()
                    .expect("application output lineage lock is available")
                    .record_restoration(
                        producer.output_binding_type,
                        producer.runtime_authority,
                        producer.schema.clone(),
                        producer.scope,
                        &observation,
                        correspondence,
                        producer.source_identity,
                        producer.observed_source_facts,
                    );
                Ok(WorthQueryRestoredGeneratedOutput {
                    branch,
                    commit: commit.commit.clone(),
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
                WorthQueryGeneratedOutputRestorationFailureCause::PublicationNoEffect(no_effect),
            )),
            RuntimeWorldPublicationOutcome::ProductUnpublished(effects) => {
                let product = self.unpublished_materialization_from_binding(effects, &publication);
                Err(WorthQueryGeneratedOutputRestorationFailure::ProductUnpublished(
                    WorthQueryUnpublishedGeneratedOutputRestoration {
                        suspended: suspended_from_completion(
                        publication,
                        branch,
                        correspondence,
                        producer,
                        completion,
                        ),
                        product,
                    },
                ))
            }
        }
    }

    pub(super) fn materialization_publication_binding(
        &self,
        observation: worth_runtime_world::facade::ProductBranchObservation,
    ) -> WorthQueryProductPublicationBinding {
        WorthQueryProductPublicationBinding::new(
            observation,
            self.product_runtime.owner.publication_port(),
            self.product_runtime.owner.recovery_port(),
            self.product_runtime.clock.clone(),
            self.product_runtime.root_identity(),
        )
    }

    pub(super) fn unpublished_materialization(
        &self,
        effects: worth_runtime_world::facade::ProductUnpublishedOwnerEffects,
        product: &crate::basis::WorthQueryProductBranchLease,
    ) -> crate::domain_computation::WorthQueryProductUnpublishedApplication {
        self.unpublished_materialization_from_binding(effects, &product.publication_binding())
    }

    fn unpublished_materialization_from_binding(
        &self,
        effects: worth_runtime_world::facade::ProductUnpublishedOwnerEffects,
        publication: &WorthQueryProductPublicationBinding,
    ) -> crate::domain_computation::WorthQueryProductUnpublishedApplication {
        crate::domain_computation::WorthQueryProductUnpublishedApplication::new(
            effects,
            publication.recovery(),
            self.primary_provider.unpublished_idempotency_disposition(),
        )
    }
}

fn suspended_from_completion(
    publication: WorthQueryProductPublicationBinding,
    branch: crate::basis::WorthQueryProductBranch,
    correspondence: std::sync::Arc<WorthQueryApplicationOutputCorrespondence>,
    producer: super::ProducerQualification,
    completion: RelationalRematerializationCompletion,
) -> WorthQuerySuspendedGeneratedOutput {
    WorthQuerySuspendedGeneratedOutput {
        publication,
        branch,
        custody: completion.into_custody(),
        correspondence,
        producer,
    }
}

fn restoration_failure(
    suspended: WorthQuerySuspendedGeneratedOutput,
    cause: WorthQueryGeneratedOutputRestorationFailureCause,
) -> WorthQueryGeneratedOutputRestorationFailure {
    WorthQueryGeneratedOutputRestorationFailure::Rejected { suspended, cause }
}
