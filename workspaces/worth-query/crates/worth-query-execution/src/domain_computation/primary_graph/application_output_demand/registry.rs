use std::collections::{BTreeMap, HashMap};
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
    Stable(crate::domain_computation::primary_graph::output_lineage::PublishedStableLineage),
    Restored(WorthQueryRestoredAcceptedOutput),
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

mod demand_key;
pub(in crate::domain_computation::primary_graph) use demand_key::WorthQueryOutputDemandKey;

/// Wake-up signal for one output demand's progress.
///
/// The generation counter advances whenever the demand's state changes. Read
/// `generation()`, then block in `wait_after(observed)` until it moves, and
/// advance the demand with a fresh request. It carries no demand state itself.
#[derive(Clone)]
pub struct WorthQueryOutputDemandNotifications {
    wake: Arc<DemandWake>,
}

mod accepted_checkpoint;
mod accepted_checkpoint_identity;
pub(in crate::domain_computation::primary_graph) use accepted_checkpoint::AcceptedCheckpointFactSource;
pub(in crate::domain_computation::primary_graph) use accepted_checkpoint_identity::{
    CheckpointOutputSlot, WorthQueryAcceptedOutputCheckpointIdentity,
    WorthQueryAcceptedOutputCheckpointPosture,
};
mod admission;
mod checkpoint;
mod cleanup;
mod held_successor;
mod lifecycle;
mod notifications;
mod obligations;
mod prerequisite_claims;
mod prerequisite_work;
mod progression;
pub(in crate::domain_computation::primary_graph) use progression::{
    PreparedSelectedCheckpointFinish, SelectedCheckpointFinishStop,
};
mod ready_backing;
mod record_capacity;
mod refresh_predecessor;
mod refreshed_rejoin;
mod required_context;
mod required_custody;
mod required_members;
mod required_stop;
mod required_work;
pub(in crate::domain_computation::primary_graph) use admission::SelectedOutputAdmission;
pub(in crate::domain_computation::primary_graph) use held_successor::HeldRequiredSuccessor;
pub(in crate::domain_computation::primary_graph) use required_work::PendingUpstream;
pub(in crate::domain_computation::primary_graph) use required_work::ReplacedRequiredWorkHint;
pub(in crate::domain_computation::primary_graph) use required_work::RequestedOutputReadClaims;
pub(in crate::domain_computation::primary_graph) use required_work::RequiredWorkMembership;
pub(in crate::domain_computation::primary_graph) use required_work::SelectedReadyReadmission;
pub(in crate::domain_computation::primary_graph) use required_work::SelectedRequiredRefreshClaim;
pub(in crate::domain_computation::primary_graph) use required_work::SelectedRequiredWork;
pub(in crate::domain_computation::primary_graph) use required_work::SelectedRequiredWorkKind;
mod restoration;
pub(in crate::domain_computation::primary_graph) use restoration::WorthQueryRestoredAcceptedOutput;
mod settlement_index;
mod settlement_progression;
mod settlement_retirement;
mod succession;
use settlement_retirement::SupersededSettlements;
mod row_stage;
pub(in crate::domain_computation::primary_graph) use row_stage::OutputRowStage;
mod source_custody;
mod source_readmission;
pub(in crate::domain_computation::primary_graph) use source_readmission::RetainedOutputReadmissionSource;
mod supersession;
#[cfg(test)]
mod tests;
use checkpoint::{WorthQueryOutputAdvancement, WorthQueryOutputProgress};
pub(in crate::domain_computation::primary_graph) use checkpoint::{
    WorthQueryOutputCheckpoint, WorthQueryOutputClaimIdentity, WorthQueryPendingOutputDelivery,
};
pub(in crate::domain_computation::primary_graph) use prerequisite_claims::PreparedPrerequisiteClaims;
#[cfg(feature = "test-query-execution-observer")]
pub use ready_backing::required_ready_custody_bytes_for_test;
pub(in crate::domain_computation::primary_graph) use ready_backing::PreparedReadyBacking;
pub(in crate::domain_computation::primary_graph) use ready_backing::ReadyCompletion;
pub(in crate::domain_computation::primary_graph) use refresh_predecessor::OutputRefreshPredecessor;
pub(in crate::domain_computation) use required_context::{
    RequiredOutputDemandContext, RequiredOutputExecution,
};
pub(in crate::domain_computation::primary_graph) use required_custody::RequiredOutputCustodyCapacity;
use supersession::supersede_predecessors;

