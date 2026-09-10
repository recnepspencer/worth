use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryExternalDispatchPostureKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPublishedExternalEffectPosture {
    NotDeclared,
    PendingDispatch,
    Completed,
    Acknowledged,
    Unresolved(WorthQueryPublishedExternalEffectFailure),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPublishedExternalEffectPostureKind {
    NotDeclared,
    PendingDispatch,
    Completed,
    Acknowledged,
    Unresolved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPublishedExternalEffectFailure {
    Timeout,
    Disconnect,
    LostResponse,
    DuplicatedAcknowledgement,
    PayloadRejected,
    InitialDispatchOwnerReadDenied,
    InitialDispatchAttemptAdmissionDenied,
    InitialDispatchCanonicalDerivationDenied,
    InitialDispatchTimeObservationDenied,
    UnsupportedProtocolVersion {
        produced: u32,
        posture: WorthQueryPublishedUnsupportedProtocolVersionPosture,
    },
    UnknownProviderOutcome,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPublishedUnsupportedProtocolVersionPosture {
    PredatesWindow,
    ExceedsWindow,
    Retired,
}

impl WorthQueryPublishedExternalEffectPosture {
    pub const fn kind(self) -> WorthQueryPublishedExternalEffectPostureKind {
        match self {
            Self::NotDeclared => WorthQueryPublishedExternalEffectPostureKind::NotDeclared,
            Self::PendingDispatch => WorthQueryPublishedExternalEffectPostureKind::PendingDispatch,
            Self::Completed => WorthQueryPublishedExternalEffectPostureKind::Completed,
            Self::Acknowledged => WorthQueryPublishedExternalEffectPostureKind::Acknowledged,
            Self::Unresolved(_) => WorthQueryPublishedExternalEffectPostureKind::Unresolved,
        }
    }

    pub const fn failure(self) -> Option<WorthQueryPublishedExternalEffectFailure> {
        match self {
            Self::Unresolved(failure) => Some(failure),
            _ => None,
        }
    }
}

pub(super) fn publish_external_effect(
    receipt: &WorthQueryApplicationCommitReceipt,
) -> WorthQueryPublishedExternalEffectPosture {
    match receipt.external_dispatch() {
        Some(dispatch) => match dispatch.posture().kind() {
            WorthQueryExternalDispatchPostureKind::Completed => {
                WorthQueryPublishedExternalEffectPosture::Completed
            }
            WorthQueryExternalDispatchPostureKind::Acknowledged => {
                WorthQueryPublishedExternalEffectPosture::Acknowledged
            }
            WorthQueryExternalDispatchPostureKind::Unresolved => {
                match dispatch.posture().classification() {
                    Some(classification) => WorthQueryPublishedExternalEffectPosture::Unresolved(
                        publish_failure(classification.fault()),
                    ),
                    None => WorthQueryPublishedExternalEffectPosture::Unresolved(
                        WorthQueryPublishedExternalEffectFailure::UnknownProviderOutcome,
                    ),
                }
            }
        },
        None => match receipt.external_dispatch_preparation_denial() {
            Some(denial) => WorthQueryPublishedExternalEffectPosture::Unresolved(
                publish_preparation_failure(denial),
            ),
            None if receipt.dispatch_outbox().is_some() => {
                WorthQueryPublishedExternalEffectPosture::PendingDispatch
            }
            None => WorthQueryPublishedExternalEffectPosture::NotDeclared,
        },
    }
}

const fn publish_preparation_failure(
    denial: worth_query_execution::facade::primary_graph::WorthQueryExternalDispatchPreparationDenial,
) -> WorthQueryPublishedExternalEffectFailure {
    use worth_query_execution::facade::primary_graph::WorthQueryExternalDispatchPreparationDenial as Execution;

    match denial {
        Execution::OwnerReadDenied(_) => {
            WorthQueryPublishedExternalEffectFailure::InitialDispatchOwnerReadDenied
        }
        Execution::AttemptAdmissionDenied => {
            WorthQueryPublishedExternalEffectFailure::InitialDispatchAttemptAdmissionDenied
        }
        Execution::CanonicalDerivationDenied => {
            WorthQueryPublishedExternalEffectFailure::InitialDispatchCanonicalDerivationDenied
        }
        Execution::TimeObservationDenied => {
            WorthQueryPublishedExternalEffectFailure::InitialDispatchTimeObservationDenied
        }
    }
}

const fn publish_failure(
    failure: worth_query_execution::facade::primary_graph::ExternalRailTransportFault,
) -> WorthQueryPublishedExternalEffectFailure {
    use worth_query_execution::facade::primary_graph::ExternalRailTransportFault as Execution;

    match failure {
        Execution::Timeout => WorthQueryPublishedExternalEffectFailure::Timeout,
        Execution::Disconnect => WorthQueryPublishedExternalEffectFailure::Disconnect,
        Execution::LostResponse => WorthQueryPublishedExternalEffectFailure::LostResponse,
        Execution::DuplicatedAcknowledgement => {
            WorthQueryPublishedExternalEffectFailure::DuplicatedAcknowledgement
        }
        Execution::PayloadRejected => WorthQueryPublishedExternalEffectFailure::PayloadRejected,
        Execution::UnsupportedProtocolVersion(unsupported) => {
            use worth_foundational::facade::BoundaryProtocolUnsupportedVersionPosture as Posture;
            let posture = match unsupported.posture() {
                Posture::PredatesWindow => {
                    WorthQueryPublishedUnsupportedProtocolVersionPosture::PredatesWindow
                }
                Posture::ExceedsWindow => {
                    WorthQueryPublishedUnsupportedProtocolVersionPosture::ExceedsWindow
                }
                Posture::Retired => WorthQueryPublishedUnsupportedProtocolVersionPosture::Retired,
            };
            WorthQueryPublishedExternalEffectFailure::UnsupportedProtocolVersion {
                produced: unsupported.produced().get(),
                posture,
            }
        }
        Execution::UnknownProviderOutcome => {
            WorthQueryPublishedExternalEffectFailure::UnknownProviderOutcome
        }
    }
}

#[cfg(test)]
mod tests {
    use worth_foundational::facade::{
        BoundaryProtocolCompatibilityWindow, BoundaryProtocolVersion,
    };
    use worth_query_execution::facade::primary_graph::ExternalRailTransportFault;
    use worth_query_execution::facade::primary_graph::{
        WorthQueryCommittedDispatchOutboxReadDenial, WorthQueryExternalDispatchPreparationDenial,
    };

    use super::{
        publish_failure, publish_preparation_failure, WorthQueryPublishedExternalEffectFailure,
        WorthQueryPublishedUnsupportedProtocolVersionPosture,
    };

    #[test]
    fn every_non_version_transport_fault_is_preserved_exactly() {
        let cases = [
            (
                ExternalRailTransportFault::Timeout,
                WorthQueryPublishedExternalEffectFailure::Timeout,
            ),
            (
                ExternalRailTransportFault::Disconnect,
                WorthQueryPublishedExternalEffectFailure::Disconnect,
            ),
            (
                ExternalRailTransportFault::LostResponse,
                WorthQueryPublishedExternalEffectFailure::LostResponse,
            ),
            (
                ExternalRailTransportFault::DuplicatedAcknowledgement,
                WorthQueryPublishedExternalEffectFailure::DuplicatedAcknowledgement,
            ),
            (
                ExternalRailTransportFault::PayloadRejected,
                WorthQueryPublishedExternalEffectFailure::PayloadRejected,
            ),
            (
                ExternalRailTransportFault::UnknownProviderOutcome,
                WorthQueryPublishedExternalEffectFailure::UnknownProviderOutcome,
            ),
        ];

        for (owner_fault, expected) in cases {
            assert_eq!(publish_failure(owner_fault), expected);
        }
    }

    #[test]
    fn every_initial_dispatch_preparation_denial_is_preserved_exactly() {
        let cases = [
            (
                WorthQueryExternalDispatchPreparationDenial::OwnerReadDenied(
                    WorthQueryCommittedDispatchOutboxReadDenial::RecordMismatch,
                ),
                WorthQueryPublishedExternalEffectFailure::InitialDispatchOwnerReadDenied,
            ),
            (
                WorthQueryExternalDispatchPreparationDenial::AttemptAdmissionDenied,
                WorthQueryPublishedExternalEffectFailure::InitialDispatchAttemptAdmissionDenied,
            ),
            (
                WorthQueryExternalDispatchPreparationDenial::CanonicalDerivationDenied,
                WorthQueryPublishedExternalEffectFailure::InitialDispatchCanonicalDerivationDenied,
            ),
            (
                WorthQueryExternalDispatchPreparationDenial::TimeObservationDenied,
                WorthQueryPublishedExternalEffectFailure::InitialDispatchTimeObservationDenied,
            ),
        ];
        for (denial, expected) in cases {
            assert_eq!(publish_preparation_failure(denial), expected);
        }
    }

    #[test]
    fn every_unsupported_version_posture_maps_exhaustively_and_exactly() {
        let v1 = BoundaryProtocolVersion::new(1);
        let v2 = BoundaryProtocolVersion::new(2);
        let v3 = BoundaryProtocolVersion::new(3);
        let cases = [
            (
                BoundaryProtocolCompatibilityWindow::inclusive(v2, v2)
                    .admit(v1)
                    .unwrap_err(),
                1,
                WorthQueryPublishedUnsupportedProtocolVersionPosture::PredatesWindow,
            ),
            (
                BoundaryProtocolCompatibilityWindow::inclusive(v1, v2)
                    .admit(v3)
                    .unwrap_err(),
                3,
                WorthQueryPublishedUnsupportedProtocolVersionPosture::ExceedsWindow,
            ),
            (
                BoundaryProtocolCompatibilityWindow::inclusive(v1, v2)
                    .retire_before(v2)
                    .admit(v1)
                    .unwrap_err(),
                1,
                WorthQueryPublishedUnsupportedProtocolVersionPosture::Retired,
            ),
        ];

        for (unsupported, produced, posture) in cases {
            assert_eq!(
                publish_failure(ExternalRailTransportFault::UnsupportedProtocolVersion(
                    unsupported,
                )),
                WorthQueryPublishedExternalEffectFailure::UnsupportedProtocolVersion {
                    produced,
                    posture,
                }
            );
        }
    }
}
