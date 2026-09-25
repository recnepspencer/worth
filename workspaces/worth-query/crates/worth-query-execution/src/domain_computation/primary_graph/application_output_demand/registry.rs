use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};

use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;

type SourceEpoch =
    crate::domain_computation::primary_graph::application_query::WorthQueryObservedSourceEpoch;

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryPerformedOutputDemandSource {
    pub(in crate::domain_computation::primary_graph) receipt:
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    pub(in crate::domain_computation::primary_graph) change: Arc<
        crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange,
    >,
    pub(in crate::domain_computation::primary_graph) observation:
        worth_runtime_world::facade::ProductBranchObservation,
    pub(in crate::domain_computation::primary_graph) output_source_identity: Option<SourceEpoch>,
}

struct DiscoveredSourceRecovery {
    receipt: crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    observation: worth_runtime_world::facade::ProductBranchObservation,
    discovery: Arc<dyn std::any::Any + Send + Sync>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
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
    consumed_sources: Vec<SourceEpoch>,
    retired_sources: Vec<(SourceEpoch, WorthQueryOutputDemandDenial)>,
    retired: Option<WorthQueryOutputDemandDenial>,
    token_count: usize,
    completed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct BoundOutputSource {
    pub(in crate::domain_computation::primary_graph) scope:
        crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    pub(in crate::domain_computation::primary_graph) identity: SourceEpoch,
}

impl SourceCustody {
    fn available(
        &self,
        identity: &SourceEpoch,
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

    fn finish_admission(&mut self, identity: SourceEpoch) {
        self.consumed_sources.push(identity);
    }

    fn source_denial(&self, identity: &SourceEpoch) -> Option<WorthQueryOutputDemandDenial> {
        self.retired_sources
            .iter()
            .find(|(retired, _)| retired == identity)
            .map(|(_, cause)| cause.clone())
    }
}

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryAcceptedOutputAuthority {
    Committed(crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt),
    Restored(WorthQueryRestoredAcceptedOutput),
}

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryRestoredAcceptedOutput {
    pub(in crate::domain_computation::primary_graph) checkpoint:
        WorthQueryAcceptedOutputCheckpointIdentity,
    pub(in crate::domain_computation::primary_graph) correspondence: std::sync::Arc<
        crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence,
    >,
    pub(in crate::domain_computation::primary_graph) observation:
        worth_runtime_world::facade::ProductBranchObservation,
    pub(in crate::domain_computation::primary_graph) source_scope:
        crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    pub(in crate::domain_computation::primary_graph) source_identity:
        crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity,
    pub(in crate::domain_computation::primary_graph) observed_source_facts: std::sync::Arc<[
        crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact
    ]>,
}

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryCompletedOutputDemand {
    pub(in crate::domain_computation::primary_graph) authority: WorthQueryAcceptedOutputAuthority,
    pub(in crate::domain_computation::primary_graph) readiness:
        super::WorthQueryOutputReadinessDeliveryEvidence,
    pub(in crate::domain_computation::primary_graph) resources:
        Option<super::super::application_contribution::WorthQueryProducerDemandResources>,
}

pub(in crate::domain_computation::primary_graph) enum WorthQueryOutputSchedulingResult {
    Scheduled,
    Deferred,
    NoEffect(WorthQueryOutputDemandDenial),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryOutputDemandKey {
    producer: String,
    source: SourceEpoch,
}

impl WorthQueryOutputDemandKey {
    pub(in crate::domain_computation::primary_graph) fn new(
        producer: String,
        source: SourceEpoch,
    ) -> Self {
        Self { producer, source }
    }

    fn same_occurrence(&self, other: &Self) -> bool {
        self.producer == other.producer && self.source.same_occurrence(&other.source)
    }

    fn same_semantic_source(&self, other: &Self) -> bool {
        self.producer == other.producer && self.source.same_semantic_source(&other.source)
    }

