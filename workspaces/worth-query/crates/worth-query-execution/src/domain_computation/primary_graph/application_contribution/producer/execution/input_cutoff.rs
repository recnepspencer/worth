//! Producer input preparation consumed by fresh execution or stable publication.

mod refusal;
mod stable_apply;
use super::*;
use crate::domain_computation::primary_graph::fact_verification_observation::input_cutoff::{
    self as observation, Gate, Outcome,
};
use crate::domain_computation::primary_graph::{
    application_contribution::InstalledProducerEdition,
    application_query::PreparedObservedSourceExpectation,
    output_lineage::{
        prepare_stable_address, InputCutoffVerificationStop, OwnedInputCutoffDecision,
        PreparedDecisionReuseContext, PreparedInputCutoffBasis, PreparedInputReuseKey,
        PublishedStableLineage,
    },
    ComputationPrior, WorthQueryAdmittedApplicationOperation, WorthQuerySelectedProductOperation,
};
use refusal::{cutoff_admission_denial, cutoff_basis_denial, cutoff_denial};

type StablePublication<'owner, Schema, OperationType, Input, Scope> = Box<
    dyn FnOnce(
            &WorthQueryAdmittedApplicationOperation<Schema, OperationType, Input, Scope>,
            &mut InvalidationEditAdmission,
        ) -> Result<PublishedStableLineage, ProducerExecutionStop>
        + 'owner,
>;

pub(super) enum ProducerInputProgression<'owner, Schema, OperationType, Input, Scope> {
    FreshPrepared {
        required_output: RequiredOutputDemandContext,
        source: PreparedObservedSourceExpectation,
        key: Option<PreparedInputReuseKey>,
        context: Option<PreparedDecisionReuseContext>,
    },
    StablePrepared(StablePublication<'owner, Schema, OperationType, Input, Scope>),
}

/// An exact selection whose producer declares no Preserve posture reuses the
/// live output it was selected for or stops here, before any effect: that
/// producer never executes over a live output.
/// A producer that retains a partitioned computation hands its run what the
/// selected record retained, or why it has nothing.
fn fresh<'owner, Schema, OperationType, Input, Scope>(
    mut required_output: RequiredOutputDemandContext,
    source: PreparedObservedSourceExpectation,
    key: Option<PreparedInputReuseKey>,
    context: Option<PreparedDecisionReuseContext>,
    prior: Option<ComputationPrior>,
    producer: &'static str,
    reason: &'static str,
) -> Result<
    ProducerInputProgression<'owner, Schema, OperationType, Input, Scope>,
    ProducerExecutionStop,
> {
    if required_output.reuses_live_output_only() {
        return Err(ProducerExecutionStop::LiveOutputNotReused { producer, reason });
    }
    if let Some(prior) = prior {
        required_output.retain_computation_prior(prior);
    }
    observation::finish(Outcome::Fresh);
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
pub(super) fn advance_input_cutoff<'owner, Schema, Binding, OperationType, Input, Scope>(
    phase: &'owner WorthQueryAdvancementPhase<'owner>,
    runtime: &'owner WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    selected: &'owner WorthQuerySelectedProductOperation<'_, Schema>,
    operation: &WorthQueryAdmittedApplicationOperation<Schema, OperationType, Input, Scope>,
    source: PreparedObservedSourceExpectation,
    required_output: RequiredOutputDemandContext,
    key: Option<PreparedInputReuseKey>,
    context: Option<PreparedDecisionReuseContext>,
    matched_predecessors: Option<super::super::demand::MatchedRequiredPredecessors<'_>>,
    edition: InstalledProducerEdition,
    admission: &mut InvalidationEditAdmission,
    currentness: &mut InvalidationEditAdmission,
) -> Result<
    ProducerInputProgression<'owner, Schema, OperationType, Input, Scope>,
    ProducerExecutionStop,
