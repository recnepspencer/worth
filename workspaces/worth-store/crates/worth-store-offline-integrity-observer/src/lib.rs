#![forbid(unsafe_code)]
//! Independent, bounded C.9 physical-family observation and disagreement.

mod comparison;
mod integrity_observation;

pub use comparison::{
    compare_integrity_observations, PhysicalIntegrityComparison,
    PhysicalIntegrityComparisonCounters, PhysicalIntegrityComparisonDenial,
    PhysicalIntegrityComparisonLimits, PhysicalIntegrityComparisonLimitsDenial,
};

pub use integrity_observation::{
    emit_offline_integrity_report, encode_offline_integrity_report, observe_store,
    OfflineArtifactDuplicateEvidence, OfflineArtifactFamily, OfflineArtifactObservation,
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
