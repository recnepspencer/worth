//! One selected Product and native image for an exact required-output wave.

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

/// A caller's Ready or an exactly requested upstream Ready anchors the wave.
/// `shared` and `positioned` retain its admitted Product/native proof basis.
pub(super) struct RequiredWaveSelection<'runtime, Schema> {
    pub(super) shared: SharedSelectedProductOperation<'runtime, Schema>,
    pub(super) positioned: PositionedRelationalSnapshot,
    pub(super) anchor_ready: SelectedReadyReadmission,
    pub(super) branch: WorthQueryProductBranch,
    pub(super) target: RequiredWaveTarget,
}

mod caller;
mod requested;
pub(super) use requested::advance_requested_output;
mod cycles;
mod drive;
mod frame;
use frame::{FrameRole, RequiredWaveFrame};
mod queued;
mod resolved;
mod selection;
use resolved::ResolvedOnWave;
pub(in crate::domain_computation::primary_graph) use resolved::{
    MatchedRequiredPredecessors, ReboundConsumedOutput, ResolvedRequiredPredecessors,
};
use selection::{reselect_required_wave, select_required_wave};

pub(super) use caller::advance_required_before_caller;

/// A requested upstream certifies only itself, never its initial consumer.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum RequiredWaveTarget {
    Caller,
    Requested,
}

/// A cue is only scheduling custody. A Current result comes from the exact
/// accepted row, its installed producer, and the selected Product/native proof.
pub(super) enum RequiredWaveStep<Schema: ApplicationSchema> {
    Current(Arc<WorthQueryOutputDemandSettlement>),
    Upstream(SelectedReadyReadmission),
    /// The upstream row is still refreshing.
    Held(WorthQueryOutputDemandKey),
    Pending,
    Fresh(RequiredFreshProgress<Schema>),
}

/// Only the exact dependency chain currently being resolved is held here.
/// It never enumerates unrelated required or clean records.
struct RequiredWaveStack {
    frames: Vec<RequiredWaveFrame>,
}

impl RequiredWaveStack {
    fn new() -> Self {
        Self { frames: Vec::new() }
    }

    /// Preserve role and contact attribution while a prerequisite runs.
    fn suspend_current(
        &mut self,
        current: &mut Option<SelectedReadyReadmission>,
        role: FrameRole,
        contacts: usize,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        match current.take() {
            Some(ready) => self.push(
                RequiredWaveFrame {
                    ready,
                    role,
                    contacts,
                },
                admission,
            ),
            None => Ok(true),
        }
    }

    /// A repeated live registry member is a pending cycle, not authority to
    /// discharge any edge. The caller keeps its original interest for retry.
    fn push(
        &mut self,
        next: RequiredWaveFrame,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        admission
            .charge_external_work(3)
            .map_err(|_| work_denial())?;
        for prior in &self.frames {
            if prior.ready.same_record(&next.ready, admission)? {
                return Ok(false);
            }
        }
        let old_bytes = self
            .frames
            .capacity()
            .checked_mul(std::mem::size_of::<RequiredWaveFrame>())
            .ok_or_else(capacity_denial)?;
        let next_len = self
            .frames
            .len()
            .checked_add(1)
            .ok_or_else(capacity_denial)?;
        if self.frames.len() == self.frames.capacity() {
            // A chain grows geometrically so successive prerequisites do not
            // repeatedly relocate the whole stack. The old and new buffers
            // coexist during reserve; the selected pin is then written once.
            let next_capacity = self.frames.capacity().saturating_mul(2).max(next_len);
            let new_bytes = next_capacity
                .checked_mul(std::mem::size_of::<RequiredWaveFrame>())
                .ok_or_else(capacity_denial)?;
            let peak = old_bytes
                .checked_add(new_bytes)
                .ok_or_else(capacity_denial)?;
            admission
                .admit_read_scratch(u64::try_from(peak).map_err(|_| capacity_denial())?)
                .map_err(admission_denial)?;
            let moved_and_written = old_bytes
                .checked_mul(2)
                .ok_or_else(work_denial)?
                .checked_add(std::mem::size_of::<RequiredWaveFrame>())
                .and_then(|work| work.checked_add(3))
                .ok_or_else(work_denial)?;
            admission
                .charge_external_work(u64::try_from(moved_and_written).map_err(|_| work_denial())?)
                .map_err(|_| work_denial())?;
            let mut replacement = Vec::new();
            replacement
                .try_reserve_exact(next_capacity)
                .map_err(|_| capacity_denial())?;
            if replacement.capacity() != next_capacity {
                return Err(capacity_denial());
            }
            replacement.append(&mut self.frames);
            self.frames = replacement;
        } else {
            admission
                .charge_external_work(
                    u64::try_from(std::mem::size_of::<RequiredWaveFrame>() + 1)
                        .map_err(|_| work_denial())?,
                )
                .map_err(|_| work_denial())?;
        }
        self.frames.push(next);
        Ok(true)
    }

