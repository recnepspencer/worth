//! Conditional admission preserves request stops and ordinary owner refusals.
pub(in crate::domain_computation::primary_graph::conditional_operation) enum WorthQueryTemporalReentryDenial
{
    Retryable(super::super::WorthQueryConditionalReentryFailure),
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    RetentionCapacityExhausted,
    ControlStopped(super::WorthQueryTemporalControlStop),
    Terminal(super::WorthQueryTemporalAdmissionTerminalFailure),
    ExecutionStopped(crate::domain_computation::WorthQueryProviderSessionDenialKind),
}
mod authorization;
mod entity;
mod invariant;
mod principal;
mod projection;
mod read_stop;
mod retention;
impl From<super::super::WorthQueryConditionalReentryFailure> for WorthQueryTemporalReentryDenial {
    fn from(cause: super::super::WorthQueryConditionalReentryFailure) -> Self {
        Self::Retryable(cause)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;

    #[test]
    fn workflow_actor_native_cause_survives_reentry_stop_and_resource_mapping() {
        use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial as Authorization;
        use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial as Attempt;
        use worth_relational::facade::transactions::RelationalExecutionDenialCause as Native;
        for cause in [
            Native::Cancelled,
            Native::DeclaredMemoryExhausted {
                requested: 37,
                admitted: 0,
            },
        ] {
            let denial = Authorization::workflow_actor_native(Attempt::relational_read_stopped(
                cause,
                "actor source",
            ));
            let expected = denial.clone();
            let retained = match WorthQueryTemporalReentryDenial::from_authorization(denial) {
                WorthQueryTemporalReentryDenial::ControlStopped(
                    super::super::WorthQueryTemporalControlStop::Authorization(denial),
                ) if cause == Native::Cancelled => denial,
                WorthQueryTemporalReentryDenial::Terminal(
                    super::super::WorthQueryTemporalAdmissionTerminalFailure::Authorization(denial),
                ) if cause != Native::Cancelled => denial,
                _ => panic!("actor native denial lost its original stop/refusal route"),
            };
            assert_eq!(retained, expected);
            assert_eq!(
                retained
                    .workflow_actor_source_denial()
                    .unwrap()
                    .relational_execution_cause(),
                Some(cause)
            );
        }
    }

    #[test]
    fn owner_capacity_survives_reentry_mapping() {
        let denial = crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial::new(
            crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind::Refused(ResolutionRefusal::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots: 23,
            }),
            "temporal scope",
        );
        assert!(matches!(
            WorthQueryTemporalReentryDenial::from_entity(denial),
            WorthQueryTemporalReentryDenial::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots: 23
            }
        ));
    }
}
