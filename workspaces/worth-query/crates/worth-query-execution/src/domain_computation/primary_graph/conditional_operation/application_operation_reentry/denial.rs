pub(in crate::domain_computation::primary_graph::conditional_operation) enum WorthQueryTemporalReentryDenial
{
    Retryable(super::super::WorthQueryConditionalReentryFailure),
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    RetentionCapacityExhausted,
    ControlStopped(super::WorthQueryTemporalControlStop),
    Terminal(super::WorthQueryTemporalAdmissionTerminalFailure),
}

impl WorthQueryTemporalReentryDenial {
    pub(in crate::domain_computation::primary_graph::conditional_operation) fn from_principal(
        denial: super::super::super::WorthQueryPrincipalResolutionDenial,
    ) -> Self {
        match denial.kind() {
            super::super::super::WorthQueryPrincipalResolutionDenialKind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => Self::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            super::super::super::WorthQueryPrincipalResolutionDenialKind::RetentionCapacityExhausted => {
                Self::RetentionCapacityExhausted
            }
            super::super::super::WorthQueryPrincipalResolutionDenialKind::Cancelled => {
                Self::ControlStopped(super::WorthQueryTemporalControlStop::Cancelled)
            }
            super::super::super::WorthQueryPrincipalResolutionDenialKind::DeadlineExceeded => {
                Self::ControlStopped(super::WorthQueryTemporalControlStop::TimedOut)
            }
            kind => Self::Terminal(super::WorthQueryTemporalAdmissionTerminalFailure::Principal(
                kind,
            )),
        }
    }

    pub(in crate::domain_computation::primary_graph::conditional_operation) fn from_entity(
        denial: super::super::super::WorthQueryEntityResolutionDenial,
    ) -> Self {
        match denial.kind() {
            super::super::super::WorthQueryEntityResolutionDenialKind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => Self::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            super::super::super::WorthQueryEntityResolutionDenialKind::RetentionCapacityExhausted => {
                Self::RetentionCapacityExhausted
            }
            super::super::super::WorthQueryEntityResolutionDenialKind::Cancelled => {
                Self::ControlStopped(super::WorthQueryTemporalControlStop::Cancelled)
            }
            super::super::super::WorthQueryEntityResolutionDenialKind::DeadlineExceeded => {
                Self::ControlStopped(super::WorthQueryTemporalControlStop::TimedOut)
            }
            kind => Self::Terminal(super::WorthQueryTemporalAdmissionTerminalFailure::Entity(kind)),
        }
    }

    pub(in crate::domain_computation::primary_graph::conditional_operation) fn from_authorization(
        denial: super::super::super::WorthQueryOperationAuthorizationDenial,
    ) -> Self {
        match denial.kind() {
            super::super::super::WorthQueryOperationAuthorizationDenialKind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => Self::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            super::super::super::WorthQueryOperationAuthorizationDenialKind::RetentionCapacityExhausted => {
                Self::RetentionCapacityExhausted
            }
            super::super::super::WorthQueryOperationAuthorizationDenialKind::Cancelled => {
                Self::ControlStopped(super::WorthQueryTemporalControlStop::Cancelled)
            }
            super::super::super::WorthQueryOperationAuthorizationDenialKind::DeadlineExceeded => {
                Self::ControlStopped(super::WorthQueryTemporalControlStop::TimedOut)
            }
            kind => Self::Terminal(
                super::WorthQueryTemporalAdmissionTerminalFailure::Authorization(kind),
            ),
        }
    }

    pub(in crate::domain_computation::primary_graph::conditional_operation) fn from_projection(
        denial: super::super::super::WorthQueryOperationProjectionDenial,
    ) -> Self {
        use super::super::super::{
            WorthQueryInvariantProjectionDenialKind, WorthQueryOperationAuthorizationDenialKind,
            WorthQueryOperationProjectionDenialKind,
        };
        let kind = denial.kind();
        let maximum_active_snapshots = match kind {
            WorthQueryOperationProjectionDenialKind::Authorization(
                WorthQueryOperationAuthorizationDenialKind::ActiveSnapshotCapacityExhausted {
                    maximum_active_snapshots,
                },
            )
            | WorthQueryOperationProjectionDenialKind::InvariantAdmission(
                WorthQueryInvariantProjectionDenialKind::ActiveSnapshotCapacityExhausted {
                    maximum_active_snapshots,
                },
            ) => Some(maximum_active_snapshots),
            _ => None,
        };
        let retention_capacity_exhausted = matches!(
            denial.kind(),
            WorthQueryOperationProjectionDenialKind::Authorization(
                WorthQueryOperationAuthorizationDenialKind::RetentionCapacityExhausted
            ) | WorthQueryOperationProjectionDenialKind::InvariantAdmission(
                WorthQueryInvariantProjectionDenialKind::RetentionCapacityExhausted
            )
        );
        if retention_capacity_exhausted {
            return Self::RetentionCapacityExhausted;
        }
        if let Some(maximum_active_snapshots) = maximum_active_snapshots {
            return Self::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            };
        }
        match kind {
            WorthQueryOperationProjectionDenialKind::Authorization(
                WorthQueryOperationAuthorizationDenialKind::Cancelled,
            ) => Self::ControlStopped(super::WorthQueryTemporalControlStop::Cancelled),
            WorthQueryOperationProjectionDenialKind::Authorization(
                WorthQueryOperationAuthorizationDenialKind::DeadlineExceeded,
            ) => Self::ControlStopped(super::WorthQueryTemporalControlStop::TimedOut),
            kind => {
                Self::Terminal(super::WorthQueryTemporalAdmissionTerminalFailure::Projection(kind))
            }
        }
    }

    pub(in crate::domain_computation::primary_graph::conditional_operation) fn from_invariant(
        denial: super::super::super::WorthQueryInvariantProjectionDenial,
    ) -> Self {
        match denial.kind() {
            super::super::super::WorthQueryInvariantProjectionDenialKind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => Self::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            super::super::super::WorthQueryInvariantProjectionDenialKind::RetentionCapacityExhausted => {
                Self::RetentionCapacityExhausted
            }
            kind => Self::Terminal(super::WorthQueryTemporalAdmissionTerminalFailure::Invariant(
                kind,
            )),
        }
    }
}

impl From<super::super::WorthQueryConditionalReentryFailure> for WorthQueryTemporalReentryDenial {
    fn from(cause: super::super::WorthQueryConditionalReentryFailure) -> Self {
        Self::Retryable(cause)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_capacity_survives_reentry_mapping() {
        let denial = crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial::new(
            crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots: 23,
            },
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
