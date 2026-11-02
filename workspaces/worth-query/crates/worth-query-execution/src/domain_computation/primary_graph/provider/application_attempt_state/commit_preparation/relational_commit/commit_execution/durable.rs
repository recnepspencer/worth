//! The same original World open brackets Query's one Native preparation.
use super::durable_custody::{
    QueryDurablePreparationDenial as Cause, QueryDurableWorldCustody as World,
    QueryPreparingApplicationCommit, RetainedQueryDurablePreparation,
};
use super::*;
use crate::domain_computation::primary_graph::provider::QueryDurablePreparationGuard;
use crate::domain_computation::{
    WorthQueryProductPreparationPending as Pending, WorthQueryProductPreparationRefusal as Refusal,
    WorthQueryProductPreparationRefused as Refused, WorthQueryProductPreparationStage as Stage,
    WorthQueryProviderSessionCommitStop as Stop,
};
use worth_runtime_world::facade::WorldOpenAttemptRefusal;

pub(super) fn refused_before_open(
    query: QueryPreparingApplicationCommit,
    cause: WorldOpenAttemptRefusal,
) -> Stop {
    Stop::ProductPreparationRefused(Refused::new(Refusal::World(cause), query))
}
fn pending(
    mut guard: QueryDurablePreparationGuard,
    provider: &WorthQueryPrimaryGraphProvider,
    stage: Stage,
) -> Stop {
    let retained = guard.preparation_mut();
    retained.stage = stage;
    let pending = Pending::new(
        stage,
        retained.recovery_handle.clone(),
        retained
            .query
            .attempt
            .affinity()
            .product_publication()
            .recovery(),
        provider.unpublished_idempotency_disposition(),
    );
    Stop::ProductPreparationPending(pending.retaining_preparation(guard))
}

pub(super) fn commit(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    query: QueryPreparingApplicationCommit,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<WorthQueryCommittedApplicationSession, Stop> {
    if query.attempt.has_conditional_definition() {
        return Err(Stop::ProductPreparationRefused(Refused::new(
            Refusal::ConditionalPublicationRequired,
            query,
        )));
    }
    let product = query.attempt.affinity().product_publication().clone();
    let control = product.request_control(query.attempt.affinity().publication_request());
    let reservation =
        match provider.reserve_query_before_world(&product, query.attempt.idempotency()) {
            Ok(reservation) => reservation,
            Err(()) => {
                return Err(Stop::ProductPreparationRefused(Refused::new(
                    Refusal::QueryPreparationReservationRefused,
                    query,
                )))
            }
        };
    let mut before = before_world::BeforeWorldPreparation::new(query, reservation);
    // Admission of the actual subject precedes these causal reservations.
    let (live, demand, client) = match before.reserve_successor_observations(&execution, provider) {
        Ok(flags) => flags,
        Err(cause) => {
            return Err(Stop::ProductPreparationRefused(Refused::new(
                match cause {
                    crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveAdmissionDenial::Execution(cause) => Refusal::QuerySuccessorObservationRequest(cause),
                    crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveAdmissionDenial::Unavailable(detail) => Refusal::QuerySuccessorObservation(detail),
                },
                before.refuse(),
            )))
        }
    };
    let open = match prepare_query_open(&execution, &product, &control) {
        Ok(open) => open,
        Err(cause) => return Err(refused_before_open(before.refuse(), cause)),
    };
    let mut guard = before.bind(open, &product, control, (live, demand, client));
    let outcome = progression::run(execution, provider, &mut guard, allocation_policy);
    // Actual terminal custody wins over a late framework stop. No performed
    // result or deleted-record refusal is reconstructed from a progress marker.
    if matches!(guard.preparation_mut().world, World::OpenRefused(_)) {
        let (mut retained, reservation) = guard.finish();
        let World::OpenRefused(cause) = std::mem::replace(&mut retained.world, World::RecoveryOnly)
        else {
            unreachable!()
        };
        reservation.release();
        return Err(refused_before_open(retained.query, cause));
    }
    if matches!(guard.preparation_mut().world, World::Performed(_)) {
        let (mut retained, reservation) = guard.finish();
        let World::Performed(performed) =
            std::mem::replace(&mut retained.world, World::RecoveryOnly)
        else {
            unreachable!()
        };
        let committed = into_managed(retained).complete_live(performed, None);
        reservation.release();
        return Ok(committed);
    }
    if matches!(guard.preparation_mut().world, World::Unpublished(_)) {
        let (mut retained, reservation) = guard.finish();
        let World::Unpublished(effects) =
            std::mem::replace(&mut retained.world, World::RecoveryOnly)
        else {
            unreachable!()
        };
        let handle = effects.recovery_handle();
        reservation.retain_managed(handle, into_managed(retained));
        return Err(Stop::ProductUnpublished(
            crate::domain_computation::WorthQueryProductUnpublishedApplication::new(
                effects,
                product.recovery(),
                provider.unpublished_idempotency_disposition(),
            ),
        ));
    }
    match outcome {
        Ok(progression::Progress::Pending(stage)) => Err(pending(guard, provider, stage)),
        Ok(progression::Progress::Terminal) => {
            unreachable!("terminal retains its actual raw owner")
        }
        Err(cause) => {
            let retained = guard.preparation_mut();
            let stage = retained.stage();
            retained.query_denial.get_or_insert(Cause::Request(cause));
            Err(pending(guard, provider, stage))
        }
    }
}
mod progression;

fn into_managed(retained: RetainedQueryDurablePreparation) -> ManagedUnpublishedAttempt {
    let query = retained.query;
    ManagedUnpublishedAttempt {
        _completion: query.completion,
        publication_mode: managed_unpublished::ManagedUnpublishedPublicationMode::Ordinary,
        attempt: query.attempt,
        predicate: query.predicate,
        work: query.work,
        index_maintenance_work: query
            .index_maintenance_work
            .expect("same candidate indexes prepared"),
        retained_preimage: query.retained_preimage,
        preimage_retention_work: query.preimage_retention_work,
        before: query.before,
        managed_views: query.managed_views,
        source_fact_rebase: query.source_fact_rebase,
        required_prerequisites: query.required_prerequisites,
        prepared_lineage_slot: retained.prepared_lineage_slot,
        prepared_output_witness: query.prepared_output_witness,
        prepared_touched_records: query
            .prepared_touched_records
            .expect("same candidate touched records prepared"),
        lineage_metadata: None,
        reserved_terminal: retained.terminal,
    }
}
