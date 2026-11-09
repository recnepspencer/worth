//! Managed index contracts exposed through the primary-graph audience.
pub use crate::domain_computation::primary_graph::{
    WorthQueryManagedIndex, WorthQueryManagedIndexCandidateWriter,
    WorthQueryManagedIndexContentReadDenial, WorthQueryManagedIndexControl,
    WorthQueryManagedIndexDenial, WorthQueryManagedIndexDenialCause,
    WorthQueryManagedIndexInvariantDenial, WorthQueryManagedIndexLifecycle,
    WorthQueryManagedIndexNodeId, WorthQueryManagedIndexNodeRegion,
    WorthQueryManagedIndexNodeWriter, WorthQueryManagedIndexNodes,
    WorthQueryManagedIndexPredicateProvider, WorthQueryManagedIndexPredicateSummary,
    WorthQueryManagedIndexProbeValue, WorthQueryManagedIndexProvider,
    WorthQueryManagedIndexProviderFailure, WorthQueryManagedIndexProviderStop,
    WorthQueryManagedIndexRefinementProvider, WorthQueryManagedIndexRefusal,
    WorthQueryManagedIndexSourceChange, WorthQueryManagedIndexSourceRow,
    WorthQueryManagedIndexSourceRows, WorthQueryManagedIndexSourceSlot, WorthQueryManagedIndexStop,
    WorthQueryManagedIndexSummarySources, WorthQueryManagedIndexTextReadDenial,
    WorthQueryManagedIndexWatch, WorthQueryManagedIndexWatchOutcome,
    WorthQueryManagedIndexWatchUpdate, WorthQueryManagedIndexWorkDenial,
};

/// Original Native Source preparation and walk refusals; these grant no authority.
pub use worth_relational::facade::runtime::{
    DerivedRelationReadDenial, ManagedIndexSourcePreparationDenial, StructuralReadError,
};