    fn pop(&mut self) -> Option<RequiredWaveFrame> {
        self.frames.pop()
    }
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

/// Certify one exact Ready cell while the wave retains one selected Product.
/// The registry's exact consumed-ID join converts PendingExact to owned
/// scheduling custody before the candidate's borrowed proof leaves scope.
pub(super) fn certify_required_ready<Schema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request_scope: &WorthQueryRequestScope,
    wave: &RequiredWaveSelection<'_, Schema>,
    selected: &SelectedReadyReadmission,
    caller_installed: Option<&InstalledProducerProvider<Schema>>,
    resolved: Option<&ResolvedRequiredPredecessors<'_, Schema>>,
    producer_contacts_in_this_demand: usize,
    refresh_permission: Result<(), WorthQueryOutputDemandDenial>,
    admission: &mut InvalidationEditAdmission,
) -> Result<RequiredWaveStep<Schema>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
    preclaim_required_settlement_arguments(admission)?;
    let candidate = runtime
        .primary_provider
        .graph
        .output_lineage
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .resolve_required_settlement(
            runtime.runtime.authority_identity().as_u64(),
            &runtime.installed_schema.binding_identity(),
            selected.completion(),
            admission,
        );
    // A reason withholds the candidate: the selected Ready is verified in
    // full.
    let candidate = candidate
        .map_err(required_settlement_denial)?
        .ok()
        .flatten();
    admission
        .charge_external_work(1)
        .map_err(admission_denial)?;
    // Only the caller's exact InterestÃ¢â€ â€™Ready join may reuse the installed
    // entry retained when that same demand was admitted. Other cues still
    // resolve their own producer through the installed table.
    let installed = match caller_installed {
        Some(entry) => entry,
        None => installed_for_selected_cue(runtime, selected.producer_identity(), admission)?,
    };
    let result = installed
        .executor
        .advance_ready_on_selected(
            runtime,
            principal,
            request_scope,
            &wave.shared,
            &wave.positioned,
            selected,
            candidate.as_ref(),
            resolved,
            installed,
            wave.branch,
            producer_contacts_in_this_demand,
            refresh_permission,
            admission,
        )
        .map_err(|stop| match stop {
            ProducerExecutionStop::RequestAdmissionDenied(rejection) => rejection.into_denial(),
            ProducerExecutionStop::ExecutionStopped(denial) => denial,
            ProducerExecutionStop::LiveOutputNotReused { producer, reason } => {
                ProducerExecutionStop::live_output_not_reused(producer, reason)
            }
        })?;
    match result {
        RequiredCueProgress::Current(settlement) => Ok(RequiredWaveStep::Current(settlement)),
        RequiredCueProgress::PendingExact(pending) => {
            let upstream = runtime.output_demands.pending_exact_ready_readmission(
                &pending,
                &wave.positioned,
                admission,
            )?;
            Ok(match upstream {
                PendingUpstream::Ready(ready) => RequiredWaveStep::Upstream(ready),
                PendingUpstream::Held(head) => RequiredWaveStep::Held(head),
                PendingUpstream::Unavailable(_) => RequiredWaveStep::Pending,
            })
        }
        RequiredCueProgress::PendingUnresolved => Ok(RequiredWaveStep::Pending),
        RequiredCueProgress::Fresh(progress) => Ok(RequiredWaveStep::Fresh(progress)),
    }
}

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

fn admission_denial(stop: CompanionPreflightStop) -> WorthQueryOutputDemandDenial {
    use CompanionPreflightStop as Stop;
    match stop {
        Stop::WorkExhausted { .. } | Stop::WorkCounterOverflow => work_denial(),
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
