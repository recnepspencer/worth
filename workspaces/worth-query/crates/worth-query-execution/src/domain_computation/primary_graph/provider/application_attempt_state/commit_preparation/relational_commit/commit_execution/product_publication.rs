use worth_runtime_world::facade::{
    RuntimeWorldConditionalDefinitionPublicationOutcome, RuntimeWorldPublicationOutcome,
};

use super::{failure, world_no_effect};
use crate::domain_computation::primary_graph::provider::{
    WorthQueryPrimaryGraphApplicationAttempt, WorthQueryPrimaryGraphProvider,
};

pub(super) struct WorthQueryPerformedApplicationProductPublication {
    pub publication: crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationReceipt,
    pub prepared_lineage_slot: Option<crate::domain_computation::primary_graph::output_lineage::PreparedOutputLineageSlot>,
}

pub(super) enum WorthQueryApplicationProductPublicationOutcome {
    Performed(WorthQueryPerformedApplicationProductPublication),
    Unpublished {
        unpublished: crate::domain_computation::WorthQueryProductUnpublishedApplication,
        reserved_terminal: crate::domain_computation::execution_runtime::product_world::WorthQueryReservedProductPublicationReceipt,
        prepared_lineage_slot: Option<crate::domain_computation::primary_graph::output_lineage::PreparedOutputLineageSlot>,
        reservation: crate::domain_computation::primary_graph::provider::WorthQueryUnpublishedIdempotencyReservation,
    },
}

pub(super) fn publish(
    provider: &WorthQueryPrimaryGraphProvider,
    attempt: &mut WorthQueryPrimaryGraphApplicationAttempt,
    candidate: worth_relational::facade::mvcc::PreparedRelationalCommitCandidate,
    required_prerequisites: &mut Option<
        crate::domain_computation::primary_graph::PreparedPrerequisiteClaims,
    >,
    admission: &mut Option<crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission>,
) -> Result<
    WorthQueryApplicationProductPublicationOutcome,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    let product = attempt.affinity().product_publication().clone();
    let request = attempt.affinity().publication_request().clone();
    let idempotency = attempt.idempotency();
    let recovery = product.recovery();
    let disposition = provider.unpublished_idempotency_disposition();
    let (retain_live_observation, retain_output_demand_observation, retain_client_observation) =
        attempt
            .reserve_successor_observations(provider)
            .map_err(|_| capacity_exhausted())?;
    let successor_observation_requested =
        retain_live_observation || retain_output_demand_observation || retain_client_observation;
    let Some(change) = attempt.take_conditional_definition() else {
        let prepared = product
            .prepare_relational_candidate(candidate, &request, successor_observation_requested)
            .map_err(world_no_effect)?;
        let lineage_slot = prepare_lineage_slot(
            provider,
            attempt,
            prepared.planned_successor(),
            required_prerequisites,
            admission
                .as_mut()
                .expect("publication admission remains live before effects"),
        )?;
        let recovery_handle = prepared.unpublished_recovery_handle();
        let terminal = crate::domain_computation::execution_runtime::product_world::WorthQueryReservedProductPublicationReceipt::new(
            product.root_identity(),
            recovery_handle.clone(),
            retain_live_observation,
            retain_output_demand_observation,
            retain_client_observation,
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
                Ok(WorthQueryApplicationProductPublicationOutcome::Performed(
                    WorthQueryPerformedApplicationProductPublication {
                        publication: terminal.fill(publication.consume(), None),
                        prepared_lineage_slot: lineage_slot,
                    },
                ))
            }
            RuntimeWorldPublicationOutcome::ProductUnpublished(effects) => Ok(
                WorthQueryApplicationProductPublicationOutcome::Unpublished {
                    unpublished:
                        crate::domain_computation::WorthQueryProductUnpublishedApplication::new(
                            effects,
                            recovery,
                            disposition,
                        ),
                    reserved_terminal: terminal,
                    prepared_lineage_slot: lineage_slot,
                    reservation,
                },
            ),
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
            .prepare_combined_candidate(candidate, &request, successor_observation_requested)
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
        let lineage_slot = prepare_lineage_slot(provider, attempt, prepared.planned_successor(), required_prerequisites, admission.as_mut().expect("publication admission remains live before effects"))?;
        let recovery_handle = prepared.unpublished_recovery_handle();
        let terminal = crate::domain_computation::execution_runtime::product_world::WorthQueryReservedProductPublicationReceipt::new(
            product.root_identity(),
            recovery_handle.clone(),
            retain_live_observation,
            retain_output_demand_observation,
            retain_client_observation,
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
                Ok(WorthQueryApplicationProductPublicationOutcome::Performed(WorthQueryPerformedApplicationProductPublication {
                    publication: terminal.fill(
                        publication.consume(),
                        Some(lowering.signal_definition_generation()),
                    ),
                    prepared_lineage_slot: lineage_slot,
                }))
            }
            RuntimeWorldConditionalDefinitionPublicationOutcome::ProductUnpublished(effects) => {
                Ok(WorthQueryApplicationProductPublicationOutcome::Unpublished {
                    unpublished: crate::domain_computation::WorthQueryProductUnpublishedApplication::new_conditional_definition(
                        effects,
                        recovery,
                        disposition,
                    ),
                    reserved_terminal: terminal,
                    prepared_lineage_slot: lineage_slot,
                    reservation,
                })
            }
            RuntimeWorldConditionalDefinitionPublicationOutcome::NoEffect(no_effect) => {
                reservation.release();
                Err(world_no_effect(no_effect))
            }
        }
    })
    .map_err(|_| denied("product activation rejected combined publication"))?
}

