use worth_signal::facade::adapters::SignalInvalidationExecutionReceipt;
use worth_signal::facade::branch::SignalConditionalServiceExecutionRequest;
use worth_signal::facade::SignalConditionalDecisionEvidence;

use super::resolver_adapters::{ComparatorAdapter, ConditionAdapter};
use super::retained_decision::BridgeRetainedConditionalDecisionCore;
use super::{
    BridgeConditionalDecisionEvidence, BridgeConditionalDenial, BridgeConditionalDenialKind,
    BridgeConditionalEvaluationAdmissionRequest, BridgeConditionalEvaluationSession,
    BridgeInstalledConditionalLowering, BridgeOwnedSignalRuntime,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BridgeConditionalExecutionCounters {
    pub signal_graph_checks: usize,
    pub snapshot_admission_attempts: usize,
    pub signal_slot_reuse_hits: usize,
    pub compute_provider_checks: usize,
    pub signal_execution_contacts: usize,
    pub observation_baseline_writes: usize,
    pub decisions_retained: usize,
    pub unrelated_lowering_scans: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BridgeConditionalReentryCounters {
    pub runtime_key_checks: usize,
    pub lowering_identity_checks: usize,
    pub installed_lowering_lookups: usize,
    pub signal_graph_checks: usize,
    pub signal_contract_checks: usize,
    pub snapshot_identity_checks: usize,
    pub query_continuation_rebindings: usize,
    pub unrelated_lowering_scans: usize,
}

pub struct BridgeConditionalExecutionRequest<'a> {
    pub lowering: &'a std::sync::Arc<BridgeInstalledConditionalLowering>,
    pub query_binding_identity: &'a str,
    pub query_capability_identity: u64,
    pub snapshot_identity: &'a str,
    pub truth_branch_identity: Option<&'a str>,
    pub bridge_snapshot_identity: Option<&'a crate::snapshot::TruthSnapshotIdentity>,
    pub execution_identity: &'a str,
    pub attempt: u64,
}

pub struct BridgeConditionalQueryContinuationAdmission<'a> {
    pub lowering: &'a std::sync::Arc<BridgeInstalledConditionalLowering>,
    pub query_binding_identity: &'a str,
    pub query_capability_identity: u64,
    pub signal_snapshot_projection: &'a str,
    pub bridge_snapshot_identity: Option<&'a crate::snapshot::TruthSnapshotIdentity>,
    pub signal_execution_projection: &'a str,
    pub attempt: u64,
}

impl BridgeOwnedSignalRuntime {
    pub fn execute(
        &self,
        signal_basis: &super::BridgeConditionalSignalBasisBinding,
        request: BridgeConditionalExecutionRequest<'_>,
        compute_context: &mut dyn std::any::Any,
    ) -> Result<BridgeConditionalDecisionEvidence, BridgeConditionalDenial> {
        self.execute_with_managed_source_record(signal_basis, request, None, compute_context)
    }

    pub(super) fn execute_with_managed_source_record(
        &self,
        signal_basis: &super::BridgeConditionalSignalBasisBinding,
        request: BridgeConditionalExecutionRequest<'_>,
        managed_source_record: Option<
            crate::relational_identity::RelationalBridgeRecordIdentityParts,
        >,
        compute_context: &mut dyn std::any::Any,
    ) -> Result<BridgeConditionalDecisionEvidence, BridgeConditionalDenial> {
        let mut admission_counters = BridgeConditionalExecutionCounters::default();
        if request.bridge_snapshot_identity.is_some() {
            admission_counters.signal_graph_checks = 1;
            admission_counters.snapshot_admission_attempts = 1;
        }
        let session = self
            .admit_conditional_evaluation_with_record(
                match request.bridge_snapshot_identity {
                    Some(identity) => {
                        BridgeConditionalEvaluationAdmissionRequest::source_present_at_signal_basis(
                            signal_basis,
                            identity,
                        )
                    }
                    None => {
                        BridgeConditionalEvaluationAdmissionRequest::source_free_at_signal_basis(
                            signal_basis,
                        )
                    }
                },
                managed_source_record,
            )
            .map_err(|denial| {
                if denial.kind() == BridgeConditionalDenialKind::StaleLowering {
                    admission_counters.snapshot_admission_attempts = 0;
                }
                denial.with_bridge_execution_counters(admission_counters)
            })?;
        self.execute_admitted_conditional(&session, request, compute_context)
    }

