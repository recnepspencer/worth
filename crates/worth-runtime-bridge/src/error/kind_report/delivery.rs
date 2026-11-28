use super::{BridgeErrorKind, *};
use worth_execution::report::Reportable;
use worth_execution::report::{DenialClass::*, Disposition::*};

impl BridgeErrorKind for BridgeDeliveryErrorKind {
    fn report(&self) -> (DenialClass, Disposition, &'static str) {
        match self {
            Self::ExecutionDenied(denial) => (
                denial.denial_class(),
                denial.disposition(),
                "bridge.delivery.execution_denied",
            ),
            Self::SourceContractMismatch => (
                Invariant,
                Escalate,
                "bridge.delivery.source_contract_mismatch",
            ),
            Self::InvalidWideningAdmission => (
                InvalidInput,
                CorrectInput,
                "bridge.delivery.invalid_widening_admission",
            ),
            Self::BulkDeliveryRejected => (
                ExternalEffect,
                Escalate,
                "bridge.delivery.bulk_delivery_rejected",
            ),
            Self::HistoricalPolicyRejected => (
                Authorization,
                Reauthorize,
                "bridge.delivery.historical_policy_rejected",
            ),
            Self::HistoricalTruthViewUnavailable(_) => (
                Unavailable,
                Escalate,
                "bridge.delivery.historical_truth_view_unavailable",
            ),
            Self::MissingBranchHeadSource => (
                Unavailable,
                Escalate,
                "bridge.delivery.missing_branch_head_source",
            ),
            Self::HistoricalBranchMismatch => (
                Conflict,
                Final,
                "bridge.delivery.historical_branch_mismatch",
            ),
            Self::HistoricalCommitMismatch => (
                Conflict,
                Final,
                "bridge.delivery.historical_commit_mismatch",
            ),
            Self::HistoricalSelectorMissingCommit => (
                InvalidInput,
                CorrectInput,
                "bridge.delivery.historical_selector_missing_commit",
            ),
            Self::SnapshotAcquisitionFailure(_) => (
                Unavailable,
                Escalate,
                "bridge.delivery.snapshot_acquisition_failure",
            ),
            Self::SnapshotReadFailure(_) => (
                ExternalEffect,
                Escalate,
                "bridge.delivery.snapshot_read_failure",
            ),
            Self::SnapshotReadContractViolation(_) => (
                Invariant,
                Escalate,
                "bridge.delivery.snapshot_read_contract_violation",
            ),
            Self::SnapshotIdentityMismatch => (
                Invariant,
                Escalate,
                "bridge.delivery.snapshot_identity_mismatch",
            ),
            Self::SignalSinkRejection(_) => (
                ExternalEffect,
                Escalate,
                "bridge.delivery.signal_sink_rejection",
            ),
        }
    }

    fn summary(&self) -> &'static str {
        "Bridge could not deliver the request."
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ExecutionDenied(error) => Some(error),
            Self::HistoricalTruthViewUnavailable(error)
            | Self::SnapshotAcquisitionFailure(error) => Some(error),
            Self::SnapshotReadFailure(error) | Self::SnapshotReadContractViolation(error) => {
                Some(error)
            }
            Self::SignalSinkRejection(error) => Some(error),
            _ => None,
        }
    }
}

impl BridgeErrorKind for crate::adapter::RelationalBridgeSourceErrorTag {
    fn report(&self) -> (DenialClass, Disposition, &'static str) {
        match self {
            Self::ExternalSourceFailure => (
                ExternalEffect,
                Escalate,
                "bridge.source.external_source_failure",
            ),
            Self::Binding(_) => (Authorization, Reauthorize, "bridge.source.binding"),
            Self::CommitSelection(_) => (Unavailable, Escalate, "bridge.source.commit_selection"),
            Self::PinnedCommit(_) => (Unavailable, Escalate, "bridge.source.pinned_commit"),
            Self::ObservationRead(_) => (Unavailable, Escalate, "bridge.source.observation_read"),
            Self::ExecutionDenied(denial) => (
                denial.denial_class(),
                denial.disposition(),
                "bridge.source.execution_denied",
            ),
            Self::PublicationDenied(_) => (Invariant, Escalate, "bridge.source.publication_denied"),
            Self::PublicationDeferred(_) => {
                (Unavailable, Escalate, "bridge.source.publication_deferred")
            }
            Self::PublicationStale(_) => (Conflict, Final, "bridge.source.publication_stale"),
            Self::PublicationRebindRequired(_) => {
                (Conflict, Final, "bridge.source.publication_rebind_required")
            }
        }
    }

    fn summary(&self) -> &'static str {
        "Bridge could not acquire the source."
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ExecutionDenied(error) => Some(error),
            Self::PublicationDenied(error) => Some(error),
            _ => None,
        }
    }
}
