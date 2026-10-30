//! Descriptive custody of an original, exposed only by a freshly published report.
use super::super::WorthQueryApplicationQueryObservation;
use crate::domain_computation::primary_graph::application_output_demand::OriginalSettledProducerSource;

/// Whether this original still owns the indicated immutable evidence.
/// Absence says nothing about whether a producer ran or whether evidence expired.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInspectionRetainedEvidence {
    NotRetained,
    Retained,
}

/// The actual supplier custody carried by this original query basis.
/// Committed output authority is not a verified source-to-output connection; reuse and
/// restoration are not evidence of a newly performed producer or suppression.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInspectionSupplierEvidence {
    NotRetained,
    CommittedOutput,
    StableInputReuse,
    CheckpointRestoration,
}

/// A metadata snapshot of the original's owners, not a new evidence owner or
/// permission to materialize fields. Strong original custody has no synthetic TTL.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryInspectionEvidenceAvailability {
    projection: WorthQueryInspectionRetainedEvidence,
    supplier: WorthQueryInspectionSupplierEvidence,
    native_observation: WorthQueryInspectionRetainedEvidence,
}

impl WorthQueryInspectionEvidenceAvailability {
    pub fn projection(&self) -> WorthQueryInspectionRetainedEvidence {
        self.projection
    }
    pub fn supplier(&self) -> WorthQueryInspectionSupplierEvidence {
        self.supplier
    }
    pub fn native_observation(&self) -> WorthQueryInspectionRetainedEvidence {
        self.native_observation
    }

    pub(super) fn canonical_labels(&self) -> [&'static str; 3] {
        use WorthQueryInspectionRetainedEvidence as Retention;
        use WorthQueryInspectionSupplierEvidence as Supplier;
        let retained = |value| match value {
            Retention::Retained => "retained",
            Retention::NotRetained => "not-retained",
        };
        [
            retained(self.projection),
            match self.supplier {
                Supplier::NotRetained => "not-retained",
                Supplier::CommittedOutput => "committed-output",
                Supplier::StableInputReuse => "stable-input-reuse",
                Supplier::CheckpointRestoration => "checkpoint-restoration",
            },
            retained(self.native_observation),
        ]
    }

    /// O(1), no allocation or body traversal. The caller is a published result
    /// whose fresh session admitted the fixed reference metadata facets.
    pub(super) fn from_original(original: &WorthQueryApplicationQueryObservation) -> Self {
        use WorthQueryInspectionRetainedEvidence::{NotRetained, Retained};
        use WorthQueryInspectionSupplierEvidence as Supplier;
        let projection = if original.source.original_projection().is_some() {
            Retained
        } else {
            NotRetained
        };
        let (supplier, native_observation) = match original
            .source
            .original_settled_producer()
            .map(|original| original.source())
        {
            None => (Supplier::NotRetained, NotRetained),
            Some(OriginalSettledProducerSource::Committed { conditional, .. }) => (
                Supplier::CommittedOutput,
                if conditional.is_some() {
                    Retained
                } else {
                    NotRetained
                },
            ),
            Some(OriginalSettledProducerSource::StableReused { .. }) => {
                (Supplier::StableInputReuse, NotRetained)
            }
            Some(OriginalSettledProducerSource::Restored { .. }) => {
                (Supplier::CheckpointRestoration, NotRetained)
            }
        };
        Self {
            projection,
            supplier,
            native_observation,
        }
    }
}
