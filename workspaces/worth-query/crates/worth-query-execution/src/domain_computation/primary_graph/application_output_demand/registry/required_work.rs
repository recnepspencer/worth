//! Exact required-work hints shared by the demand record and native marks.

mod branch_selection;
mod discontinuity;
mod interest_readmission;
mod native_hints;
mod pending_readmission;
mod queue;
mod refresh_claim;
mod requested_claim;
mod selection;
mod subsumed;
mod successor_join;

pub(in crate::domain_computation::primary_graph) use pending_readmission::PendingUpstream;
pub(in crate::domain_computation::primary_graph) use refresh_claim::SelectedRequiredRefreshClaim;
pub(in crate::domain_computation::primary_graph) use requested_claim::RequestedOutputReadClaims;
pub(in crate::domain_computation::primary_graph) use selection::SelectedReadyReadmission;

pub(super) use queue::{RequiredWorkPop, RequiredWorkQueue};
pub(super) use selection::charge_required_key_lookup;

pub(super) use discontinuity::DiscontinuityCursors;
use native_hints::FundedNativeBranch;
pub(in crate::domain_computation::primary_graph) use native_hints::NativeHint;

use std::sync::{Arc, Mutex, Weak};

use worth_relational::facade::mvcc::CompanionPublicationCompletionObserver;

use super::record_capacity::RecordCapacity;
use super::{WorthQueryOutputDemandKey, WorthQueryOutputDemandRegistry};
use crate::domain_computation::primary_graph::output_lineage::invalidation::{
    InvalidationEditAdmission, SourceInvalidationOwner,
};
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

pub(in crate::domain_computation::primary_graph) struct RequiredWorkMembership {
    key: Arc<WorthQueryOutputDemandKey>,
    queue: Weak<RequiredWorkQueue>,
    _queue_capacity: Arc<RecordCapacity>,
    state: Mutex<MembershipState>,
    _capacity: RecordCapacity,
}

impl std::fmt::Debug for RequiredWorkMembership {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RequiredWorkMembership")
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

#[derive(Default)]
struct MembershipState {
    active: bool,
    queued: bool,
    version: u64,
    version_exhausted: bool,
    marked: bool,
    unresolved_initial: bool,
    discontinuity_pending: bool,
    native_hints: Option<Box<NativeHint>>,
    local_settlement: Option<Arc<RecordedSettlementIdentity>>,
    next: Option<Arc<RequiredWorkMembership>>,
}

pub(in crate::domain_computation::primary_graph) struct SelectedRequiredWork {
    pub(in crate::domain_computation::primary_graph) membership: Arc<RequiredWorkMembership>,
    pub(in crate::domain_computation::primary_graph) version: u64,
    kind: SelectedRequiredWorkKind,
    armed: bool,
}

pub(in crate::domain_computation::primary_graph) enum SelectedRequiredWorkKind {
    Native {
        observer: CompanionPublicationCompletionObserver,
        branch: Arc<FundedNativeBranch>,
    },
    Local {
        #[cfg_attr(not(test), allow(dead_code))] // Tests read which cutover a selection carries.
        settlement: Arc<RecordedSettlementIdentity>,
    },
    Discontinuity,
    UnresolvedInitial,
}

pub(in crate::domain_computation::primary_graph) struct ReplacedRequiredWorkHint {
    _native: Option<Box<NativeHint>>,
    _settlement: Option<Arc<RecordedSettlementIdentity>>,
}

impl RequiredWorkMembership {
    pub(super) fn new(
        key: Arc<WorthQueryOutputDemandKey>,
        queue: &Arc<RequiredWorkQueue>,
        capacity: RecordCapacity,
    ) -> Self {
        Self {
            key,
            queue: Arc::downgrade(queue),
            _queue_capacity: Arc::clone(&queue.capacity),
            state: Mutex::new(MembershipState::default()),
            _capacity: capacity,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn key(&self) -> &WorthQueryOutputDemandKey {
        &self.key
    }

    pub(super) fn key_arc(&self) -> &Arc<WorthQueryOutputDemandKey> {
        &self.key
    }

    pub(super) fn set_required(self: &Arc<Self>, required: bool) {
        let should_enqueue = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if required && !state.active && !state.marked {
                state.unresolved_initial = true;
                state.marked = true;
                bump_version(&mut state);
            }
            state.active = required;
            required && state.marked
        };
        if should_enqueue {
            if let Some(queue) = self.queue.upgrade() {
                queue.enqueue(self);
            }
        }
    }

    /// The native prepared effect owns this observer. Publish its hint before
    /// testing active membership so a simultaneous activation cannot miss it.
    pub(in crate::domain_computation::primary_graph) fn prepare_hint(
        self: &Arc<Self>,
        mut hint: Box<NativeHint>,
    ) {
        let active = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            bump_version(&mut state);
            state.marked = true;
            hint.next = state.native_hints.take();
            state.native_hints = Some(hint);
            state.active
        };
        if active {
            if let Some(queue) = self.queue.upgrade() {
                queue.enqueue(self);
            }
        }
    }

