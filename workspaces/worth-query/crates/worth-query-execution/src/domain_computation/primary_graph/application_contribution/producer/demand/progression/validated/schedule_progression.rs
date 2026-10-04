use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::primary_graph::application_contribution::producer::demand::MatchedRequiredPredecessors;
use crate::domain_computation::primary_graph::product_operation::SharedSelectedProductOperation;
use crate::domain_computation::primary_graph::WorthQueryObservedSource;

use super::super::*;

/// Ordinary advancement yields after scheduling. A required wave continues
/// only with the same owner-issued Product it used for disclosure.
pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression)
enum ScheduleProgression<'selection, 'runtime, Schema>
where
    Schema: ApplicationSchema,
{
    ReturnPending,
    ContinueScheduled {
        shared: &'selection SharedSelectedProductOperation<'runtime, Schema>,
        matched_predecessors: Option<MatchedRequiredPredecessors<'selection>>,
    },
}

impl<Schema> ScheduleProgression<'_, '_, Schema>
where
    Schema: ApplicationSchema,
{
    pub(super) fn is_selected(&self) -> bool {
        matches!(self, Self::ContinueScheduled { .. })
    }

    pub(super) fn source_matches<Family>(
        &self,
        demand: &WorthQueryAdmittedOutputDemand<Schema, Family>,
        source: &WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>,
    ) -> bool
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        match self {
            Self::ReturnPending => demand.matches_observed_source(source),
            // The selected source is validated once before registry.begin,
            // while refusal can still leave the exact Interest retryable.
            Self::ContinueScheduled { .. } => true,
        }
    }
}
