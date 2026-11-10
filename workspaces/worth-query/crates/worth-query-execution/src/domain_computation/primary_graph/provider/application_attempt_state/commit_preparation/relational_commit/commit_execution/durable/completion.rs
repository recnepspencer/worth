//! Consume the actual terminal custody only after owner progression has returned.
use super::*;
#[inline(never)]
pub(super) fn finish(
    provider: &WorthQueryPrimaryGraphProvider,
    mut guard: QueryDurablePreparationGuard,
    outcome: Result<progression::Progress, worth_execution::WorkCeilingDenial>,
) -> Result<WorthQueryCommittedApplicationSession, Stop> {
    let product = guard
        .preparation_mut()
        .query
        .attempt
        .affinity()
        .product_publication()
        .clone();
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
        let generation = retained.authored_completion.as_ref().map(|completion| {
            let output = performed
                .component_results()
                .signal()
                .durable_advance_output()
                .expect("the authored intent performed with its original Signal output");
            completion
                .installed_lowering(output)
                .expect("World admitted the original projection before movement")
                .signal_definition_generation()
        });
        let committed = into_managed(retained).complete_live(performed, generation);
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

fn into_managed(retained: RetainedQueryDurablePreparation) -> ManagedUnpublishedAttempt {
    let query = retained.query;
    ManagedUnpublishedAttempt {
        _completion: query.completion,
        publication_mode: if query
            .attempt
            .idempotency()
            .conditional_definition_identity()
            .is_some()
        {
            managed_unpublished::ManagedUnpublishedPublicationMode::ConditionalDefinition
        } else {
            managed_unpublished::ManagedUnpublishedPublicationMode::Ordinary
        },
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
