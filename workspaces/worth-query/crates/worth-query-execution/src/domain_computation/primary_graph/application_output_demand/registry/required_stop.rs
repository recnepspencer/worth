//! A required row whose certification or refresh stopped for every principal.
//!
//! A dependent's consumed edge names the row it read, and only the wave that
//! refreshes that row can resolve the edge. When the row's producer or program
//! itself cannot refresh it, no later advance by any caller resolves the edge:
//! its dependents fail with the recorded stop instead of reporting Pending
//! forever. Only such intrinsic stops are recorded on the shared row. A stop
//! that belongs to the request that met it (its principal, scope, deadline,
//! cancellation or budgets) or to a moment (a stale publication, a newer
//! source, missing coverage) stays with that advance. The newest row of the
//! occurrence decides: its own stopped state, or the recorded stop, which its
//! successful certification or refresh clears.

use std::collections::BTreeMap;

use super::{
    DemandRecord, DemandState, WorthQueryOutputAdvancement, WorthQueryOutputDemandKey,
    WorthQueryOutputDemandRegistry,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryOutputDemandRecoveryPosture,
};

impl WorthQueryOutputDemandRegistry {
    /// Record `stop` on the row whose required certification or refresh it
    /// ended, when the row fails the same way for any principal.
    pub(in crate::domain_computation::primary_graph) fn record_required_stop(
        &self,
        key: &WorthQueryOutputDemandKey,
        stop: &WorthQueryOutputDemandDenial,
    ) {
        if stop.recovery_posture() != WorthQueryOutputDemandRecoveryPosture::Terminal
            || !intrinsic_to_row(stop.kind())
        {
            return;
        }
        self.set_required_stop(key, Some(stop.kind()));
    }

    /// The row certified Current or refreshed: no recorded stop survives it.
    pub(in crate::domain_computation::primary_graph) fn clear_required_stop(
        &self,
        key: &WorthQueryOutputDemandKey,
    ) {
        self.set_required_stop(key, None);
    }

    fn set_required_stop(
        &self,
        key: &WorthQueryOutputDemandKey,
        stop: Option<WorthQueryOutputDemandDenialKind>,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(record) = state.records.get_mut(key) {
            record.required_stop = stop;
        }
    }
}

/// Stops that the row's installed producer, program or source decide, the same
/// for every principal and every request.
pub(super) fn intrinsic_to_row(kind: WorthQueryOutputDemandDenialKind) -> bool {
    use WorthQueryOutputDemandDenialKind as Kind;
    match kind {
        Kind::SourceQueryInstallation(_)
        | Kind::ForeignSource
        | Kind::MissingApplicableProducer
        | Kind::AmbiguousApplicableProducer
        | Kind::ProducerUnavailable
        | Kind::ForeignDemand
        | Kind::ForeignSettlement => true,
        Kind::SourcePrincipal(_)
        | Kind::SourceScope(_)
        | Kind::SourceQueryAdmission(_)
        | Kind::SourceQueryExecution(_)
        | Kind::RequestAuthorization(_)
        | Kind::ProductSelection(_)
        | Kind::SchedulingRejected
        | Kind::SchedulingDeferred
        | Kind::PublicationStale
        | Kind::NoEffect
        | Kind::Superseded
        | Kind::Cancelled
        | Kind::TimedOut
        | Kind::WorkBudgetExceeded
        | Kind::RetentionBudgetExceeded
        | Kind::PublicationCapacityExceeded
        | Kind::IncompleteDependencyCoverage
        | Kind::RetainedBasisUnavailable
        | Kind::Closed
        | Kind::DuplicatePerformedSource => false,
    }
}

/// The newest row of `key`'s refresh lineage: `key` itself until a refresh of
/// its occurrence replaced it. An edge that read `key` waits on that row.
pub(super) fn lineage_head(
    records: &BTreeMap<WorthQueryOutputDemandKey, DemandRecord>,
    key: &WorthQueryOutputDemandKey,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<WorthQueryOutputDemandKey>, WorthQueryOutputDemandDenial> {
    let Some(record) = records.get(key) else {
        return Ok(None);
    };
    if !super::refreshed_rejoin::superseded(record) {
        return Ok(Some(key.clone()));
    }
    // One pass over the rows compares each key's occurrence once.
    admission
        .charge_external_work(u64::try_from(records.len()).map_err(|_| work_denial())?)
        .map_err(|_| work_denial())?;
    Ok(Some(
        super::refreshed_rejoin::newest_of_occurrence(records, key).unwrap_or_else(|| key.clone()),
    ))
}

/// Why the head row of a refresh lineage can never become Ready: the row
/// itself, or its refresh, met a stop intrinsic to it. Any other stop
/// belonged to the advance that met it.
pub(super) fn head_stop(
    records: &BTreeMap<WorthQueryOutputDemandKey, DemandRecord>,
    key: &WorthQueryOutputDemandKey,
) -> Option<WorthQueryOutputDemandDenial> {
    let record = records.get(key)?;
    let stopped = match &record.state {
        DemandState::Failed(denial) => Some(denial),
        DemandState::Output(output) => match &output.advancement {
            WorthQueryOutputAdvancement::Stopped { denial, .. } => Some(denial),
            _ => None,
        },
        _ => None,
    };
    if let Some(denial) = stopped.filter(|denial| intrinsic_to_row(denial.kind())) {
        return Some(denial.clone());
    }
    record.required_stop.map(|kind| {
        WorthQueryOutputDemandDenial::new(kind, "the required upstream refresh stopped")
    })
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "required upstream stop lookup exceeds request work",
    )
}
