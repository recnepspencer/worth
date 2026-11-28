use super::{
    provider_failure, WorthQueryProviderSessionFailure, WorthQueryProviderSessionProtocolStage,
};
use crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_stop;

pub(super) fn failure(detail: &'static str) -> WorthQueryProviderSessionFailure {
    provider_failure(WorthQueryProviderSessionProtocolStage::Commit, detail)
}

pub(super) fn native_output_witness_stop(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    crate::domain_computation::WorthQueryProviderSessionCommitStop::Deferred(
        crate::domain_computation::WorthQueryProviderSessionCommitDeferred::new(
            crate::domain_computation::WorthQueryProviderSessionCommitDeferredKind::RelationalDeferred(
                worth_relational::facade::mvcc::RelationalPublicationDeferred::CompanionPreflight(
                    stop,
                ),
            ),
            "native output witness preparation denied before publication",
        ),
    )
}

pub(super) fn index_preparation_stop(
    denial: worth_relational::facade::indexes::DerivedIndexMaintenanceDenial,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    use worth_relational::facade::indexes::DerivedIndexMaintenanceDenialKind as Kind;
    let kind = match denial.kind {
        Kind::Execution(cause) => return relational_execution_stop(cause, None),
        Kind::WorkBudgetExceeded
        | Kind::ColdRecordSlotBudgetExceeded
        | Kind::ColdReconstructionRequired => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::IndexMaintenanceBudgetExceeded
        }
        Kind::GenerationIdentityExhausted => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::IndexGenerationIdentityExhausted
        }
        Kind::Basis(_)
        | Kind::SnapshotUnavailable
        | Kind::CommitMismatch
        | Kind::BeforeRootMismatch
        | Kind::IndexUnavailable(_)
        | Kind::GenerationKindMismatch(_)
        | Kind::PriorEntryMismatch
        | Kind::ForeignCandidate
        | Kind::CandidateUnavailable
        | Kind::CandidateIndexesAlreadyPrepared => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::ProviderRejected
        }
    };
    crate::domain_computation::WorthQueryProviderSessionCommitStop::PreEffectDenied(
        WorthQueryProviderSessionFailure::new(
            kind,
            WorthQueryProviderSessionProtocolStage::Commit,
            format!("primary index candidate preparation denied: {denial:?}"),
            crate::domain_computation::WorthQueryProviderSessionProtocolCounters::default(),
        ),
    )
}

/// Used only for `prepare_validated_proposal`, before product publication.
pub fn transaction_commit_stop(
    error: worth_relational::facade::mvcc::TransactionCommitError,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    use crate::domain_computation::{
        WorthQueryProviderSessionCommitStop as Stop, WorthQueryProviderSessionRecoveryPosture,
    };
    use worth_query_execution_contracts::facade::{
        classify_relational_commit, commit_execution_kind, RelationalCommitStop as Route,
        WorthQueryProviderSessionDenialKind as Kind,
    };
    let pre_effect = |kind, error: worth_relational::facade::mvcc::TransactionCommitError| {
        WorthQueryProviderSessionFailure::new(
            kind,
            WorthQueryProviderSessionProtocolStage::Commit,
            error.detail(),
            Default::default(),
        )
        .with_native_preparation_error(error)
    };
    match classify_relational_commit(&error) {
        Route::Execution(denial) => match commit_execution_kind(denial) {
            Ok(_) => Stop::PreEffectDenied(pre_effect(Kind::ProviderRejected, error)),
            Err(kind) => Stop::ControlStopped(
                crate::domain_computation::WorthQueryProviderSessionCommitControlStopped::execution_preparation(kind, error.detail()),
            ),
        },
        Route::Interrupted(event) => Stop::ControlStopped(interruption_control_stopped(event)),
        Route::Deferred(deferred) => Stop::Deferred(publication_deferred(deferred)),
        Route::PreEffect(kind) => Stop::PreEffectDenied(pre_effect(kind, error)),
        Route::RecoveryRequired => Stop::Denied(
            pre_effect(Kind::ProviderRejected, error)
                .with_recovery_posture(WorthQueryProviderSessionRecoveryPosture::RecoveryRequired),
        ),
    }
}

