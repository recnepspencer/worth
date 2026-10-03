//! Producer input preparation consumed by fresh execution or stable publication.

use super::*;
use crate::domain_computation::primary_graph::{
    application_query::PreparedObservedSourceExpectation,
    output_lineage::{
        prepare_stable_address, InputCutoffDecision, InputCutoffVerificationStop,
        PreparedDecisionReuseContext, PreparedInputCutoffBasis, PreparedInputReuseKey,
        PublishedStableLineage,
    },
    WorthQueryAdmittedApplicationOperation, WorthQuerySelectedProductOperation,
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
    matched_predecessor: Option<super::super::demand::MatchedRequiredPredecessor<'_>>,
    resources: super::super::WorthQueryProducerDemandResources,
    admission: &mut InvalidationEditAdmission,
    currentness: &mut InvalidationEditAdmission,
) -> Result<ProducerInputProgression, ProducerExecutionStop>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    let (key, context) = match (key, context) {
        (Some(key), Some(context)) => (key, context),
        (key, context) => {
            return Ok(ProducerInputProgression::FreshPrepared {
                required_output,
                source,
                key,
                context,
            });
        }
    };
    let scope = operation.operation_scope_binding();
    let observation = selected.product().observation();
    let handle = &runtime.primary_provider.graph;
    let source_owner = &handle.source_owner.invalidation_owner;
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
        return Ok(ProducerInputProgression::FreshPrepared {
            required_output,
            source,
            key: Some(key),
            context: Some(context),
        });
    };
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
                matched_predecessor,
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
            return Ok(ProducerInputProgression::FreshPrepared {
                required_output,
                source,
                key: Some(key),
                context: Some(context),
            });
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
        .retire(source_owner);
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
