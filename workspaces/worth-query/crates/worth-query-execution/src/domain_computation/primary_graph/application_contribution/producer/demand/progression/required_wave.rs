//! One selected Product and native image for an exact required-output wave.

use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use std::sync::Arc;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::{
    mvcc::CompanionPreflightStop,
    runtime::{PositionedRelationalSnapshot, RelationalSnapshotPositionAdmissionStop},
};

use crate::basis::WorthQueryProductBranch;
use crate::domain_computation::primary_graph::{
    application_output_demand::{
        PendingUpstream, SelectedReadyReadmission, WorthQueryOutputDemandInterest,
        WorthQueryOutputDemandKey, WorthQueryOutputDemandSettlement,
    },
    output_lineage::{invalidation::InvalidationEditAdmission, RequiredSettlementStop},
    product_operation::SharedSelectedProductOperation,
    WorthQueryPrimaryGraphApplicationRuntime,
};

use super::super::super::{
    execution::{ProducerExecutionStop, RequiredCueProgress},
    registry::InstalledProducerProvider,
};
use super::super::RequiredFreshProgress;
use super::{WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind};

pub(in crate::domain_computation::primary_graph) use performed::{CurrentBasis, CurrentProof};
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
use resolved::ResolvedOnWave;
pub(in crate::domain_computation::primary_graph) use resolved::{
    MatchedRequiredPredecessors, ReboundConsumedOutput, ResolvedRequiredPredecessors,
};
use selection::{select_required_wave, RequiredWaveSelection};

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
    admission: &mut InvalidationEditAdmission,
) -> Result<&'runtime InstalledProducerProvider<Schema>, WorthQueryOutputDemandDenial> {
    // These visits pay the ordered-table and producer-width measurements
    // before the variable lookup quote is computed.
    admission
        .charge_external_work(5)
        .map_err(|_| work_denial())?;
    let count = runtime.installed_producers.entries.len();
    let levels = usize::BITS as usize - count.max(1).leading_zeros() as usize;
    let comparisons = count.min(11).checked_mul(levels).ok_or_else(work_denial)?;
    let comparison_work = producer.len().checked_add(2).ok_or_else(work_denial)?;
    let work = comparisons
        .checked_mul(comparison_work)
        .and_then(|visits| visits.checked_add(1))
        .ok_or_else(work_denial)?;
    admission
        .charge_external_work(u64::try_from(work).map_err(|_| work_denial())?)
        .map_err(|_| work_denial())?;
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

/// The schema getter materializes its inline identity before the lineage
/// owner receives it. Pay that copy and the caller's fixed owner/getter visits.
fn preclaim_required_settlement_arguments(
    admission: &mut InvalidationEditAdmission,
) -> Result<(), WorthQueryOutputDemandDenial> {
    let work =
        std::mem::size_of::<worth_query_installation::facade::ApplicationSchemaBindingIdentity>()
            .checked_add(6)
            .ok_or_else(work_denial)?;
    admission
        .charge_external_work(u64::try_from(work).map_err(|_| work_denial())?)
        .map_err(|_| work_denial())
}

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

fn position_denial(
    stop: RelationalSnapshotPositionAdmissionStop<CompanionPreflightStop>,
) -> WorthQueryOutputDemandDenial {
    match stop {
        RelationalSnapshotPositionAdmissionStop::Admission(stop) => admission_denial(stop),
        RelationalSnapshotPositionAdmissionStop::AccountingOverflow => capacity_denial(),
        RelationalSnapshotPositionAdmissionStop::Position(_) => denial(
            WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
            "required output native snapshot is unavailable",
        ),
    }
}
