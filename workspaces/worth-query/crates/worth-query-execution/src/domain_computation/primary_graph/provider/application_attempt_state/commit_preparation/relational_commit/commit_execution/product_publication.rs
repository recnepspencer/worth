use worth_runtime_world::facade::{
    RuntimeWorldConditionalDefinitionPublicationOutcome, RuntimeWorldPublicationOutcome,
};

use super::{failure, world_no_effect};
use crate::domain_computation::primary_graph::provider::{
    WorthQueryPrimaryGraphApplicationAttempt, WorthQueryPrimaryGraphProvider,
};

pub(super) struct WorthQueryPerformedApplicationProductPublication {
    pub publication: crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationReceipt,
}

pub(super) fn publish(
    provider: &WorthQueryPrimaryGraphProvider,
    attempt: &mut WorthQueryPrimaryGraphApplicationAttempt,
    candidate: worth_relational::facade::mvcc::PreparedRelationalCommitCandidate,
) -> Result<
    WorthQueryPerformedApplicationProductPublication,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    let product = attempt.affinity().product_publication().clone();
    let request = attempt.affinity().publication_request().clone();
    let idempotency = attempt.idempotency();
    let recovery = product.recovery();
    let disposition = provider.unpublished_idempotency_disposition();
    let Some(change) = attempt.take_conditional_definition() else {
        let prepared = product
            .prepare_relational_candidate(candidate, &request)
            .map_err(world_no_effect)?;
        let recovery_handle = prepared.unpublished_recovery_handle();
        let terminal = crate::domain_computation::execution_runtime::product_world::WorthQueryReservedProductPublicationReceipt::new(
            product.root_identity(),
            recovery_handle.clone(),
        );
        let reservation = provider
            .reserve_unpublished_application_idempotency(
                &product,
                idempotency,
                recovery_handle,
                recovery.clone(),
            )
            .map_err(|()| capacity_exhausted())?;
        return match prepared.execute() {
            RuntimeWorldPublicationOutcome::Performed(publication) => {
                reservation.release();
                Ok(WorthQueryPerformedApplicationProductPublication {
                    publication: terminal.fill(publication.consume(), None),
                })
            }
            RuntimeWorldPublicationOutcome::ProductUnpublished(effects) => {
                reservation.retain(&effects);
                Err(product_unpublished(
                    crate::domain_computation::WorthQueryProductUnpublishedApplication::new(
                        effects,
                        recovery,
                        disposition,
                    ),
                ))
            }
            RuntimeWorldPublicationOutcome::NoEffect(no_effect) => {
                reservation.release();
                Err(world_no_effect(no_effect))
            }
        };
    };

    let parts = change.into_parts();
    if parts.expected_product != *product.observation() {
        return Err(denied(
            "conditional definition belongs to another product occurrence",
        ));
    }
    let gate = parts
        .activations
        .gate(product.observation().branch_identity())
        .map_err(|_| denied("product activation rejected combined publication"))?;
    gate.publish(|| {
        let bridge = parts
            .bridge
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let prepared = product
            .prepare_combined_candidate(candidate, &request)
            .map_err(world_no_effect)?;
        let predecessor = bridge
            .admit_exact_conditional_signal_basis(
                &parts.predecessor,
                product.observation().basis().signal_basis(),
            )
            .map_err(|_| denied("Bridge rejected the exact combined Signal basis"))?;
        let bridge_prepared = bridge
            .prepare_owned_conditional_definition_successor(&predecessor, parts.request)
            .map_err(|_| denied("Bridge rejected combined conditional preparation"))?;
        let recovery_handle = prepared.unpublished_recovery_handle();
        let terminal = crate::domain_computation::execution_runtime::product_world::WorthQueryReservedProductPublicationReceipt::new(
            product.root_identity(),
            recovery_handle.clone(),
        );
        let reservation = provider
            .reserve_unpublished_application_idempotency(
                &product,
                idempotency,
                recovery_handle,
                recovery.clone(),
            )
            .map_err(|()| capacity_exhausted())?;
        match prepared.execute(bridge_prepared, &bridge) {
            RuntimeWorldConditionalDefinitionPublicationOutcome::Performed {
                publication,
                lowering,
            } => {
                reservation.release();
                Ok(WorthQueryPerformedApplicationProductPublication {
                    publication: terminal.fill(
                        publication.consume(),
                        Some(lowering.signal_definition_generation()),
                    ),
                })
            }
            RuntimeWorldConditionalDefinitionPublicationOutcome::ProductUnpublished(effects) => {
                reservation.retain(effects.effects());
                Err(product_unpublished(
                    crate::domain_computation::WorthQueryProductUnpublishedApplication::new_conditional_definition(
                        effects,
                        recovery,
                        disposition,
                    ),
                ))
            }
            RuntimeWorldConditionalDefinitionPublicationOutcome::NoEffect(no_effect) => {
                reservation.release();
                Err(world_no_effect(no_effect))
            }
        }
    })
    .map_err(|_| denied("product activation rejected combined publication"))?
}

fn capacity_exhausted() -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    denied("unpublished idempotency retention capacity is exhausted")
}

fn denied(detail: &'static str) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    crate::domain_computation::WorthQueryProviderSessionCommitStop::Denied(failure(detail))
}

fn product_unpublished(
    unpublished: crate::domain_computation::WorthQueryProductUnpublishedApplication,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    crate::domain_computation::WorthQueryProviderSessionCommitStop::ProductUnpublished(unpublished)
}