struct DemandWake {
    generation: Mutex<u64>,
    changed: Condvar,
    _record_capacity: record_capacity::RecordCapacity,
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
    _record_capacity: record_capacity::RecordCapacity,
    work_membership: Option<Arc<required_work::RequiredWorkMembership>>,
    interests: usize,
    required_interests: usize,
    performed_obligations: Vec<PerformedOutputObligation>,
    framework_required_count: usize,
    prerequisites: Vec<Arc<WorthQueryOutputDemandKey>>,
    checkpoint_prerequisites: Option<prerequisite_claims::CheckpointPrerequisiteClaims>,
    prepared_prerequisite_claims: usize,
    pending_cleanup_next: Option<Arc<WorthQueryOutputDemandKey>>,
    pending_cleanup_queued: bool,
    pending_cleanup_key_bytes: usize,
    settlements: Vec<(
        Arc<crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity>,
        usize,
    )>,
    product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    source_scope:
        Option<crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding>,
    source_commits: Vec<worth_runtime_world::facade::CompositeCommitIdentity>,
    source_commit_capacity: Option<record_capacity::RecordCapacity>,
    state: DemandState,
    performed_source: Option<WorthQueryPerformedOutputDemandSource>,
    readmission_source: Option<Arc<source_readmission::RequiredOutputReadmission>>,
    successor_of: Option<succession::Succession>,
    /// A terminal stop met while certifying or refreshing this row as required
    /// work. Dependents still pending on it, or on an older row of its
    /// occurrence, fail with it instead of waiting for work no advance runs.
    required_stop:
        Option<crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind>,
    /// A queue frame's unfinished successor of this row; see `held_successor`.
    held_successor: Option<held_successor::HeldRequiredSuccessor>,
    wake: Arc<DemandWake>,
}

/// An obligation admitted from an actual retained performed source. Its
/// source commit is kept until settlement or terminal retirement.
struct PerformedOutputObligation {
    source_commit: worth_runtime_world::facade::CompositeCommitIdentity,
    source: SourceEpoch,
}

struct DemandRegistryState {
    records: BTreeMap<WorthQueryOutputDemandKey, DemandRecord>,
    // May remain after the last row is removed until the map itself drops.
    _empty_root_capacity: Option<record_capacity::RecordCapacity>,
    record_budget_bytes: usize,
    record_retained_bytes: Arc<std::sync::atomic::AtomicUsize>,
    obligation_budget_bytes: usize,
    obligation_reserved_bytes: usize,
    required_keys: std::collections::BTreeSet<Arc<WorthQueryOutputDemandKey>>,
    required_work_queue: Option<Arc<required_work::RequiredWorkQueue>>,
    discontinuity_cursors: required_work::DiscontinuityCursors,
    required_budget_bytes: usize,
    required_reserved_bytes: usize,
    required_custody_retained_bytes: Arc<std::sync::atomic::AtomicUsize>,
    settlement_keys: settlement_index::SettlementIndex,
    pending_cleanup_head: Option<Arc<WorthQueryOutputDemandKey>>,
    source_preparations:
        HashMap<worth_runtime_world::facade::ProductBranchIncarnation, SourcePreparationState>,
    source_custody: HashMap<worth_runtime_world::facade::CompositeCommitIdentity, SourceCustody>,
}

impl Default for DemandRegistryState {
    fn default() -> Self {
        Self {
            records: BTreeMap::new(),
            _empty_root_capacity: None,
            record_budget_bytes: crate::domain_computation::execution_runtime::WorthQueryOutputDemandResourceProfile::standard().registry_record_retained_bytes(),
            record_retained_bytes: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            obligation_budget_bytes: crate::domain_computation::execution_runtime::WorthQueryOutputDemandResourceProfile::standard().registry_obligation_retained_bytes(),
            obligation_reserved_bytes: 0,
            required_keys: std::collections::BTreeSet::new(),
            required_work_queue: None,
            discontinuity_cursors: required_work::DiscontinuityCursors::default(),
            required_budget_bytes: crate::domain_computation::execution_runtime::WorthQueryOutputDemandResourceProfile::standard().registry_required_retained_bytes(),
            required_reserved_bytes: 0,
            required_custody_retained_bytes: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            settlement_keys: settlement_index::SettlementIndex::default(),
            pending_cleanup_head: None,
            source_preparations: HashMap::new(),
            source_custody: HashMap::new(),
        }
    }
}

pub(in crate::domain_computation::primary_graph) use source_custody::PreparedOutputRootKind;
use source_custody::SourcePreparationState;

#[derive(Clone, Default)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryOutputDemandRegistry {
    state: Arc<Mutex<DemandRegistryState>>,
}

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn with_budgets(
        obligation_bytes: usize,
        record_bytes: usize,
        required_bytes: usize,
    ) -> Self {
        let state = DemandRegistryState {
            obligation_budget_bytes: obligation_bytes,
            record_budget_bytes: record_bytes,
            required_budget_bytes: required_bytes,
            ..DemandRegistryState::default()
        };
        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryOutputDemandInterest {
    key: WorthQueryOutputDemandKey,
    requires_output: bool,
    notifications: WorthQueryOutputDemandNotifications,
    owner: WorthQueryOutputDemandRegistry,
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
    Ready(ReadyCompletion),
    Pending,
    Failed(WorthQueryOutputDemandDenial),
}
