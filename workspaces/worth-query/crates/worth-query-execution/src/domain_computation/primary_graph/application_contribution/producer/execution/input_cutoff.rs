//! Producer input preparation consumed by fresh execution or stable publication.

use super::*;
use crate::domain_computation::primary_graph::{
    application_contribution::InstalledProducerEdition,
    application_query::PreparedObservedSourceExpectation,
    output_lineage::{
        prepare_stable_address, InputCutoffDecision, InputCutoffVerificationStop,
        PreparedDecisionReuseContext, PreparedInputCutoffBasis, PreparedInputReuseKey,
        PublishedStableLineage,
    },
    ComputationPrior, WorthQueryAdmittedApplicationOperation, WorthQuerySelectedProductOperation,
};

pub(super) enum ProducerInputProgression {
    FreshPrepared {
        required_output: RequiredOutputDemandContext,
        source: PreparedObservedSourceExpectation,
        key: Option<PreparedInputReuseKey>,
        context: Option<PreparedDecisionReuseContext>,
    },
    StablePublished(PublishedStableLineage),
}

/// An exact selection whose producer declares no Preserve posture reuses the
/// live output it was selected for or stops here, before any effect: that
/// producer never executes over a live output.
/// A producer that retains a partitioned computation hands its run what the
/// selected record retained, or why it has nothing.
fn fresh(
    mut required_output: RequiredOutputDemandContext,
    source: PreparedObservedSourceExpectation,
    key: Option<PreparedInputReuseKey>,
    context: Option<PreparedDecisionReuseContext>,
    prior: Option<ComputationPrior>,
) -> Result<ProducerInputProgression, ProducerExecutionStop> {
    if required_output.reuses_live_output_only() {
        return Err(ProducerExecutionStop::LiveOutputNotReused);
    }
    if let Some(prior) = prior {
        required_output.retain_computation_prior(prior);
    }
    Ok(ProducerInputProgression::FreshPrepared {
        required_output,
        source,
        key,
        context,
    })
}

/// Eligibility precedes lineage storage, registry claims, and source-image
/// construction. The Fresh branch returns the original preparation custody;
/// no input/context proof or source fact set is reconstructed on fallback.
pub(super) fn advance_input_cutoff<Schema, Binding, OperationType, Input, Scope>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    operation: &WorthQueryAdmittedApplicationOperation<Schema, OperationType, Input, Scope>,
    source: PreparedObservedSourceExpectation,
    required_output: RequiredOutputDemandContext,
    key: Option<PreparedInputReuseKey>,
    context: Option<PreparedDecisionReuseContext>,
    matched_predecessors: Option<super::super::demand::MatchedRequiredPredecessors<'_>>,
    resources: super::super::WorthQueryProducerDemandResources,
    edition: InstalledProducerEdition,
    admission: &mut InvalidationEditAdmission,
    currentness: &mut InvalidationEditAdmission,
) -> Result<ProducerInputProgression, ProducerExecutionStop>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    let scope = operation.operation_scope_binding();
    let observation = selected.product().observation();
    let handle = &runtime.primary_provider.graph;
    let source_owner = &handle.source_owner.invalidation_owner;
    // A producer whose operation runs a retained partitioned computation
    // takes the live record at the demand's own address, the one its
    // publication replaces: the run compares its edition, its input digest
    // and every fact it retained, so reusing a partition needs no
    // whole-output input reuse. The lookup charges `admission` for the
    // partition index entries and the record it reads. No other producer
    // looks up or pays for one.
    let prior = if edition.retains_computation() {
        let prior = handle
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .computation_prior_at_address::<Operation<Schema, Binding>>(
                scope,
                observation,
                source.partition_identity(),
                edition,
                admission,
            )
            .map_err(cutoff_admission_denial)?;
        Some(prior)
    } else {
        None
    };
    let (key, context) = match (key, context) {
        (Some(key), Some(context)) => (key, context),
        (key, context) => return fresh(required_output, source, key, context, prior),
    };
    admission
        .charge_external_work(4)
        .map_err(cutoff_admission_denial)?;
    let candidate = handle
        .output_lineage
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .prior_input_cutoff_candidate::<Operation<Schema, Binding>>(
            scope,
            observation,
            source.partition_identity(),
            admission,
        )
        .map_err(cutoff_admission_denial)?;
    let Some(candidate) = candidate else {
        return fresh(required_output, source, Some(key), Some(context), prior);
    };
    // Eviction degrades to Fresh. Only a row holds the claims on what its
    // record consumed and answers its pending edges. A candidate of this
    // occurrence that consumed upstream outputs and that no row posts lost its
    // row to custody: it is neither reused nor waited on, and the execution
    // commits as its successor.
    let settlement = candidate.settlement_identity();
    if !candidate.consumed_outputs().is_empty()
        && settlement.address().0 == observation.lifecycle_incarnation()
        && !required_output
            .registry()
            .posts_settlement(settlement, admission)?
    {
        return fresh(required_output, source, Some(key), Some(context), prior);
    }
    // The Native owner authenticates and funds the exact issued handle once.
    // Verification consumes this paired proof instead of resolving it again.
    let basis = handle
        .with_runtime(|relational| {
            PreparedInputCutoffBasis::prepare(
                relational,
                selected.application_basis().snapshot_handle(),
                admission,
            )
        })
        .map_err(cutoff_basis_denial)?;
    let decision = handle
        .with_runtime(|relational| {
            candidate.verify_for_reuse(
                key,
                context,
                matched_predecessors,
                relational,
                &basis,
                source_owner,
                admission,
                currentness,
            )
        })
        .map_err(cutoff_denial)?;
    let verified = match decision {
        InputCutoffDecision::Fresh { key, context } => {
            return fresh(required_output, source, Some(key), Some(context), prior);
        }
        InputCutoffDecision::Reuse(verified) => verified,
    };
    let fresh = source
        .consume_stable(operation)
        .map_err(|error| failed(Binding::IDENTITY, error))?;
    let lane = runtime
        .primary_provider
        .admitted_application_branch_commit_lane(observation, admission)
        .map_err(|stop| {
            use crate::domain_computation::primary_graph::provider::WorthQueryBranchCommitLaneDenial as Stop;
            match stop {
                Stop::Preparation(stop) => cutoff_admission_denial(stop),
                Stop::RetainedCapacityExhausted { .. } | Stop::CapacityCounterOverflow => denial(
                    WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                    "producer input cutoff branch coordination",
                ).with_recovery_posture(crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable),
            }
        })?;
    let coordination = lane.enter();
    let address = prepare_stable_address::<Operation<Schema, Binding>>(
        &handle.output_lineage,
        verified,
        scope,
        observation,
        &coordination,
        admission,
    )?;
    let publication = address.prepare_record(fresh, operation, resources, admission)?;
    // Registry preparation is outside both lineage and Product guards.
    let mut prerequisites = required_output.prepare_prerequisites(
        publication
            .consumed_outputs()
            .iter()
            .map(|consumed| consumed.identity()),
        admission,
    )?;
    prerequisites.reserve_identity(publication.identity(), admission)?;
    let registration = publication.prepare_current_registration(
        source_owner,
        prerequisites.work_membership(),
        admission,
    )
        .map_err(|stop| match stop {
            crate::domain_computation::primary_graph::output_lineage::invalidation::SettlementRegistrationStop::Admission(stop) => cutoff_admission_denial(stop),
            _ => denial(WorthQueryOutputDemandDenialKind::PublicationStale, "producer input cutoff source changed during preparation"),
        })?;
    let displaced = publication.displaced_settlement();
    let published = publication.publish::<Operation<Schema, Binding>, _, _, _, _>(
        operation,
        observation,
        &runtime.product_runtime.owner.observation_port(),
        registration,
        admission,
    ).map_err(|stop| match stop {
        crate::domain_computation::primary_graph::output_lineage::StablePublicationStop::Demand(stop) => ProducerExecutionStop::ExecutionStopped(stop),
        crate::domain_computation::primary_graph::output_lineage::StablePublicationStop::RequestAuthority(stop) => super::denial::request_authority_denied(Binding::IDENTITY, stop),
    })?;
    prerequisites
        .publish(Arc::clone(published.exact_settlement()))
        .retire(displaced, source_owner);
    Ok(ProducerInputProgression::StablePublished(published))
}

