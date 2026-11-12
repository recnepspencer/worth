//! The same original Query proposal and candidate work serve both owner paths.
use super::durable_custody::{
    QueryCandidatePreparationDenial as Denial, QueryNativeCustody, QueryPreparingApplicationCommit,
};
use super::*;

pub(super) fn prepare_before_native(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    prepared: WorthQueryPreparedApplicationCommit,
    owned_history: bool,
) -> Result<
    QueryPreparingApplicationCommit,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    let WorthQueryPreparedApplicationCommit {
        mut attempt,
        members,
        candidate,
        predicate,
        work,
        retained_preimage,
        preimage_retention_work,
        source_fact_rebase,
        _completion,
    } = prepared;
    let group_evidence = coupled::prepare_evidence_slots(execution, members.len())?;
    let group_emissions = coupled::prepare_emissions(execution, &attempt, &members)?;
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
    } = consumed_output::verify(execution, provider, &mut attempt, before.as_snapshot())
        .map_err(consumed_output::memory_stop)?;
    let availability_fence =
        availability::prepare(execution, provider, &attempt, before.as_snapshot())?;
    let candidate = required_origin::bind(
        candidate,
        execution,
        before.as_snapshot(),
        provider,
        &attempt,
        required_prerequisites.as_ref(),
        owned_history,
    )?;
    let predecessor = match (
        required_prerequisites.as_ref(),
        attempt.idempotency().producer_dependency_identity(),
    ) {
        (Some(required), Some(dependency)) => {
            let (producer, _) = required.native_prior_checkpoint_input();
            candidate
                .output_observation_predecessor(execution, producer, &dependency)
                .map_err(transaction_commit_stop)?
        }
        _ => None,
    };
    let source_fact_rebase = source_fact_rebase.with_predecessor(predecessor);
    Ok(QueryPreparingApplicationCommit {
        attempt,
        members,
        group_evidence,
        group_emissions,
        prepared_group_publications: None,
        predicate,
        native: QueryNativeCustody::Proposal(candidate),
        work,
        retained_preimage,
        preimage_retention_work,
        completion: Some(_completion),
        before,
        required_prerequisites,
        availability_fence,
        prepared_output_witness: None,
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
    managed_source::bind(
        execution,
        provider,
        query.before.as_snapshot(),
        &query.attempt,
        candidate,
    )
    .map_err(Denial::ManagedSource)?;
    coupled::prepare_members(
        execution,
        provider,
        &query.attempt,
        &mut query.members,
        candidate,
        query.before.as_snapshot(),
        allocation_policy,
    )?;
    query.prepared_group_publications = if query.members.is_empty() {
        None
    } else {
        Some(
            candidate
                .prepare_required_output_publications(
                    execution,
                    &provider
                        .graph
                        .application_publication
                        .as_ref()
                        .expect("Native admitted the installed group issuer")
                        .native,
                )
                .map_err(Denial::Native)?,
        )
    };
    correspondence::admit(
        execution,
        provider,
        &mut query.attempt,
        &mut query.members,
        candidate,
        query.prepared_group_publications.as_ref(),
    )
    .map_err(Denial::Correspondence)?;
    // Canonical correspondence is admitted before preparing its output witness.
    query.prepared_output_witness =
        crate::domain_computation::primary_graph::output_lineage::PreparedNativeOutputWitness::prepare(
            &query.attempt, &provider.graph.layout,
        ).map_err(Denial::OutputWitness)?;
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
    let index_maintenance_work = candidate.prepared_index_maintenance_work().ok_or_else(|| {
        Denial::Index(worth_relational::facade::indexes::DerivedIndexMaintenanceDenial {
            kind: worth_relational::facade::indexes::DerivedIndexMaintenanceDenialKind::CandidateUnavailable,
            work: Default::default(),
        })
    })?;
    // Native already prepared the indexes. Enforce the caller's tighter policy
    // against that actual work before any World publication begins.
    if provider.take_tight_index_maintenance_budget() && index_maintenance_work.work_units > 1 {
        return Err(Denial::Index(worth_relational::facade::indexes::DerivedIndexMaintenanceDenial {
            kind: worth_relational::facade::indexes::DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded,
            work: index_maintenance_work,
        }));
    }
    query.index_maintenance_work = Some(index_maintenance_work);
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
        Denial::ConsumedOutput(cause) => consumed_output::memory_stop(cause),
        Denial::CoupledSupport(cause) => cause.into_memory_stop(),
        Denial::Dispatch(cause) => match cause {
            crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveAdmissionDenial::Execution(cause) => crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_stop(cause, None),
            crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveAdmissionDenial::Unavailable(detail) => crate::domain_computation::WorthQueryProviderSessionCommitStop::PreEffectDenied(failure(detail)),
        },
        Denial::Correspondence(cause) => cause.into_memory_stop(),
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
        Denial::ManagedSource(cause) => cause.into_memory_stop(),
        Denial::OutputWitness(stop) => native_output_witness_stop(stop),
        Denial::SourceFacts(cause) => stops::source_rebase_preparation_stop(cause),
        Denial::Index(cause) => index_preparation_stop(cause),
        Denial::TouchedRecords(cause) => cause.into_memory_stop(),
    }
}
