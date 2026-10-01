#![forbid(unsafe_code)]
//! Independent, bounded C.9 physical-family observation and disagreement.

mod comparison;
mod integrity_observation;

pub use comparison::{
    compare_integrity_observations, compare_selected_integrity_observations,
    encode_offline_selected_integrity_observation, PhysicalIntegrityComparison,
    PhysicalIntegrityComparisonCounters, PhysicalIntegrityComparisonDenial,
    PhysicalIntegrityComparisonLimits, PhysicalIntegrityComparisonLimitsDenial,
};

pub use integrity_observation::{
    emit_offline_integrity_report, emit_offline_selected_integrity_report,
    encode_offline_integrity_report, observe_store,
    OfflineArtifactDuplicateEvidence, OfflineArtifactFamily, OfflineArtifactObservation,
    OfflineBlobReclaimSourceKind,
    OfflineIndeterminatePhysicalReason, OfflineIntegrityObservationCounters,
    OfflineIntegrityObservationDenial, OfflineIntegrityObservationLimits,
    OfflineIntegrityObservationLimitsDenial, OfflineIntegrityObservationRequest,
    OfflineIntegrityObservationRequestDenial, OfflineIntegrityOutcome,
    OfflineIntegrityProtocolContext, OfflineIntegrityProtocolContextDenial, OfflineIntegrityReport,
    OfflineIntegrityReportBoundaryDenial, OfflineIntegrityReportCompleteness,
    OfflineIntegrityReportDestination, OfflineIntegrityReportDestinationDenial,
    OfflineIntegrityReportEmissionDenial, OfflineIntegrityReportWireDenial,
    OfflineIntegrityRootProtocolDeclarations, OfflinePhysicalBlastRadius,
    OfflinePhysicalDamageCause, OfflinePhysicalDamageLocalization, OfflinePhysicalFormatField,
    OfflineUnknownPhysicalReason, OfflineUnsupportedPhysicalVersion, OfflineUnsupportedVersionAxis,
    OFFLINE_INTEGRITY_ROOT_PROTOCOL_DECLARATIONS, OFFLINE_OBSERVER_ROLE_IDENTITY,
    PHYSICAL_INTEGRITY_OBSERVATION_COMPATIBILITY, PHYSICAL_INTEGRITY_OBSERVATION_PROTOCOL_IDENTITY,
    PHYSICAL_INTEGRITY_OBSERVATION_PROTOCOL_VERSION,
};
