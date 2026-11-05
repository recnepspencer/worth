//! Preserve Native cold-recovery causes at the Query installation boundary.
use super::*;
pub(in crate::domain_computation::primary_graph::bootstrap) fn map_checkpoint_recovery_stop(
    stop: worth_relational::facade::durability::RelationalRecoveryStop,
) -> WorthQueryPrimaryGraphInstallationDenial {
    use worth_relational::facade::durability::RelationalRecoveryStop;
    match stop {
        RelationalRecoveryStop::Content(cause) => primary_graph_denial(
            WorthQueryPrimaryGraphInstallationDenialKind::ContentAdmissionRejected,
            "native checkpoint content refused",
        )
        .with_content_cause(cause),
        RelationalRecoveryStop::Durability(error) => {
            let denial = primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
                format!("native checkpoint recovery denied: {error:?}"),
            );
            if error.storage_unavailable.is_some_and(|e|matches!(e.cause,worth_relational::facade::storage::RelationalStorageUnavailableCause::CorruptCurrentOwnership{..})) {
                denial.with_native_recovery_cause(RelationalRecoveryStop::Durability(error))
            } else {denial}
        }
        RelationalRecoveryStop::Allocation(cause) => primary_graph_denial(
            WorthQueryPrimaryGraphInstallationDenialKind::NativeAllocation {
                kind: cause.classify(),
                requested_payload_bytes: cause.requested_payload_bytes(),
            },
            "native checkpoint recovery allocation refused",
        ),
        RelationalRecoveryStop::Request(cause) => {
            let translated =
                crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_kind(
                    cause, None,
                );
            let kind = match translated {
                Ok(kind) => WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied { kind },
                Err(kind) => {
                    WorthQueryPrimaryGraphInstallationDenialKind::ExecutionControlStopped { kind }
                }
            };
            primary_graph_denial(
                kind,
                format!("native checkpoint recovery request stopped: {cause:?}"),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checkpoint_recovery_request_arm_keeps_control_and_memory_by_value() {
        use crate::domain_computation::primary_graph::application_contribution::{
            WorthQueryManagedComputationResourceDenial as Resource,
            WorthQueryMemoryLimitLevel as Level,
        };
        use crate::domain_computation::{
            WorthQueryProviderSessionControlStopKind as Control,
            WorthQueryProviderSessionDenialKind as Denial,
        };
        use worth_relational::facade::{
            durability::RelationalRecoveryStop as Stop,
            transactions::RelationalExecutionDenialCause as Cause,
        };
        use WorthQueryPrimaryGraphInstallationDenialKind as Kind;
        for (cause, kind) in [
            (
                Cause::Cancelled,
                Kind::ExecutionControlStopped {
                    kind: Control::Cancelled,
                },
            ),
            (
                Cause::DeadlineElapsed,
                Kind::ExecutionControlStopped {
                    kind: Control::TimedOut,
                },
            ),
            (
                Cause::PolicyMemoryExhausted {
                    requested: 4096,
                    admitted: 4095,
                    ancestor: 2,
                },
                Kind::ExecutionDenied {
                    kind: Denial::ExecutionResource {
                        denial: Resource::MemoryLimit {
                            requested: 4096,
                            admitted: 4095,
                            level: Level::Policy,
                        },
                        partition_identity: None,
                        policy_ancestor: Some(2),
                    },
                },
            ),
        ] {
            assert_eq!(
                map_checkpoint_recovery_stop(Stop::Request(cause)).kind(),
                kind
            );
        }
    }
    #[test]
    fn checkpoint_recovery_allocation_keeps_physical_kind_and_checked_quote() {
        use worth_execution::{
            ExecutionAllocationDenialClass as AllocationKind, ExecutionAllocationPolicy,
            ExecutionByteBuffer,
        };
        use worth_relational::facade::durability::RelationalRecoveryStop;
        use WorthQueryPrimaryGraphInstallationDenialKind as Kind;

        // Real owner failures: unrepresentable layout has no checked quote;
        // an incomplete, representable backing retains its exact quote.
        let layout =
            ExecutionByteBuffer::allocate(usize::MAX, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap_err();
        let incomplete =
            ExecutionByteBuffer::allocate(17, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap()
                .seal()
                .unwrap_err();
        for (cause, kind, quote) in [
            (
                layout,
                AllocationKind::Physical(
                    worth_execution::ExecutionPhysicalAllocationDenialKind::Layout,
                ),
                None,
            ),
            (
                incomplete,
                AllocationKind::Physical(
                    worth_execution::ExecutionPhysicalAllocationDenialKind::IncompleteSeal,
                ),
                Some(17),
            ),
        ] {
            assert_eq!(
                map_checkpoint_recovery_stop(RelationalRecoveryStop::Allocation(cause)).kind(),
                Kind::NativeAllocation {
                    kind,
                    requested_payload_bytes: quote,
                }
            );
        }
    }
    #[test]
    fn checkpoint_recovery_durability_arm_keeps_the_existing_denial_by_value() {
        use worth_relational::facade::durability::{
            DurabilityError, RecoveryFailureClass, RelationalRecoveryStop,
        };
        let error = DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            "invalid checkpoint evidence",
        );
        let expected = primary_graph_denial(
            WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
            format!("native checkpoint recovery denied: {error:?}"),
        );
        assert_eq!(
            map_checkpoint_recovery_stop(RelationalRecoveryStop::Durability(error)),
            expected
        );
    }
}
