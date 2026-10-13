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

/// Inspection describes truth; it cannot carry a plan or claim fresh work.
pub(super) enum ReadyInspection<'basis> {
    Current(Arc<WorthQueryOutputDemandSettlement>),
    PendingExact(SelectedPendingConsumedOutput<'basis>),
    PendingUnresolved,
}
impl<'basis, 'owner, Schema: ApplicationSchema> From<ReadyInspection<'basis>>
    for RequiredCueProgress<'basis, 'owner, Schema>
{
    fn from(value: ReadyInspection<'basis>) -> Self {
        match value {
            ReadyInspection::Current(value) => Self::Current(value),
            ReadyInspection::PendingExact(value) => Self::PendingExact(value),
            ReadyInspection::PendingUnresolved => Self::PendingUnresolved,
        }
    }
}
/// The frozen door has no execution, scheduling or claim outcome.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) enum FrozenCurrentInspection<
    'basis,
> {
    Current(Arc<WorthQueryOutputDemandSettlement>),
    PendingExact(SelectedPendingConsumedOutput<'basis>),
    NotCurrent,
}
impl<'basis> From<ReadyInspection<'basis>> for FrozenCurrentInspection<'basis> {
    fn from(value: ReadyInspection<'basis>) -> Self {
        match value {
            ReadyInspection::Current(value) => Self::Current(value),
            ReadyInspection::PendingExact(value) => Self::PendingExact(value),
            ReadyInspection::PendingUnresolved => Self::NotCurrent,
        }
    }
}

/// Inputs borrowed for native currentness inspection, without a preparation phase.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct FrozenInspection<'basis, 'call, Schema: ApplicationSchema> {
    pub execution: &'call worth_execution::ExecutionRequest<'call, 'call>,
    pub runtime: &'call crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub principal: &'call worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
    pub request: &'call worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    pub shared: &'basis crate::domain_computation::primary_graph::SharedSelectedProductOperation<'call, Schema>,
    pub positioned: &'basis worth_relational::facade::runtime::PositionedRelationalSnapshot,
    pub selected: &'call crate::domain_computation::primary_graph::application_output_demand::SelectedReadyReadmission,
    pub candidate: Option<&'basis crate::domain_computation::primary_graph::output_lineage::AcceptedCurrentCandidate>,
    pub contacts: usize,
}
