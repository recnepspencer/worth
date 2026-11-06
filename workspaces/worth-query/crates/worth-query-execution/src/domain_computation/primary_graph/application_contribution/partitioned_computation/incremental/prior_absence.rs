//! Why a record has no reusable computation, carried from the dropping site.

use super::SealedComputationRun;
use crate::domain_computation::primary_graph::computation_retention::prior_absence::PriorAbsence;

/// Completion must state an absence explicitly; publication cannot infer it.
pub enum SealedComputationRetention {
    Produced(SealedComputationRun),
    Absent(PriorAbsence),
}

/// A run writes its result where it completes or drops its state.
pub enum CompletedComputationRetention {
    Produced(super::retained::CompletedComputationRun),
    Absent(PriorAbsence),
}