>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    let _observation = observation::Scope::begin(Binding::IDENTITY, Gate::ExecutionRequest);
    let scope = operation.operation_scope_binding();
    let execution = phase
        .execution_request_for(&runtime.product_runtime)
        .map_err(|cause| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()),
                "advancement phase",
            )
        })?;
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
        observation::reach(Gate::ComputationPrior);
        let mut prior = handle
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .computation_prior_at_address::<Operation<Schema, Binding>>(
                scope,
                observation,
                source.partition_identity(),
                edition,
                &execution,
                admission,
            )
            .map_err(cutoff_admission_denial)?;
        handle
            .with_runtime(&execution, |relational| {
                prior.observe_changes(
                    source_owner,
                    relational,
                    selected.application_basis().snapshot_handle(),
                    admission,
                )
            })
            .map_err(cutoff_admission_denial)?;
        Some(prior)
    } else {
        None
    };
    let (key, context) = match (key, context) {
        (Some(key), Some(context)) => (key, context),
        (key, context) => {
            observation::reach(if key.is_none() {
                Gate::PreparedKey
            } else {
                Gate::PreparedContext
            });
            return fresh(
                required_output,
                source,
                key,
                context,
                prior,
                Binding::IDENTITY,
                "prepared input key or decision context unavailable",
            );
        }
    };
    observation::reach(Gate::Candidate);
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
            &execution,
            admission,
        )
        .map_err(cutoff_admission_denial)?;
    let Some(candidate) = candidate else {
        return fresh(
            required_output,
            source,
            Some(key),
            Some(context),
            prior,
            Binding::IDENTITY,
            "prior input cutoff candidate unavailable",
        );
    };
    // Eviction degrades to Fresh. Only a row holds the claims on what its
    // record consumed and answers its pending edges. A candidate of this
    // occurrence that consumed upstream outputs and that no row posts lost its
    // row to custody. Without exact predecessor matching it executes fresh.
    // A wave handoff retains the actual consumer row before its own refresh
    // supersedes the posting. That closed custody proof survives this handoff;
    // Released has no interest or evidence and always executes fresh.
    observation::reach(Gate::RegistryCustody);
    let settlement = candidate.settlement_identity();
    if !candidate.consumed_outputs().is_empty()
        && settlement.address().0 == observation.lifecycle_incarnation()
        && !required_output
            .registry()
            .posts_settlement(settlement, admission)?
    {
        observation::reach(Gate::PredecessorHandoff);
        let handed_off = match matched_predecessors.as_ref() {
            Some(matched) => matched
                .matches_consumer(settlement, admission)
                .map_err(cutoff_admission_denial)?,
            None => false,
        };
        if !handed_off {
            return fresh(
                required_output,
                source,
                Some(key),
                Some(context),
                prior,
                Binding::IDENTITY,
                "consumed-output settlement lost registry custody",
            );
        }
    }
    // The Native owner authenticates and funds the exact issued handle once.
    // Verification consumes this paired proof instead of resolving it again.
    observation::reach(Gate::NativeBasis);
    let basis = handle
        .with_runtime(&execution, |relational| {
            PreparedInputCutoffBasis::prepare(
                relational,
                selected.application_basis().snapshot_handle(),
                admission,
            )
        })
        .map_err(cutoff_basis_denial)?;
    let decision = handle
        .with_runtime(&execution, |relational| {
            candidate.verify_for_reuse_owned(
                &execution,
                key,
                context,
                matched_predecessors,
                relational,
                basis,
                source_owner,
                admission,
                currentness,
            )
        })
        .map_err(cutoff_denial)?;
    let verified = match decision {
        OwnedInputCutoffDecision::Fresh { key, context } => {
            return fresh(
                required_output,
                source,
                Some(key),
                Some(context),
                prior,
                Binding::IDENTITY,
                "prior input cutoff verification requires fresh execution",
            );
        }
        OwnedInputCutoffDecision::Reuse(verified) => verified,
    };
    observation::reach(Gate::SourceConsumption);
    let fresh = source
        .consume_stable(operation)
        .map_err(|error| failed(Binding::IDENTITY, error))?;
    observation::finish(Outcome::Eligible);
    Ok(ProducerInputProgression::StablePrepared(Box::new(
        move |operation, admission| {
            let _observation =
                observation::Scope::begin(Binding::IDENTITY, Gate::PublicationSelection);
            // Completion uses the current head after earlier canonical slots
            // published. Recompare prepared facts; do not rerun a handler read.
            let current = runtime
                .on_branch(selected.product().product_branch())
                .select(phase)
                .map_err(|stop| {
                    WorthQueryOutputDemandDenial::product_selection(
                        stop,
                        "stable publication current Product",
                    )
                })?;
            observation::reach(Gate::PublicationBasis);
            let basis = handle
                .with_runtime(&execution, |relational| {
                    PreparedInputCutoffBasis::prepare(
                        relational,
                        current.application_basis().snapshot_handle(),
                        admission,
                    )
                })
                .map_err(cutoff_basis_denial)?;
            let verified = handle
                .with_runtime(&execution, |relational| {
                    verified.recompare_for_publication(
                        &execution,
                        relational,
                        basis,
                        source_owner,
                        fresh.source_facts(),
                        admission,
                    )
                })
                .map_err(cutoff_denial)?;
            observation::reach(Gate::PublicationApply);
            let result = verified.with_verified(|verified| {
                stable_apply::publish::<Schema, Binding, _, _, _>(
                    runtime,
                    current.product().observation(),
                    &execution,
                    operation,
                    fresh,
                    verified,
                    required_output,
                    admission,
                )
            });
            if result.is_ok() {
                observation::finish(Outcome::Published);
            }
            result
        },
    )))
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
