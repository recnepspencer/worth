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

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryPerformedOutputDemandSource {
    pub(in crate::domain_computation::primary_graph) receipt:
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    pub(in crate::domain_computation::primary_graph) change: Arc<
        crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange,
    >,
    pub(in crate::domain_computation::primary_graph) observation:
        worth_runtime_world::facade::ProductBranchObservation,
    pub(in crate::domain_computation::primary_graph) output_source_identity: Option<[u8; 32]>,
}

struct DiscoveredSourceRecovery {
    receipt: crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    observation: worth_runtime_world::facade::ProductBranchObservation,
    discovery: Arc<dyn std::any::Any + Send + Sync>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum PreparedOutputRootKind {
    Required(std::any::TypeId),
    Discovered(std::any::TypeId),
}

struct SourceCustody {
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    root_kind: PreparedOutputRootKind,
    source: Option<WorthQueryPerformedOutputDemandSource>,
    discovery: Option<DiscoveredSourceRecovery>,
    bound_sources: Option<Vec<BoundOutputSource>>,
    consumed_sources: Vec<[u8; 32]>,
    retired_sources: Vec<([u8; 32], WorthQueryOutputDemandDenial)>,
    retired: Option<WorthQueryOutputDemandDenial>,
    token_count: usize,
    completed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct BoundOutputSource {
    pub(in crate::domain_computation::primary_graph) scope:
        crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    pub(in crate::domain_computation::primary_graph) identity: [u8; 32],
}

impl SourceCustody {
    fn available(
        &self,
        identity: &[u8; 32],
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    ) -> bool {
        self.retired.is_none()
            && self.source.is_some()
            && self.bound_sources.as_ref().is_some_and(|sources| {
                sources
                    .iter()
                    .any(|source| &source.identity == identity && source.scope == scope)
            })
            && !self.consumed_sources.contains(identity)
            && !self
                .retired_sources
                .iter()
                .any(|(retired, _)| retired == identity)
    }

    #[cfg(any(test, feature = "test-primary-graph-faults"))]
    fn prepared_count(&self) -> usize {
        if self.retired.is_some() || self.source.is_none() {
            return 0;
        }
        self.bound_sources.as_ref().map_or(1, |sources| {
            sources
                .iter()
                .filter(|source| {
                    !self.consumed_sources.contains(&source.identity)
                        && self.source_denial(&source.identity).is_none()
                })
                .count()
        })
    }

    fn finish_admission(&mut self, identity: [u8; 32]) {
        self.consumed_sources.push(identity);
    }

    fn source_denial(&self, identity: &[u8; 32]) -> Option<WorthQueryOutputDemandDenial> {
        self.retired_sources
            .iter()
            .find(|(retired, _)| retired == identity)
            .map(|(_, cause)| cause.clone())
    }
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
mod progression;
mod source_custody;
mod supersession;
#[cfg(test)]
mod tests;
use supersession::supersede_predecessors;

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
    source_commits: Vec<worth_runtime_world::facade::CompositeCommitIdentity>,
    state: DemandState,
    performed_source: Option<WorthQueryPerformedOutputDemandSource>,
    wake: Arc<DemandWake>,
}

#[derive(Default)]
struct DemandRegistryState {
    records: HashMap<WorthQueryOutputDemandKey, DemandRecord>,
    source_preparations:
        HashMap<worth_runtime_world::facade::ProductBranchIncarnation, SourcePreparationState>,
    source_custody: HashMap<worth_runtime_world::facade::CompositeCommitIdentity, SourceCustody>,
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