fn prepare_lineage_slot(
    provider: &WorthQueryPrimaryGraphProvider,
    attempt: &WorthQueryPrimaryGraphApplicationAttempt,
    planned: &worth_runtime_world::facade::PlannedProductReferenceSuccessor,
    required: &mut Option<crate::domain_computation::primary_graph::PreparedPrerequisiteClaims>,
    admission: &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
) -> Result<
    Option<crate::domain_computation::primary_graph::output_lineage::PreparedOutputLineageSlot>,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    let Some(binding) = attempt.output_binding_type() else {
        if required.is_some() {
            return Err(denied("managed producer omitted its sealed output binding"));
        }
        return Ok(None);
    };
    let mut slot =
        crate::domain_computation::primary_graph::output_lineage::prepare_output_lineage_slot(
            &provider.graph.output_lineage,
            attempt.affinity().operation_scope(),
            binding,
            attempt.idempotency().source_partition_identity(),
            planned,
            admission,
        )
        .map_err(lineage_pending)?;
    if let Some(required) = required {
        // Move already declared resources through the prepared owner record.
        // This also pays its final fixed-width copy before World publication.
        admission
            .charge_external_work((4 * std::mem::size_of::<Option<crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerDemandResources>>() + 4) as u64)
            .map_err(|_| crate::domain_computation::WorthQueryProviderSessionCommitStop::Deferred(
                crate::domain_computation::WorthQueryProviderSessionCommitDeferred::required_prerequisite(
                    crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial::new(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""),
                    "",
                ),
            ))?;
        slot.retain_actual_resources(required.take_actual_resources());
        required
            .reserve_identity(slot.identity(), admission)
            .map_err(lineage_pending)?;
        slot.retain_completed_handler_facts(
            required
                .take_completed_handler_facts()
                .expect("managed handler completed its sealed read before World publication"),
        );
        if let Some(proof) = required.take_completed_decision_reuse() {
            slot.retain_completed_decision_reuse(proof);
        }
        if let Some(key) = required.take_prepared_input_reuse_key() {
            slot.retain_prepared_input_reuse_key(key);
        }
    }
    Ok(Some(slot))
}

fn lineage_pending(
    denial: crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    crate::domain_computation::WorthQueryProviderSessionCommitStop::Deferred(
        crate::domain_computation::WorthQueryProviderSessionCommitDeferred::required_prerequisite(
            denial,
            "exact output lineage and required settlement could not reserve before World publication",
        ),
    )
}

fn capacity_exhausted() -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    denied("unpublished idempotency retention capacity is exhausted")
}

fn denied(detail: &'static str) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    crate::domain_computation::WorthQueryProviderSessionCommitStop::Denied(failure(detail))
}
