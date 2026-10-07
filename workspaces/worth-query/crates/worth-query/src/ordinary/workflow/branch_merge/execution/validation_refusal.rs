//! A validation refusal is pre-effect and preserves its typed owner cause.
use super::*;

pub(super) fn execution_denied(
    kind: worth_query_execution::facade::primary_graph::WorthQueryProviderSessionDenialKind,
    counters: WorthQueryWorkflowCounters,
) -> WorthQueryBranchMergeOutcome {
    WorthQueryBranchMergeOutcome::Stopped(WorthQueryBranchMergeStop::from_execution_denial(
        crate::runtime::WorthQueryOrdinaryMergeFailureStage::Basis,
        Some(crate::effect_lifecycle::EffectExecutionDenialKind::RelationalExecutionDenied(kind)),
        "execution refused branch-merge validation".to_owned(),
        counters,
    ))
}

pub(super) fn control_stopped(
    kind: worth_query_execution::facade::primary_graph::WorthQueryProviderSessionControlStopKind,
    counters: WorthQueryWorkflowCounters,
) -> WorthQueryBranchMergeOutcome {
    use worth_query_execution::facade::primary_graph::WorthQueryProviderSessionControlStopKind as Kind;
    let kind = match kind {
        Kind::Cancelled => crate::effect_lifecycle::EffectExecutionControlStopKind::Cancelled,
        Kind::TimedOut => crate::effect_lifecycle::EffectExecutionControlStopKind::TimedOut,
    };
    WorthQueryBranchMergeOutcome::ControlStopped(WorthQueryBranchMergeControlStopped::new(
        kind,
        "execution stopped branch-merge validation".to_owned(),
        counters,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_refusal_survives_the_merge_validation_mapping_seam() {
        use worth_relational::facade::transactions::{
            CommitExecutionDenialKind, TransactionCommitError,
        };
        // Production merge validation currently only observes branch bases.
        // Its backend error type can carry execution stops; exercise that seam
        // with a real commit-owner refusal, without threading a production lease.
        let TransactionCommitError::Execution { denial, .. } =
            crate::relational_execution_refusal::work_exhausted()
        else {
            panic!("real execution refusal");
        };
        let kind = match denial.kind {
            CommitExecutionDenialKind::Cause(cause) => {
                worth_query_execution::facade::primary_graph::relational_execution_kind(
                    cause,
                    denial.partition_identity,
                )
                .unwrap()
            }
        };
        assert_eq!(kind, worth_query_execution::facade::primary_graph::WorthQueryProviderSessionDenialKind::ExecutionResource {
            denial: worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial::WorkExhausted,
            partition_identity: Some(1), policy_ancestor: None,
        });
        let classified =
            crate::runtime::ordinary_workflow_authority::classify_merge_validation_kind(
                crate::memory_workspace::WorthQueryWorkspaceErrorKind::ExecutionDenied(kind),
            );
        let WorthQueryMergeAuthorityValidationError::ExecutionDenied(kind) = classified else {
            panic!("validation folded refusal");
        };
        let WorthQueryBranchMergeOutcome::Stopped(observed) =
            execution_denied(kind, WorthQueryWorkflowCounters::context_checked())
        else {
            panic!("caller lost refusal");
        };
        assert_eq!(
            observed.effect_kind(),
            Some(
                crate::effect_lifecycle::EffectExecutionDenialKind::RelationalExecutionDenied(kind)
            )
        );
    }
    #[test]
    fn validation_controls_remain_control_stops() {
        use worth_query_execution::facade::primary_graph::WorthQueryProviderSessionControlStopKind as Kind;
        for (kind, expected) in [
            (
                Kind::Cancelled,
                crate::effect_lifecycle::EffectExecutionControlStopKind::Cancelled,
            ),
            (
                Kind::TimedOut,
                crate::effect_lifecycle::EffectExecutionControlStopKind::TimedOut,
            ),
        ] {
            let WorthQueryBranchMergeOutcome::ControlStopped(observed) =
                control_stopped(kind, WorthQueryWorkflowCounters::context_checked())
            else {
                panic!("control folded");
            };
            assert_eq!(observed.kind(), expected);
        }
    }
}
