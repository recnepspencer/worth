//! One selected Product and native image for an exact required-output wave.

use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use std::sync::Arc;

use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;
use worth_relational::facade::runtime::SnapshotPositionDenial;

use crate::basis::WorthQueryProductBranch;
use crate::domain_computation::primary_graph::application_output_demand::PendingUpstream;
use crate::domain_computation::primary_graph::application_output_demand::SelectedReadyReadmission;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandInterest;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandKey;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandSettlement;
use crate::domain_computation::primary_graph::output_lineage::RequiredSettlementStop;
use crate::domain_computation::primary_graph::product_operation::SharedSelectedProductOperation;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

use super::super::super::execution::ProducerExecutionStop;
use super::super::super::execution::RequiredCueProgress;
use super::super::super::registry::InstalledProducerProvider;
use super::super::RequiredFreshProgress;
use super::WorthQueryOutputDemandDenial;
use super::WorthQueryOutputDemandDenialKind;

pub(in crate::domain_computation::primary_graph) use performed::CurrentBasis;
pub(in crate::domain_computation::primary_graph) use performed::CurrentProof;
mod caller;
mod requested;
pub(super) use requested::advance_requested_output;
mod cycles;
mod drive;
mod frontier;
mod member_key;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) mod performed;
mod queued;
mod resolved;
mod selection;
pub(in crate::domain_computation::primary_graph) use resolved::MatchedRequiredPredecessors;
pub(in crate::domain_computation::primary_graph) use resolved::ReboundConsumedOutput;
use resolved::ResolvedOnWave;
pub(in crate::domain_computation::primary_graph) use resolved::ResolvedRequiredPredecessors;
use selection::select_required_wave;
use selection::RequiredWaveSelection;

pub(super) use caller::advance_required_before_caller;

/// A requested upstream certifies only itself, never its initial consumer.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum RequiredWaveTarget {
    Caller,
    Requested,
}

/// A cue is only scheduling custody. A Current result comes from the exact
/// accepted row, its installed producer, and the selected Product/native proof.
enum RequiredWaveStep<'owner, Schema: ApplicationSchema> {
    Current(performed::CurrentProof<Arc<WorthQueryOutputDemandSettlement>>),
    Upstream(SelectedReadyReadmission),
    /// The upstream row is still refreshing.
    Held(WorthQueryOutputDemandKey),
    Pending,
    Fresh(
        super::super::super::execution::prepared_plan::OwnerPlan<
            'owner,
            RequiredFreshProgress<Schema>,
            WorthQueryOutputDemandDenial,
        >,
    ),
}

/// Dispatch by the selected registry row's producer identity. The installed
/// table is an ordered tree; this pays its selected String comparisons before
/// the lookup and does not infer a provider from output family or branch.
pub(super) fn installed_for_selected_cue<'runtime, Schema>(
    runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    producer: &str,
) -> Result<&'runtime InstalledProducerProvider<Schema>, WorthQueryOutputDemandDenial> {
    // These visits pay the ordered-table and producer-width measurements
    // before the variable lookup quote is computed.

    runtime
        .installed_producers
        .entries
        .get(producer)
        .map(std::sync::Arc::as_ref)
        .ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                "required output producer is no longer installed",
            )
        })
}

use performed::certify_required_ready;

fn work_denial() -> WorthQueryOutputDemandDenial {
    denial(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "required output selection exceeds request work",
    )
}

fn capacity_denial() -> WorthQueryOutputDemandDenial {
    denial(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "required output selection exceeds request preparation memory",
    )
}

fn denial(
    kind: WorthQueryOutputDemandDenialKind,
    subject: &'static str,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, subject)
}

/// An exact accepted-row lookup runs before any effect, so its stop denies
/// the request. Only its reasons withhold a candidate.
fn required_settlement_denial(stop: RequiredSettlementStop) -> WorthQueryOutputDemandDenial {
    match stop {
        RequiredSettlementStop::Admission(stop) => admission_denial(stop),
        RequiredSettlementStop::Foreign => foreign_denial(),
    }
}

fn foreign_denial() -> WorthQueryOutputDemandDenial {
    denial(
        WorthQueryOutputDemandDenialKind::ForeignSettlement,
        "required accepted authority belongs to another source",
    )
}

pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn admission_denial(
    stop: CompanionPreflightStop,
) -> WorthQueryOutputDemandDenial {
    use CompanionPreflightStop as Stop;
    match stop {
        Stop::WorkExhausted { .. }
        | Stop::WorkExhaustedWithoutRequirement { .. }
        | Stop::WorkCounterOverflow => work_denial(),
        _ => capacity_denial(),
    }
}

fn position_denial(_: SnapshotPositionDenial) -> WorthQueryOutputDemandDenial {
    denial(
        WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
        "required output native snapshot is unavailable",
    )
}
