//! The same original Query proposal and candidate work serve both owner paths.
use super::durable_custody::{
    QueryCandidatePreparationDenial as Denial, QueryNativeCustody, QueryPreparingApplicationCommit,
};
use super::*;

pub(super) fn prepare_before_native(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    prepared: WorthQueryPreparedApplicationCommit,
) -> Result<
    QueryPreparingApplicationCommit,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    let WorthQueryPreparedApplicationCommit {
        mut attempt,
        candidate,
        work,
        retained_preimage,
        preimage_retention_work,
        source_fact_rebase,
        _completion,
    } = prepared;
    let product = attempt.affinity().product_publication().clone();
    let before = precommit_snapshot::WorthQueryPrecommitSnapshot::acquire(
        execution,
        provider.graph.clone(),
        product.observation().basis().relational_basis(),
    )
    .map_err(|denial| {
        snapshot_admission_failure(
            WorthQueryProviderSessionProtocolStage::Commit,
            denial.into(),
            "application publication could not retain its exact pre-commit basis",
        )
    })?;
    let consumed_output::VerifiedConsumedOutputPublication {
        required_prerequisites,
    } = consumed_output::verify(execution, provider, &mut attempt, before.as_snapshot())?;
    let availability_fence =
        availability::prepare(execution, provider, &attempt, before.as_snapshot())?;
    let prepared_output_witness = crate::domain_computation::primary_graph::output_lineage::PreparedNativeOutputWitness::prepare(
        &attempt, &provider.graph.layout,
    ).map_err(native_output_witness_stop)?;
    let candidate = required_origin::bind(
        candidate,
        execution,
        before.as_snapshot(),
        provider,
        &attempt,
        required_prerequisites.as_ref(),
    )?;
    Ok(QueryPreparingApplicationCommit {
        attempt,
        native: QueryNativeCustody::Proposal(candidate),
        work,
        retained_preimage,
        preimage_retention_work,
        completion: Some(_completion),
        before,
        required_prerequisites,
        availability_fence,
        prepared_output_witness,
        source_fact_rebase: Some(source_fact_rebase),
        prepared_touched_records: None,
        index_maintenance_work: None,
        managed_views: None,
    })
}

/// The caller keeps this concrete owner in its external checkout guard. Native
/// consumes the one original proposal; unwind leaves an honest in-call stage.
pub(super) fn prepare_native_candidate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    query: &mut QueryPreparingApplicationCommit,
) -> Result<(), Denial> {
    assert!(matches!(query.native, QueryNativeCustody::Proposal(_)));
    #[cfg(test)]
    provider.fault_port.candidate_ready_for_native_preparation();
    let QueryNativeCustody::Proposal(proposal) =
        std::mem::replace(&mut query.native, QueryNativeCustody::OwnerCallInProgress)
    else {
        unreachable!("the private preparation consumes its original proposal once")
    };
    let candidate = provider
        .graph
        .with_runtime(execution, |runtime| {
            runtime.prepare_validated_proposal(execution, proposal)
        })
        .map_err(Denial::Native)?;
    query.native = QueryNativeCustody::Candidate(candidate);
    Ok(())
}

pub(super) fn prepare_candidate_support(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    query: &mut QueryPreparingApplicationCommit,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<(), Denial> {
    let QueryNativeCustody::Candidate(candidate) = &mut query.native else {
        unreachable!("candidate support follows the one completed Native preparation")
    };
    if let Some(fence) = query.availability_fence.take() {
        fence.bind(candidate).map_err(Denial::OutputWitness)?;
    }
    query.prepared_touched_records = Some(
        touched_records::PreparedTouchedRecords::prepare_retained(candidate, allocation_policy)
            .map_err(Denial::TouchedRecords)?,
    );
    let source_fact_rebase = query
        .source_fact_rebase
        .take()
        .expect("the original source preparation is consumed once");
    query.source_fact_rebase = Some(provider.graph.with_runtime(execution, |runtime| source_fact_rebase.prepare_candidate(
        candidate, runtime, query.before.as_snapshot(),
        crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(
            allocation_policy, Some(query.attempt.affinity().publication_request()),
        ),
    )).map_err(Denial::SourceFacts)?);
    let budget = crate::domain_computation::primary_graph::index_maintenance_budget::ordinary_index_maintenance_budget();
    #[cfg(test)]
    let budget = if provider.take_tight_index_maintenance_budget() {
        worth_relational::facade::indexes::DerivedIndexMaintenanceBudget {
            maximum_work_units: Some(1),
            ..budget
        }
    } else {
        budget
    };
    query.index_maintenance_work = Some(provider.graph.with_runtime(execution, |runtime| {
        crate::domain_computation::primary_graph::index_maintenance_budget::prepare_candidate_with_cold_fallback(
            execution, runtime, candidate, &provider.graph.primary_index_ids, query.before.as_snapshot(), budget,
        )
    }).map_err(Denial::Index)?);
    query.managed_views = managed_views::prepare(
        execution,
        provider,
        query.attempt.affinity().product_publication(),
        query.before.as_snapshot(),
        candidate,
    );
    provider.after_application_candidate_preparation_for_test();
    Ok(())
}

pub(super) fn memory_stop(
    cause: Denial,
    attempt: &WorthQueryPrimaryGraphApplicationAttempt,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    use crate::domain_computation::WorthQueryProviderSessionCommitStop as Stop;
    match cause {
        Denial::Native(error) => match transaction_commit_stop(error) {
            Stop::ControlStopped(stopped)
                if attempt
                    .affinity()
                    .publication_request()
                    .interruption()
                    .is_some() =>
            {
                Stop::ControlStopped(
                    crate::domain_computation::WorthQueryProviderSessionCommitControlStopped::new(
                        stopped.kind(),
                        stopped.detail(),
                    ),
                )
            }
            stop => stop,
        },
        Denial::OutputWitness(stop) => native_output_witness_stop(stop),
        Denial::SourceFacts(cause) => stops::source_rebase_preparation_stop(cause),
        Denial::Index(cause) => index_preparation_stop(cause),
        Denial::TouchedRecords(cause) => cause.into_memory_stop(),
    }
}