pub(super) fn interruption_control_stopped(
    event: worth_relational::facade::mvcc::RelationalInterruptionEvent,
) -> crate::domain_computation::WorthQueryProviderSessionCommitControlStopped {
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Kind;
    let kind = match event.interruption() {
        worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled => {
            Kind::Cancelled
        }
        worth_relational::facade::mvcc::RelationalOperationInterruption::TimedOut => Kind::TimedOut,
    };
    crate::domain_computation::WorthQueryProviderSessionCommitControlStopped::new(
        kind,
        format!("{event:?}"),
    )
}

fn publication_deferred(
    deferred: worth_relational::facade::mvcc::RelationalPublicationDeferred,
) -> crate::domain_computation::WorthQueryProviderSessionCommitDeferred {
    use crate::domain_computation::WorthQueryProviderSessionCommitDeferredKind as Kind;
    crate::domain_computation::WorthQueryProviderSessionCommitDeferred::new(
        Kind::RelationalDeferred(deferred),
        format!("{deferred:?}"),
    )
}

#[cfg(test)]
mod tests;

pub(super) fn source_rebase_preparation_stop(
    cause: super::super::PreparedRebaseDenial,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    use super::super::PreparedRebaseDenial as Denial;
    use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial;
    use crate::domain_computation::{
        WorthQueryProviderSessionCommitControlStopped as Control,
        WorthQueryProviderSessionCommitStop as Stop,
        WorthQueryProviderSessionControlStopKind as Kind,
    };
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption as Interrupted;
    if let Denial::Admission(stop) = &cause {
        if let worth_relational::facade::mvcc::CompanionPreflightStop::Execution(cause) = stop {
            return relational_execution_stop(*cause, None);
        }
        if let worth_relational::facade::mvcc::CompanionPreflightStop::Interrupted(event) = stop {
            return Stop::ControlStopped(interruption_control_stopped(*event));
        }
        return Stop::Deferred(crate::domain_computation::WorthQueryProviderSessionCommitDeferred::new(
            crate::domain_computation::WorthQueryProviderSessionCommitDeferredKind::RelationalDeferred(
                worth_relational::facade::mvcc::RelationalPublicationDeferred::CompanionPreflight(*stop)),
            "source rebase selection refused before publication"));
    }
    if let Denial::Retention(StoreDenial::RequestInterruption(interruption)) = &cause {
        return Stop::ControlStopped(Control::new(
            match interruption {
                Interrupted::Cancelled => Kind::Cancelled,
                Interrupted::DeadlineExceeded => Kind::TimedOut,
            },
            "source rebase preparation interrupted",
        ));
    }
    if let Denial::Retention(StoreDenial::ExecutionRequest(stop)) = &cause {
        return relational_execution_stop((*stop).into(), None);
    }
    if let Denial::Retention(StoreDenial::Allocation(allocation)) = cause {
        use worth_execution::{
            ExecutionAllocationDenialClass as Class, MapKernelStop as Kernel,
            WorkCeilingDenial as Work,
        };
        if let Class::Request(Work::Stopped(stop @ (Kernel::Cancelled | Kernel::DeadlineElapsed))) =
            allocation.classify()
        {
            return Stop::ControlStopped(Control::new(
                if stop == Kernel::Cancelled {
                    Kind::Cancelled
                } else {
                    Kind::TimedOut
                },
                "source rebase preparation interrupted",
            ));
        }
        return Stop::PreEffectDenied(WorthQueryProviderSessionFailure::allocation_denied(
            allocation,
            "source fact rebase",
        ));
    }
    Stop::PreEffectDenied(WorthQueryProviderSessionFailure::new(
        crate::domain_computation::WorthQueryProviderSessionDenialKind::ProviderRejected,
        WorthQueryProviderSessionProtocolStage::Commit,
        format!("source rebase preparation refused before publication: {cause:?}"),
        crate::domain_computation::WorthQueryProviderSessionProtocolCounters::default(),
    ))
}

pub(super) fn readmission_allocation(
    cause: worth_execution::ExecutionAllocationDenial,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    match cause.classify() {
        worth_execution::ExecutionAllocationDenialClass::Request(stop) =>
            crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_stop(stop.into(), None),
        worth_execution::ExecutionAllocationDenialClass::Physical(_) =>
            crate::domain_computation::WorthQueryProviderSessionCommitStop::PreEffectDenied(
                crate::domain_computation::WorthQueryProviderSessionFailure::allocation_denied(
                    cause, "current publication role readmission")),
    }
}
