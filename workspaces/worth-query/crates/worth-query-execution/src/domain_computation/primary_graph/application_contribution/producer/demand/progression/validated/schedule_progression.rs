use crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication;
use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::primary_graph::application_contribution::producer::demand::MatchedRequiredPredecessors;
use crate::domain_computation::primary_graph::product_operation::SharedSelectedProductOperation;

use super::super::*;

/// What one pass over a row's own stages reached.
pub(super) enum OwnStages<Query, Value> {
    /// Settled, or waiting on something outside this call.
    Answer(WorthQueryOutputDemandAdvance),
    /// This call published the row's checkpoint or moved it a stage.
    Checkpoint(CheckpointProgress),
    /// This call replaced the demand's row with one admitted under the
    /// disclosed source.
    Refreshed(super::super::super::disclosure::ValidatedOutputDisclosure<Query, Value>),
}

/// Both entries drive a row's own stages in one call. A required wave does
/// so only with the same owner-issued Product it used for disclosure.
pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression)
enum ScheduleProgression<'selection, 'runtime, Schema>
where
    Schema: ApplicationSchema,
{
    Ordinary,
    Selected {
        shared: &'selection SharedSelectedProductOperation<'runtime, Schema>,
        matched_predecessors: Option<MatchedRequiredPredecessors<'selection>>,
        readiness: super::super::required_wave::performed::FreshReadiness,
    },
}

impl<Schema> ScheduleProgression<'_, '_, Schema>
where
    Schema: ApplicationSchema,
{
    pub(super) fn is_selected(&self) -> bool {
        matches!(self, Self::Selected { .. })
    }
}

/// Only a performed publication minted during this pass can permit its own reobservation.
pub(super) enum CheckpointProgress {
    Advanced,
    Published(OwnPublication),
}
pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression)
struct OwnPublication(pub(crate) WorthQueryCommittedProductPublication);