    pub fn execute_admitted_conditional(
        &self,
        session: &BridgeConditionalEvaluationSession,
        request: BridgeConditionalExecutionRequest<'_>,
        compute_context: &mut dyn std::any::Any,
    ) -> Result<BridgeConditionalDecisionEvidence, BridgeConditionalDenial> {
        let mut counters = BridgeConditionalExecutionCounters {
            signal_graph_checks: 1,
            snapshot_admission_attempts: 0,
            ..BridgeConditionalExecutionCounters::default()
        };
        self.validate_evaluation_session(session, &request)
            .map_err(|denial| denial.with_bridge_execution_counters(counters))?;
        let decision_reservation = super::retention::reserve_decision(&self.retention, &request)?;
        let context_reservation = super::retention::reserve_context(&self.retention, &request)?;
        let execution = self.execute_installed_signal_conditional(
            session,
            &request,
            compute_context,
            &mut counters,
            context_reservation.as_ref(),
        );
        let (signal, observations, performed_signal_invalidation) =
            execution.map_err(|denial| denial.with_bridge_execution_counters(counters))?;
        retain_successful_observation_baseline(session, &observations);
        counters.observation_baseline_writes = observations.len();
        counters.decisions_retained = 1;
        Ok(retain_bridge_decision(
            &request,
            self.bridge.signal_runtime_key,
            RetainedBridgeDecisionOutcome {
                reservation: decision_reservation,
                source_snapshot: session.source_snapshot.as_ref().map(std::sync::Arc::clone),
                signal,
                observations,
                counters,
                performed_signal_invalidation,
            },
        ))
    }

    fn execute_installed_signal_conditional(
        &self,
        session: &BridgeConditionalEvaluationSession,
        request: &BridgeConditionalExecutionRequest<'_>,
        compute_context: &mut dyn std::any::Any,
        counters: &mut BridgeConditionalExecutionCounters,
        context_reservation: Option<&std::sync::Arc<super::retention::BridgeRetentionReservation>>,
    ) -> Result<
        (
            SignalConditionalDecisionEvidence,
            super::observation_retention::BridgeRetainedObservations,
            Option<SignalInvalidationExecutionReceipt>,
        ),
        BridgeConditionalDenial,
    > {
        counters.compute_provider_checks = 1;
        let compute = request.lowering.providers.compute.as_ref().ok_or_else(|| {
            BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::MissingComputeProvider,
                "installed conditional lowering lost its exact compute provider",
            )
        })?;
        let previous_observations = session.observation_baselines.snapshot()?;
        let mut condition = ConditionAdapter::new(
            request.lowering,
            session.source_snapshot.as_deref(),
            &previous_observations,
            session.managed_source_record,
            request.truth_branch_identity,
            request.snapshot_identity,
            &session.observation_baselines.ledger,
            context_reservation,
        );
        let mut comparator = ComparatorAdapter::new(request.lowering);
        counters.signal_execution_contacts = 1;
        let signal_request = if request
            .lowering
            .providers
            .trigger
            .as_ref()
            .is_some_and(|provider| provider.requested())
        {
            SignalConditionalServiceExecutionRequest::new(request.attempt).force_on_demand()
        } else {
            SignalConditionalServiceExecutionRequest::new(request.attempt)
        };
        let completion = session
            .signal_port
            .execute(
                &session.signal,
                signal_request,
                &mut condition,
                &mut comparator,
                || {
                    compute
                        .compute(compute_context)
                        .map_err(worth_signal::facade::SignalError::invalid_input)
                },
            )
            .map_err(signal_service_denial)?;
        counters.signal_slot_reuse_hits = usize::from(completion.slot_reused());
        if !completion.slot_reused() {
            counters.snapshot_admission_attempts = session.snapshot_admission_attempts;
        }
        let (signal, performed_signal_invalidation) = completion.into_parts();
        let signal = admit_signal_execution(signal, &mut condition)?;
        let performed_signal_invalidation = performed_signal_invalidation.map_err(|error| {
            BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::SignalExecution,
                error.to_string(),
            )
        })?;
        let observations = condition.take_observations();
        Ok((signal, observations, performed_signal_invalidation))
    }

    pub(super) fn require_current_signal_graph(
        &self,
        lowering: &BridgeInstalledConditionalLowering,
    ) -> Result<(), BridgeConditionalDenial> {
        if lowering.signal_contract().graph_instance_id() == self.signal_graph_instance_id {
            return Ok(());
        }
        Err(BridgeConditionalDenial::new(
            BridgeConditionalDenialKind::StaleLowering,
            "conditional lowering belongs to another Signal graph",
        ))
    }

    fn validate_evaluation_session(
        &self,
        session: &BridgeConditionalEvaluationSession,
        request: &BridgeConditionalExecutionRequest<'_>,
    ) -> Result<(), BridgeConditionalDenial> {
        let requested_snapshot = request.bridge_snapshot_identity;
        let retained_snapshot = session
            .source_snapshot
            .as_ref()
            .map(|snapshot| snapshot.snapshot_identity());
        if session.bridge_runtime_key != self.bridge.signal_runtime_key
            || !std::sync::Arc::ptr_eq(&session.lowering, request.lowering)
            || retained_snapshot != requested_snapshot
        {
            return Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::SnapshotAdmission,
                "conditional evaluation session does not match this runtime, lowering, or source",
            ));
        }
        self.require_live_installed_lowering(request.lowering)
    }
}