/// Basis preparation may refuse its first claim, so no diagnostic allocation
/// can depend on a budget that has already been exhausted.
fn cutoff_basis_denial(stop: InputCutoffVerificationStop) -> WorthQueryOutputDemandDenial {
    use worth_relational::facade::mvcc::CompanionPreflightStop as Admission;
    let kind = match stop {
        InputCutoffVerificationStop::Admission(
            Admission::WorkExhausted { .. } | Admission::WorkCounterOverflow,
        )
        | InputCutoffVerificationStop::WorkExhausted => {
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        }
        InputCutoffVerificationStop::Admission(Admission::Interrupted(event)) => {
            WorthQueryOutputDemandDenialKind::of_interruption(event.interruption())
        }
        InputCutoffVerificationStop::Admission(_) => {
            WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
        }
        InputCutoffVerificationStop::SelectedSourceUnavailable => {
            WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable
        }
        _ => WorthQueryOutputDemandDenialKind::PublicationStale,
    };
    denial(kind, String::new())
}

fn cutoff_admission_denial(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> WorthQueryOutputDemandDenial {
    use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
    denial(
        match stop {
            Stop::WorkExhausted { .. } | Stop::WorkCounterOverflow => {
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
            }
            Stop::Interrupted(event) => {
                WorthQueryOutputDemandDenialKind::of_interruption(event.interruption())
            }
            _ => WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        },
        String::new(),
    )
}

fn cutoff_denial(stop: InputCutoffVerificationStop) -> WorthQueryOutputDemandDenial {
    match stop {
        InputCutoffVerificationStop::Admission(stop) => cutoff_admission_denial(stop),
        InputCutoffVerificationStop::WorkExhausted => denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, String::new()),
        InputCutoffVerificationStop::PendingUpstream => denial(WorthQueryOutputDemandDenialKind::SchedulingDeferred, "producer input cutoff awaits upstream outputs")
            .with_recovery_posture(crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable),
        _ => denial(WorthQueryOutputDemandDenialKind::PublicationStale, "producer input cutoff source moved"),
    }
}

pub(super) fn source_preparation_denial(
    identity: &str,
    stop: crate::domain_computation::primary_graph::WorthQuerySourceExpectationDenial,
) -> WorthQueryOutputDemandDenial {
    use crate::domain_computation::primary_graph::WorthQuerySourceExpectationDenialKind as Kind;
    match stop.kind() {
        Kind::WorkBudgetExceeded => denial(
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
            format!("{identity}: {stop}"),
        ),
        Kind::PreparationMemoryExceeded => denial(
            WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
            format!("{identity}: {stop}"),
        ),
        _ => failed(identity, stop),
    }
}
