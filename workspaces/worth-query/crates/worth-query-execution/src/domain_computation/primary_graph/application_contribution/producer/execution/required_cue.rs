//! Exact per-cue result for a required-work wave on one selected Product.

use std::sync::Arc;
use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::primary_graph::{
    application_output_demand::WorthQueryOutputDemandSettlement,
    invariant_projection::SelectedPendingConsumedOutput,
};

/// The exact Ready cue may be certified or may publish a real required
/// successor. Fresh carries its move-only typed demand/interest custody.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) enum RequiredCueProgress<
    'basis,
    'owner,
    Schema,
> where
    Schema: ApplicationSchema,
{
    Current(Arc<WorthQueryOutputDemandSettlement>),
    PendingExact(SelectedPendingConsumedOutput<'basis>),
    PendingUnresolved,
    Fresh(
        super::prepared_plan::OwnerPlan<
            'owner,
            super::super::demand::RequiredFreshProgress<Schema>,
            super::WorthQueryOutputDemandDenial,
        >,
    ),
}
