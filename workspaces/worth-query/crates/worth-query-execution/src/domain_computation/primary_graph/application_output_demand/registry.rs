use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};

use super::WorthQueryOutputDemandSettlement;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
pub(in crate::domain_computation::primary_graph) struct WorthQueryPendingOutputDelivery {
    pub(in crate::domain_computation::primary_graph) receipt:
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    pub(in crate::domain_computation::primary_graph) change:
        crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryPendingOutputReadiness {
    pub(in crate::domain_computation::primary_graph) receipt:
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    pub(in crate::domain_computation::primary_graph) delivery:
        worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryPerformedOutputDemandSource {
    pub(in crate::domain_computation::primary_graph) receipt:
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    pub(in crate::domain_computation::primary_graph) change:
        crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange,
    pub(in crate::domain_computation::primary_graph) observation:
        worth_runtime_world::facade::ProductBranchObservation,
    pub(in crate::domain_computation::primary_graph) source_identity: [u8; 32],
    pub(in crate::domain_computation::primary_graph) output_source_identity: Option<[u8; 32]>,
}

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryCompletedOutputDemand {
    pub(in crate::domain_computation::primary_graph) receipt:
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    pub(in crate::domain_computation::primary_graph) readiness:
        super::WorthQueryOutputReadinessDeliveryEvidence,
}

pub(in crate::domain_computation::primary_graph) enum WorthQueryOutputSchedulingResult {
    Scheduled,
    Deferred,
    NoEffect(WorthQueryOutputDemandDenial),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryOutputDemandKey {
    producer: String,
    source: [u8; 32],
}

impl WorthQueryOutputDemandKey {
    pub(in crate::domain_computation::primary_graph) fn new(
        producer: String,
        source: [u8; 32],
    ) -> Self {
        Self { producer, source }
    }

    fn same_occurrence(&self, other: &Self) -> bool {
        self.producer == other.producer
            && self.source[..16] == other.source[..16]
            && self.source[24..] == other.source[24..]
    }

    fn revision(&self) -> u64 {
        u64::from_be_bytes(
            self.source[16..24]
                .try_into()
                .expect("source revision occupies eight bytes"),
        )
    }

    fn source_same_occurrence(left: &[u8; 32], right: &[u8; 32]) -> bool {
        left[..16] == right[..16] && left[24..] == right[24..]
    }

    fn source_revision(source: &[u8; 32]) -> u64 {
        u64::from_be_bytes(
            source[16..24]
                .try_into()
                .expect("source revision occupies eight bytes"),
        )
    }
}

#[derive(Clone)]
pub struct WorthQueryOutputDemandNotifications {
    wake: Arc<DemandWake>,
}

mod admission;
mod lifecycle;
mod notifications;
mod program_recovery;
mod progression;
mod source_custody;
mod supersession;
#[cfg(test)]
mod tests;
use supersession::supersede_predecessors;

#[derive(Default)]
struct DemandWake {
    generation: Mutex<u64>,
    changed: Condvar,
}

impl DemandWake {
    fn notify(&self) {
        let mut generation = self
            .generation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *generation = generation.saturating_add(1);
        self.changed.notify_all();
    }
}

enum DemandState {
    Admitted,
    Scheduling,
    Scheduled,
    Running,
    Delivering,
    DeliveryPending(Option<WorthQueryPendingOutputDelivery>),
    ReadinessPending(Option<WorthQueryPendingOutputReadiness>),
    EvaluatingReadiness,
    Settled(Arc<WorthQueryOutputDemandSettlement>),
    Completed(WorthQueryCompletedOutputDemand),
    Recovering(WorthQueryCompletedOutputDemand),
    Failed(WorthQueryOutputDemandDenial),
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum DemandAdmissionKind {
    Ordinary,
    Required,
    Recovery,
}

impl DemandAdmissionKind {
    const fn is_required(self) -> bool {
        !matches!(self, Self::Ordinary)
    }
}

struct DemandRecord {
    interests: usize,
    required: bool,
    product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    source_scope:
        Option<crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding>,
    source_commit: Option<worth_runtime_world::facade::CompositeCommitIdentity>,
    state: DemandState,
    performed_source: Option<WorthQueryPerformedOutputDemandSource>,
    performed_source_accepted: bool,
    wake: Arc<DemandWake>,
}

#[derive(Default)]
struct DemandRegistryState {
    wake: Arc<DemandWake>,
    records: HashMap<WorthQueryOutputDemandKey, DemandRecord>,
    source_preparations:
        HashMap<worth_runtime_world::facade::ProductBranchIncarnation, SourcePreparationState>,
    prepared_sources: Vec<(
        worth_runtime_world::facade::CompositeCommitIdentity,
        WorthQueryPerformedOutputDemandSource,
    )>,
    retired_prepared_sources:
        HashMap<worth_runtime_world::facade::CompositeCommitIdentity, WorthQueryOutputDemandDenial>,
    program_recovery: Vec<ProgramRecoveryCustody>,
}

struct ProgramRecoveryCustody {
    provider_runtime_instance_id: u64,
    product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    source_commit: worth_runtime_world::facade::CompositeCommitIdentity,
    inventory: std::any::TypeId,
    root: std::any::TypeId,
    demand: Box<dyn std::any::Any + Send + Sync>,
}

#[derive(Default)]
struct SourcePreparationState {
    active: usize,
    retired: bool,
}

#[derive(Clone, Default)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryOutputDemandRegistry {
    state: Arc<Mutex<DemandRegistryState>>,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryOutputDemandInterest {
    key: WorthQueryOutputDemandKey,
    notifications: WorthQueryOutputDemandNotifications,
    owner: WorthQueryOutputDemandRegistry,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryRequiredOutputSourcePreparation {
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    owner: WorthQueryOutputDemandRegistry,
}

pub(in crate::domain_computation::primary_graph) enum WorthQueryOutputDemandAdvanceAdmission {
    Schedule(Option<WorthQueryPerformedOutputDemandSource>),
    Execute,
    Deliver(WorthQueryPendingOutputDelivery),
    EvaluateReadiness(WorthQueryPendingOutputReadiness),
    Recover(WorthQueryCompletedOutputDemand),
    Pending,
    Settled(Arc<WorthQueryOutputDemandSettlement>),
    Failed(WorthQueryOutputDemandDenial),
}
