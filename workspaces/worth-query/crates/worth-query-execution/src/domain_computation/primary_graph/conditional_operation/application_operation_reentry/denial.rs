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
