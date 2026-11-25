use worth_execution::{MapKernelStop, WorkCeilingDenial};
use worth_query_execution::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    domain_computation::{
        WorthQueryProviderSessionControlStopKind, WorthQueryProviderSessionDenialKind,
    },
    primary_graph::{
        WorthQueryConditionalClockObservationDenial,
        WorthQueryConditionalClockObservationDenialKind,
        WorthQueryConditionalClockObservationFailure,
        WorthQueryConditionalClockObservationFailureKind,
        WorthQueryConditionalRuntimeInstallationDenialKind,
    },
    product::{
        WorthQueryProductBranchAdmissionDenial, WorthQueryProductBranchAdmissionRefusalKind,
    },
};

use super::super::WorthQueryTemporalCatchUpStatus;

pub(in super::super) fn classify_work_ceiling_denial(
    denial: &WorkCeilingDenial,
) -> WorthQueryTemporalCatchUpStatus {
    match denial {
        WorkCeilingDenial::Stopped(MapKernelStop::Cancelled) => {
            WorthQueryTemporalCatchUpStatus::Cancelled
        }
        WorkCeilingDenial::Stopped(
            MapKernelStop::WorkCeiling
            | MapKernelStop::WorkCounterOverflow
            | MapKernelStop::DeadlineElapsed
            | MapKernelStop::NestedStopped
            | MapKernelStop::AccountingUnavailable,
        )
        | WorkCeilingDenial::Admission(_)
        | WorkCeilingDenial::Panicked => WorthQueryTemporalCatchUpStatus::BudgetExhausted,
    }
}

pub(in super::super) fn classify_advancement_stop(
    denial: &WorthQueryAdvancementDenial,
) -> Option<WorthQueryTemporalCatchUpStatus> {
    match denial {
        WorthQueryAdvancementDenial::Interrupted(
            WorthQueryManagedComputationInterruption::Cancelled,
        ) => Some(WorthQueryTemporalCatchUpStatus::Cancelled),
        WorthQueryAdvancementDenial::Interrupted(
            WorthQueryManagedComputationInterruption::DeadlineExceeded,
        )
        | WorthQueryAdvancementDenial::Resource(_) => {
            Some(WorthQueryTemporalCatchUpStatus::BudgetExhausted)
        }
        WorthQueryAdvancementDenial::NestedOpening
        | WorthQueryAdvancementDenial::NestedStopped
        | WorthQueryAdvancementDenial::Panicked => None,
    }
}

pub(in super::super) fn classify_branch_stop(
    denial: &WorthQueryProductBranchAdmissionDenial,
) -> Option<WorthQueryTemporalCatchUpStatus> {
    match denial {
        WorthQueryProductBranchAdmissionDenial::Stopped(stop) => match stop {
            Ok(_) => Some(WorthQueryTemporalCatchUpStatus::BudgetExhausted),
            Err(WorthQueryProviderSessionControlStopKind::Cancelled) => {
                Some(WorthQueryTemporalCatchUpStatus::Cancelled)
            }
            Err(WorthQueryProviderSessionControlStopKind::TimedOut) => {
                Some(WorthQueryTemporalCatchUpStatus::BudgetExhausted)
            }
        },
        WorthQueryProductBranchAdmissionDenial::Refused(
            WorthQueryProductBranchAdmissionRefusalKind::ObservationCapacityExhausted
            | WorthQueryProductBranchAdmissionRefusalKind::CustodyCapacityExhausted
            | WorthQueryProductBranchAdmissionRefusalKind::ActiveSnapshotCapacityExhausted { .. }
            | WorthQueryProductBranchAdmissionRefusalKind::RetentionCapacityExhausted,
        ) => Some(WorthQueryTemporalCatchUpStatus::Backpressured),
        WorthQueryProductBranchAdmissionDenial::Refused(_) => None,
    }
}

pub(in super::super) fn classify_clock_port_denial(
    denial: &WorthQueryConditionalClockObservationDenial,
) -> Option<WorthQueryTemporalCatchUpStatus> {
    classify_clock_port_denial_kind(&denial.kind())
}

pub(super) fn classify_clock_port_denial_kind(
    kind: &WorthQueryConditionalClockObservationDenialKind,
) -> Option<WorthQueryTemporalCatchUpStatus> {
    let WorthQueryConditionalClockObservationDenialKind::ProductAdmission(kind) = kind else {
        return None;
    };
    match kind {
        WorthQueryConditionalRuntimeInstallationDenialKind::ReconstructionPrincipal => {
            Some(WorthQueryTemporalCatchUpStatus::AuthorizationRefused)
        }
        WorthQueryConditionalRuntimeInstallationDenialKind::ReadExecution(Err(
            WorthQueryProviderSessionControlStopKind::Cancelled,
        )) => Some(WorthQueryTemporalCatchUpStatus::Cancelled),
        WorthQueryConditionalRuntimeInstallationDenialKind::ReadExecution(Err(
            WorthQueryProviderSessionControlStopKind::TimedOut,
        ))
        | WorthQueryConditionalRuntimeInstallationDenialKind::ReadExecution(Ok(
            WorthQueryProviderSessionDenialKind::ExecutionResource { .. },
        )) => Some(WorthQueryTemporalCatchUpStatus::BudgetExhausted),
        WorthQueryConditionalRuntimeInstallationDenialKind::ActiveSnapshotCapacityExhausted {
            ..
        }
        | WorthQueryConditionalRuntimeInstallationDenialKind::RetentionCapacityExhausted
        | WorthQueryConditionalRuntimeInstallationDenialKind::ManagedTemporal(
            worth_runtime_bridge::facade::BridgeManagedTemporalDenialKind::RetentionCapacityExhausted
            | worth_runtime_bridge::facade::BridgeManagedTemporalDenialKind::IntentCapacityExhausted,
        ) => Some(WorthQueryTemporalCatchUpStatus::Backpressured),
        _ => None,
    }
}

pub(in super::super) fn classify_observation_failure(
    failure: WorthQueryConditionalClockObservationFailure,
) -> WorthQueryTemporalCatchUpStatus {
    classify_observation_failure_kind(&failure.kind())
        .unwrap_or(WorthQueryTemporalCatchUpStatus::ObservationFailed(failure))
}

pub(super) fn classify_observation_failure_kind(
    kind: &WorthQueryConditionalClockObservationFailureKind,
) -> Option<WorthQueryTemporalCatchUpStatus> {
    match kind {
        WorthQueryConditionalClockObservationFailureKind::ExecutionRequest(denial) => {
            classify_advancement_stop(denial)
        }
        WorthQueryConditionalClockObservationFailureKind::ActiveSnapshotCapacityExhausted {
            ..
        }
        | WorthQueryConditionalClockObservationFailureKind::RetentionCapacityExhausted
        | WorthQueryConditionalClockObservationFailureKind::ReconstructionDenied(
            WorthQueryConditionalRuntimeInstallationDenialKind::ActiveSnapshotCapacityExhausted {
                ..
            }
            | WorthQueryConditionalRuntimeInstallationDenialKind::RetentionCapacityExhausted
            | WorthQueryConditionalRuntimeInstallationDenialKind::ManagedTemporal(
                worth_runtime_bridge::facade::BridgeManagedTemporalDenialKind::RetentionCapacityExhausted
                | worth_runtime_bridge::facade::BridgeManagedTemporalDenialKind::IntentCapacityExhausted,
            ),
        ) => Some(WorthQueryTemporalCatchUpStatus::Backpressured),
        _ => None,
    }
}