    pub(in crate::domain_computation::primary_graph) fn mark_local_required(
        self: &Arc<Self>,
        settlement: Arc<RecordedSettlementIdentity>,
    ) -> ReplacedRequiredWorkHint {
        let (prior, active) = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            bump_version(&mut state);
            state.marked = true;
            let prior = ReplacedRequiredWorkHint {
                _native: None,
                _settlement: state.local_settlement.replace(settlement),
            };
            (prior, state.active)
        };
        if active {
            if let Some(queue) = self.queue.upgrade() {
                queue.enqueue(self);
            }
        }
        prior
    }

    pub(super) fn mark_discontinuity(self: &Arc<Self>) {
        let active = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            bump_version(&mut state);
            state.discontinuity_pending = true;
            state.marked = true;
            state.active
        };
        if active {
            if let Some(queue) = self.queue.upgrade() {
                queue.enqueue(self);
            }
        }
    }

    pub(in crate::domain_computation::primary_graph) fn acknowledge(
        self: &Arc<Self>,
        version: u64,
        kind: &SelectedRequiredWorkKind,
    ) -> Option<(ReplacedRequiredWorkHint, bool)> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.version != version || state.version_exhausted {
            return None;
        }
        let removed = match kind {
            SelectedRequiredWorkKind::Native { .. } => {
                let mut head = state
                    .native_hints
                    .take()
                    .expect("selected native hint exists");
                state.native_hints = head.next.take();
                ReplacedRequiredWorkHint {
                    _native: Some(head),
                    _settlement: None,
                }
            }
            SelectedRequiredWorkKind::Local { .. } => ReplacedRequiredWorkHint {
                _native: None,
                _settlement: state.local_settlement.take(),
            },
            SelectedRequiredWorkKind::Discontinuity => {
                state.discontinuity_pending = false;
                ReplacedRequiredWorkHint {
                    _native: None,
                    _settlement: None,
                }
            }
            SelectedRequiredWorkKind::UnresolvedInitial => {
                state.unresolved_initial = false;
                ReplacedRequiredWorkHint {
                    _native: None,
                    _settlement: None,
                }
            }
        };
        Some((removed, finish_acknowledgement(&mut state)))
    }
}

impl SelectedRequiredWork {
    pub(in crate::domain_computation::primary_graph) fn key(&self) -> &WorthQueryOutputDemandKey {
        self.membership.key()
    }

    pub(in crate::domain_computation::primary_graph) fn kind(&self) -> &SelectedRequiredWorkKind {
        &self.kind
    }

    pub(in crate::domain_computation::primary_graph) fn acknowledge(
        mut self,
    ) -> Option<ReplacedRequiredWorkHint> {
        let cleared = self.membership.acknowledge(self.version, &self.kind);
        self.armed = cleared.as_ref().is_none_or(|(_, more)| *more);
        cleared.map(|(removed, _)| removed)
    }

    /// Fund the exact selected-token transition before it can detach its
    /// native/local cause. The bound includes the member state rewrite,
    /// returned carrier, and its possible queue reattachment on Drop.
    pub(in crate::domain_computation::primary_graph) fn acknowledge_admitted(
        self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<ReplacedRequiredWorkHint>, WorthQueryOutputDemandDenial> {
        charge_acknowledgement(admission)?;
        Ok(self.acknowledge())
    }
}

fn charge_acknowledgement(
    admission: &mut InvalidationEditAdmission,
) -> Result<(), WorthQueryOutputDemandDenial> {
    let work = std::mem::size_of::<MembershipState>()
        .checked_mul(3)
        .and_then(|work| {
            std::mem::size_of::<ReplacedRequiredWorkHint>()
                .checked_mul(2)
                .and_then(|carrier| work.checked_add(carrier))
        })
        .and_then(|work| work.checked_add(std::mem::size_of::<SelectedRequiredWork>() + 16))
        .ok_or_else(required_ack_work_denial)?;
    admission
        .charge_external_work(u64::try_from(work).map_err(|_| required_ack_work_denial())?)
        .map_err(|_| required_ack_work_denial())
}

/// A popped member may be requeued and selected again before this
/// acknowledgement. Retire the selected version so that second selection
/// cannot consume the next independent cause. Returns whether causes remain.
fn finish_acknowledgement(state: &mut MembershipState) -> bool {
    bump_version(state);
    state.marked = state.native_hints.is_some()
        || state.local_settlement.is_some()
        || state.discontinuity_pending
        || state.unresolved_initial;
    state.marked
}

fn required_ack_work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, "")
}

fn bump_version(state: &mut MembershipState) {
    if let Some(next) = state.version.checked_add(1) {
        state.version = next;
    } else {
        state.version_exhausted = true;
    }
}

impl Drop for SelectedRequiredWork {
    fn drop(&mut self) {
        if self.armed {
            if let Some(queue) = self.membership.queue.upgrade() {
                queue.enqueue(&self.membership);
            }
        }
    }
}

#[cfg(test)]
impl super::DemandRegistryState {
    pub(super) fn fixture_page_required_discontinuity(
        &mut self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        branch: &worth_relational::facade::history::BranchId,
        epoch: worth_relational::facade::history::CommitId,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        self.page_required_discontinuity(occurrence, branch, epoch, admission)
            .map(|_| ())
    }
}
