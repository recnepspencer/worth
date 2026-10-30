//! Managed index contracts exposed through the primary-graph audience.
pub use crate::domain_computation::primary_graph::{
    WorthQueryManagedIndex, WorthQueryManagedIndexCandidateWriter, WorthQueryManagedIndexControl,
    WorthQueryManagedIndexDenial, WorthQueryManagedIndexDenialCause,
    WorthQueryManagedIndexInvariantDenial, WorthQueryManagedIndexLifecycle,
    WorthQueryManagedIndexNodeId, WorthQueryManagedIndexNodeRegion,
    WorthQueryManagedIndexNodeWriter, WorthQueryManagedIndexNodes,
    WorthQueryManagedIndexProbeValue, WorthQueryManagedIndexProvider,
    WorthQueryManagedIndexProviderFailure, WorthQueryManagedIndexProviderStop,
    WorthQueryManagedIndexRefusal, WorthQueryManagedIndexSourceChange,
    WorthQueryManagedIndexSourceRow, WorthQueryManagedIndexSourceRows,
    WorthQueryManagedIndexSourceSlot, WorthQueryManagedIndexStop, WorthQueryManagedIndexWorkDenial,
};