    fn replacement_order(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (self.producer == other.producer)
            .then(|| self.source.replacement_order(&other.source))
            .flatten()
    }
}

#[derive(Clone)]
pub struct WorthQueryOutputDemandNotifications {
    wake: Arc<DemandWake>,
}

mod accepted_checkpoint;
mod admission;
mod checkpoint;
mod lifecycle;
mod notifications;
mod progression;
mod restoration;
mod source_custody;
mod supersession;
#[cfg(test)]
mod tests;
use checkpoint::{WorthQueryOutputAdvancement, WorthQueryOutputProgress};
pub(in crate::domain_computation::primary_graph) use checkpoint::{
    WorthQueryOutputCheckpoint, WorthQueryOutputClaimIdentity, WorthQueryPendingOutputDelivery,
};
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
    Output(WorthQueryOutputProgress),
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
    successor_of: Option<[u8; 32]>,
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

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryAcceptedOutputCheckpointIdentity {
    pub(in crate::domain_computation::primary_graph) producer: String,
    pub(in crate::domain_computation::primary_graph) source: [u8; 32],
    pub(in crate::domain_computation::primary_graph) scope:
        crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    pub(in crate::domain_computation::primary_graph) source_partition: [u8; 32],
    pub(in crate::domain_computation::primary_graph) producer_dependency: Option<[u8; 32]>,
    pub(in crate::domain_computation::primary_graph) idempotency_key: [u8; 32],
    pub(in crate::domain_computation::primary_graph) resources:
        Option<super::super::application_contribution::WorthQueryProducerDemandResources>,
    pub(in crate::domain_computation::primary_graph) roles:
        Vec<crate::domain_computation::primary_graph::application_attempt::WorthQueryCheckpointOutputRole>,
    /// Authenticated v5 encoding of the complete rebased producer fact set.
    /// Legacy and unsupported fact sets carry no reuse authority.
    pub(in crate::domain_computation::primary_graph) producer_facts: Option<Vec<u8>>,
}

impl WorthQueryAcceptedOutputCheckpointIdentity {
    pub(in crate::domain_computation::primary_graph) fn canonical_cmp(
        &self,
        other: &Self,
    ) -> std::cmp::Ordering {
        self.producer
            .cmp(&other.producer)
            .then_with(|| self.source.cmp(&other.source))
            .then_with(|| self.scope.cmp(&other.scope))
            .then_with(|| self.source_partition.cmp(&other.source_partition))
            .then_with(|| self.producer_dependency.cmp(&other.producer_dependency))
            .then_with(|| self.idempotency_key.cmp(&other.idempotency_key))
            .then_with(|| self.resources.cmp(&other.resources))
            .then_with(|| self.roles.cmp(&other.roles))
    }

    pub(in crate::domain_computation::primary_graph) fn same_output_slot(
        &self,
        other: &Self,
    ) -> bool {
        self.producer == other.producer
            && self.scope == other.scope
            && self.source_partition == other.source_partition
    }
}

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryReadmittedAcceptedOutput {
    pub(in crate::domain_computation::primary_graph) checkpoint:
        WorthQueryAcceptedOutputCheckpointIdentity,
    pub(in crate::domain_computation::primary_graph) correspondence: std::sync::Arc<
        crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence,
    >,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryRequiredOutputSourcePreparation {
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    owner: WorthQueryOutputDemandRegistry,
}

pub(in crate::domain_computation::primary_graph) enum WorthQueryOutputDemandAdvanceAdmission {
    Schedule(Option<WorthQueryPerformedOutputDemandSource>),
    Execute {
        successor_of: Option<[u8; 32]>,
    },
    AdvanceCheckpoint {
        claim: WorthQueryOutputClaimIdentity,
        checkpoint: WorthQueryOutputCheckpoint,
    },
    Ready(WorthQueryCompletedOutputDemand),
    Pending,
    Failed(WorthQueryOutputDemandDenial),
}