fn retain_successful_observation_baseline(
    session: &BridgeConditionalEvaluationSession,
    observations: &super::observation_retention::BridgeRetainedObservations,
) {
    if !observations.is_empty() {
        session.observation_baselines.publish(observations.clone());
    }
}

fn admit_signal_execution(
    signal: Result<
        SignalConditionalDecisionEvidence,
        worth_signal::facade::SignalConditionalExecutionFailure,
    >,
    condition: &mut ConditionAdapter<'_>,
) -> Result<SignalConditionalDecisionEvidence, BridgeConditionalDenial> {
    match signal {
        Ok(signal) => Ok(signal),
        Err(failure) => {
            let counters = failure.counters();
            let observation_reads = condition.observation_count();
            if let Some(denial) = condition.take_observation_denial() {
                return Err(denial.with_execution_counters(counters, observation_reads));
            }
            Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::SignalExecution,
                format!("{:?}", failure.into_error()),
            )
            .with_execution_counters(counters, observation_reads))
        }
    }
}

pub(super) fn signal_service_denial(
    error: worth_signal::facade::branch::SignalConditionalServiceExecutionDenial,
) -> BridgeConditionalDenial {
    use worth_signal::facade::branch::SignalConditionalServiceExecutionDenial as Denial;
    let kind = match error {
        Denial::AdmissionCapacityExhausted => {
            BridgeConditionalDenialKind::ConditionalEvaluationAdmissionCapacity
        }
        Denial::SlotBusy => BridgeConditionalDenialKind::ConditionalEvaluationBusy,
        Denial::SlotPoisoned => BridgeConditionalDenialKind::ConditionalEvaluationPoisoned,
        Denial::UnconsumedUnwind => BridgeConditionalDenialKind::ConditionalEvaluationUnwindPending,
        Denial::StaleBasisAdmission
        | Denial::DefinitionReadmissionRequired
        | Denial::DefinitionMismatch => BridgeConditionalDenialKind::StaleLowering,
        Denial::MissingSourceEvidence => BridgeConditionalDenialKind::MissingSourceObservation,
        Denial::UnexpectedSourceEvidence => BridgeConditionalDenialKind::SourcePostureMismatch,
        Denial::SourceAuthorityMismatch => BridgeConditionalDenialKind::OperationAuthorityMismatch,
        Denial::OwnerUnavailable(_)
        | Denial::OwnerAdmission(_)
        | Denial::NestedOperationScopeMismatch
        | Denial::EvaluationIdentityExhausted
        | Denial::AdmissionUnavailable
        | Denial::SlotAdmission(_)
        | Denial::ObservationAdmission(_) => BridgeConditionalDenialKind::SignalExecution,
    };
    BridgeConditionalDenial::new(
        kind,
        format!("Signal conditional service denied execution: {error:?}"),
    )
}

struct RetainedBridgeDecisionOutcome {
    reservation: super::retention::DecisionReservations,
    source_snapshot: Option<
        std::sync::Arc<
            crate::snapshot::AdmittedSnapshotContext<Box<dyn crate::snapshot::TruthSnapshotReader>>,
        >,
    >,
    signal: SignalConditionalDecisionEvidence,
    observations: super::observation_retention::BridgeRetainedObservations,
    counters: BridgeConditionalExecutionCounters,
    performed_signal_invalidation: Option<SignalInvalidationExecutionReceipt>,
}

fn retain_bridge_decision(
    request: &BridgeConditionalExecutionRequest<'_>,
    bridge_runtime_key: u64,
    outcome: RetainedBridgeDecisionOutcome,
) -> BridgeConditionalDecisionEvidence {
    let RetainedBridgeDecisionOutcome {
        reservation,
        source_snapshot,
        signal,
        observations,
        counters,
        performed_signal_invalidation,
    } = outcome;
    BridgeConditionalDecisionEvidence {
        _reservation: reservation.evidence,
        core: std::sync::Arc::new(BridgeRetainedConditionalDecisionCore {
            _reservation: reservation.core,
            bridge_runtime_key,
            lowering: std::sync::Arc::clone(request.lowering),
            source_snapshot,
            signal_snapshot_projection: request.snapshot_identity.into(),
            signal_execution_projection: request.execution_identity.into(),
            attempt: request.attempt,
            signal,
            semantic_observations: observations,
            bridge_execution_counters: counters,
            triggering_change_set: None,
        }),
        query_binding_identity: request.query_binding_identity.into(),
        query_capability_identity: request.query_capability_identity,
        reentry_counters: BridgeConditionalReentryCounters::default(),
        performed_signal_invalidation,
    }
}
