use super::*;

pub(super) fn resolve_retained_idempotency<Schema, Operation, Input, Scope>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    idempotency: WorthQueryApplicationIdempotencyBinding,
    aftermath_causality: Option<&WorthQueryPendingAftermathCausality>,
) -> Option<WorthQueryApplicationCommitOutcome>
where
    Schema: ApplicationSchema,
    Input: Clone + Send + Sync + 'static,
{
    let serialization = application.primary_provider.serialize_application_commit();
    let Some(product) = admission.graph_work().mutation_product() else {
        return Some(denied(DenialStage::DecisionReadSet));
    };
    let product = product.publication_binding();
    let proof = match application.authorize_retained_idempotency(admission, &serialization) {
        Ok(proof) => proof,
        Err(denial) => {
            return Some(commit_outcome_from_authorization_denial(
                denial,
                DenialStage::DecisionReadSet,
            ))
        }
    };
    match proof.govern((), |()| {
        application
            .primary_provider
            .resolve_idempotency_binding_at_product(idempotency, &product)
    }) {
        Err(((), denial)) => Some(commit_outcome_from_authorization_denial(
            denial,
            DenialStage::DecisionReadSet,
        )),
        Ok(Ok(WorthQueryProviderIdempotencyResolution::Absent)) => None,
        Ok(Ok(WorthQueryProviderIdempotencyResolution::Equivalent(receipt))) => {
            let causality = match resolve_exact_committed_aftermath(
                &application.primary_provider,
                aftermath_causality,
                &receipt,
            ) {
                Ok(causality) => causality,
                Err(crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::ActiveSnapshotCapacityExhausted {
                    maximum_active_snapshots,
                }) => {
                    return Some(WorthQueryApplicationCommitOutcome::Denied(
                        WorthQueryApplicationCommitDenial::active_snapshot_capacity_exhausted(
                            DenialStage::Idempotency,
                            maximum_active_snapshots,
                        ),
                    ))
                }
                Err(crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::Unavailable) => {
                    return Some(WorthQueryApplicationCommitOutcome::Denied(
                        WorthQueryApplicationCommitDenial::idempotency_intent_drift(),
                    ))
                }
                Err(crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::RetentionCapacityExhausted) => {
                    return Some(WorthQueryApplicationCommitOutcome::Denied(
                        WorthQueryApplicationCommitDenial::retention_capacity_exhausted(
                            DenialStage::Idempotency,
                        ),
                    ))
                }
                Err(crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::RetentionIdentityExhausted) => {
                    return Some(WorthQueryApplicationCommitOutcome::Denied(
                        WorthQueryApplicationCommitDenial::retention_identity_exhausted(
                            DenialStage::Idempotency,
                        ),
                    ))
                }
                Err(crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::SnapshotIdentityExhausted) => {
                    return Some(WorthQueryApplicationCommitOutcome::Denied(
                        WorthQueryApplicationCommitDenial::snapshot_identity_exhausted(
                            DenialStage::Idempotency,
                        ),
                    ))
                }
            };
            let projection = match WorthQueryCommittedReceiptProjection::resolve(receipt) {
                Ok(projection) => projection,
                Err(_) => return Some(denied(DenialStage::Idempotency)),
            };
            let receipt = WorthQueryApplicationCommitReceipt::from_early_equivalent(
                WorthQueryEarlyEquivalentCommitReceiptPermit::mint(),
                projection,
                recover_equivalent_commit_evidence(admission.mutation_preconditions()),
                admission.canonical_work(),
                WorthQueryApplicationCommitAuthorityBinding::from_admission(admission, idempotency),
            );
            Some(WorthQueryApplicationCommitOutcome::AlreadyCommitted(
                receipt.with_aftermath_causality(causality),
            ))
        }
        Ok(Ok(WorthQueryProviderIdempotencyResolution::Drift)) => {
            Some(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::idempotency_intent_drift(),
            ))
        }
        Ok(Ok(WorthQueryProviderIdempotencyResolution::Unpublished)) => {
            Some(denied(DenialStage::Idempotency))
        }
        Ok(Err(crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        })) => Some(WorthQueryApplicationCommitOutcome::Denied(
            WorthQueryApplicationCommitDenial::active_snapshot_capacity_exhausted(
                DenialStage::Idempotency,
                maximum_active_snapshots,
            ),
        )),
        Ok(Err(crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial::Unavailable)) => {
            Some(denied(DenialStage::Idempotency))
        }
        Ok(Err(crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial::RetentionCapacityExhausted)) => {
            Some(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::retention_capacity_exhausted(
                    DenialStage::Idempotency,
                ),
            ))
        }
        Ok(Err(crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial::RetentionIdentityExhausted)) => {
            Some(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::retention_identity_exhausted(
                    DenialStage::Idempotency,
                ),
            ))
        }
        Ok(Err(crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial::SnapshotIdentityExhausted)) => {
            Some(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::snapshot_identity_exhausted(
                    DenialStage::Idempotency,
                ),
            ))
        }
    }
}
