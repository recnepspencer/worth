pub mod checkpoint;
pub use lifecycle::RunningSelectedExecution;

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;

use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;

pub type SourceEpoch =
    crate::domain_computation::primary_graph::application_query::WorthQueryObservedSourceEpoch;

#[derive(Clone)]
pub struct WorthQueryRetainedOutputDemandSource {
    pub receipt:
        super::WorthQueryOutputSourceEvidence,
    pub change: Option<Arc<
        crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange,
    >>,
    pub observation:
        worth_runtime_world::facade::ProductBranchObservation,
    pub output_source_identity: Option<SourceEpoch>,
}

pub struct DiscoveredSourceRecovery {
    receipt: super::WorthQueryOutputSourceEvidence,
    observation: worth_runtime_world::facade::ProductBranchObservation,
    discovery: Arc<dyn std::any::Any + Send + Sync>,
}

pub struct SourceCustody {
    cleanup: cleanup::CustodyCleanupLinks,
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    root_kind: PreparedOutputRootKind,
    source: Option<WorthQueryRetainedOutputDemandSource>,
    discovery: Option<DiscoveredSourceRecovery>,
    bound_sources: Option<Vec<BoundOutputSource>>,
    consumed_sources: Vec<SourceEpoch>,
    retired_sources: Vec<(SourceEpoch, WorthQueryOutputDemandDenial)>,
    retired: Option<WorthQueryOutputDemandDenial>,
    token_count: usize,
    completed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundOutputSource {
    pub scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    pub identity: SourceEpoch,
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
pub enum WorthQueryAcceptedOutputAuthority {
    Recovered(super::WorthQueryRecoveredProducerPublication),
    Committed(crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt),
    Stable(crate::domain_computation::primary_graph::output_lineage::PublishedStableLineage),
    Restored(WorthQueryRestoredAcceptedOutput),
}

#[derive(Clone)]
pub struct WorthQueryCompletedOutputDemand {
    pub authority: WorthQueryAcceptedOutputAuthority,
    pub readiness: super::WorthQueryOutputReadinessDeliveryEvidence,
}

pub enum WorthQueryOutputSchedulingResult {
    Scheduled,
    Deferred,
    NoEffect(WorthQueryOutputDemandDenial),
}

pub mod demand_key;
pub use demand_key::WorthQueryOutputDemandKey;

/// Wake-up signal for one output demand's progress.
///
/// The generation counter advances whenever the demand's state changes. Read
/// `generation()`, then block in `wait_after(observed)` until it moves, and
/// advance the demand with a fresh request. It carries no demand state itself.
#[derive(Clone)]
pub struct WorthQueryOutputDemandNotifications {
    wake: Arc<DemandWake>,
}

pub mod accepted_checkpoint;
pub mod accepted_checkpoint_identity;
pub use accepted_checkpoint::AcceptedCheckpointFactSource;
pub use accepted_checkpoint_identity::CheckpointOutputSlot;
pub use accepted_checkpoint_identity::WorthQueryAcceptedOutputCheckpointIdentity;
pub use accepted_checkpoint_identity::WorthQueryAcceptedOutputCheckpointPosture;
pub mod admission;
pub mod cleanup;
pub mod disposition;
pub mod publication_head;
use disposition::IdleRowDisposition;
use disposition::RowRetirement;
pub use publication_head::LivePublicationHead;
pub use publication_head::PublicationHeadSelection;
pub mod held_successor;
pub mod read_close;
pub mod successor_release;
pub use held_successor::RequiredUpstreamSuccessor;
pub mod lifecycle;
pub mod notifications;
pub mod obligations;
pub mod post_commit;
pub mod prerequisite_claims;
pub mod progression;
pub mod unavailable;
pub use progression::CheckpointFinishStop;
pub use progression::PreparedSelectedCheckpointFinish;
pub mod claim_readthrough;
pub mod custody_visibility;
pub mod ready_backing;

pub mod join_observation;
pub mod record_map;
pub mod record_preparation;
pub mod refresh_predecessor;
pub mod refreshed_rejoin;
pub mod required_context;

pub mod required_members;
pub mod required_stop;
pub mod required_work;
pub use admission::SelectedOutputAdmission;
pub use held_successor::HeldRequiredSuccessor;
pub use required_work::AdmittedRequiredWave;
pub use required_work::PendingUpstream;
pub use required_work::PreparedRequiredInputs;
pub use required_work::ReplacedRequiredWorkHint;
pub use required_work::RequestedOutputReadClaims;
pub use required_work::RequiredRefreshRejection;
pub use required_work::RequiredWorkClaim;
pub use required_work::RequiredWorkMembership;
pub use required_work::SelectedReadyReadmission;
pub use required_work::SelectedRequiredRefreshClaim;
pub use required_work::SelectedRequiredWork;
pub use required_work::SelectedRequiredWorkKind;
pub mod restoration;
pub use restoration::RestoredOutputOrigin;
pub use restoration::WorthQueryRestoredAcceptedOutput;
pub mod consumer_custody;
pub mod settlement_index;
pub mod settlement_progression;

pub mod settlement_retirement;
pub mod succession;
use settlement_retirement::SupersededSettlements;
pub mod row_stage;
pub use row_stage::OutputRowStage;
pub mod source_custody;
pub mod source_readmission;
pub use source_readmission::RetainedOutputReadmissionSource;
pub mod supersession;

use checkpoint::WorthQueryOutputAdvancement;
pub use checkpoint::WorthQueryOutputCheckpoint;
pub use checkpoint::WorthQueryOutputClaimIdentity;
use checkpoint::WorthQueryOutputProgress;
pub use checkpoint::WorthQueryPendingOutputDelivery;
pub use prerequisite_claims::PreparedPrerequisiteClaims;
pub use ready_backing::PreparedReadyBacking;
pub use ready_backing::ReadyCompletion;

pub use refresh_predecessor::OutputRefreshPredecessor;
pub use required_context::RequiredOutputDemandContext;
pub use required_context::RequiredOutputExecution;

use supersession::supersede_predecessors;

pub struct DemandWake {
    generation: Mutex<u64>,
    changed: Condvar,
}

pub enum DemandState {
    Admitted,
    Scheduling,
    Scheduled,
    Running,
    Output(WorthQueryOutputProgress),
    Failed(WorthQueryOutputDemandDenial),
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum DemandAdmissionKind {
    Ordinary,
    Required,
    Recovery,
}

impl DemandAdmissionKind {
    pub const fn is_required(self) -> bool {
        !matches!(self, Self::Ordinary)
    }
}

pub struct DemandRecord {
    pub closed_cleanup: cleanup::ClosedCleanupLinks,
    pub work_membership: Option<Arc<required_work::RequiredWorkMembership>>,
    pub interests: usize,
    pub required_interests: usize,
    pub performed_obligations: Vec<PerformedOutputObligation>,
    pub framework_required_count: usize,
    pub prerequisites: Vec<Arc<WorthQueryOutputDemandKey>>,
    pub claim_batch: Option<claim_readthrough::ClaimBatch>,
    pub checkpoint_prerequisites: Option<prerequisite_claims::CheckpointPrerequisiteClaims>,
    pub prepared_prerequisite_claims: usize,
    pub pending_cleanup_next: Option<Arc<WorthQueryOutputDemandKey>>,
    pub pending_cleanup_queued: bool,

    pub settlements: Vec<
        Arc<crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity>,
    >,
    pub product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    pub source_scope:
        Option<crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding>,
    pub source_commits: Vec<worth_runtime_world::facade::CompositeCommitIdentity>,
    pub state: DemandState,
    pub performed_source: Option<WorthQueryRetainedOutputDemandSource>,
    pub readmission_source: Option<Arc<source_readmission::RequiredOutputReadmission>>,
    pub successor_of: Option<succession::Succession>,
    /// A terminal stop met while certifying or refreshing this row as required
    /// work. Dependents still pending on it, or on an older row of its
    /// occurrence, fail with it instead of waiting for work no advance runs.
    pub required_stop:
        Option<crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind>,
    /// A queue frame's unfinished successor of this row; see `held_successor`.
    pub held_successor: Option<held_successor::HeldRequiredSuccessor>,
    pub wake: Arc<DemandWake>,
}

/// An obligation admitted from an actual retained performed source. Its
/// source commit is kept until settlement or terminal retirement.
pub struct PerformedOutputObligation {
    source_commit: worth_runtime_world::facade::CompositeCommitIdentity,
    source: SourceEpoch,
}

pub struct DemandRegistryState {
    pub last_selection_frozen: Option<bool>,
    join_pause: Option<(
        worth_relational::facade::identity::EntityId,
        join_observation::JoinLatch,
    )>,
    pub records: record_map::DemandRecords,
    // May remain after the last row is removed until the map itself drops.
    required_keys: std::collections::BTreeSet<Arc<WorthQueryOutputDemandKey>>,
    pub required_work_queue: Option<Arc<required_work::RequiredWorkQueue>>,
    discontinuity_cursors: required_work::DiscontinuityCursors,
    settlement_keys: settlement_index::SettlementIndex,
    pending_cleanup_head: Option<Arc<WorthQueryOutputDemandKey>>,
    closed_cleanup: cleanup::ClosedCleanupQueue,
    custody_cleanup: cleanup::CustodyCleanupQueue,
    retention_reservation: Option<Arc<std::sync::atomic::AtomicBool>>,
    source_preparations:
        HashMap<worth_runtime_world::facade::ProductBranchIncarnation, SourcePreparationState>,
    pub source_custody:
        BTreeMap<worth_runtime_world::facade::CompositeCommitIdentity, SourceCustody>,
}

impl Default for DemandRegistryState {
    fn default() -> Self {
        let closed_cleanup = cleanup::ClosedCleanupQueue::default();
        Self {
            records: record_map::DemandRecords::with_cleanup(closed_cleanup.clone()),
            last_selection_frozen: None,

            join_pause: None,
            required_keys: std::collections::BTreeSet::new(),
            required_work_queue: None,
            discontinuity_cursors: required_work::DiscontinuityCursors::default(),
            settlement_keys: settlement_index::SettlementIndex::default(),
            pending_cleanup_head: None,
            closed_cleanup,
            custody_cleanup: Default::default(),
            retention_reservation: None,
            source_preparations: HashMap::new(),
            source_custody: BTreeMap::new(),
        }
    }
}

pub use source_custody::PreparedOutputRootKind;
use source_custody::SourcePreparationState;

#[derive(Clone, Default)]
pub struct WorthQueryOutputDemandRegistry {
    pub state: Arc<Mutex<DemandRegistryState>>,
}

pub struct WorthQueryOutputDemandInterest {
    key: WorthQueryOutputDemandKey,
    requires_output: bool,
    notifications: WorthQueryOutputDemandNotifications,
    owner: WorthQueryOutputDemandRegistry,
}

#[derive(Clone)]
pub struct WorthQueryReadmittedAcceptedOutput {
    pub observation: Option<worth_runtime_world::facade::ProductBranchObservation>,
    pub checkpoint: WorthQueryAcceptedOutputCheckpointIdentity,
    pub correspondence: std::sync::Arc<
        crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence,
    >,
}

pub struct WorthQueryRequiredOutputSourcePreparation {
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    owner: WorthQueryOutputDemandRegistry,
}

pub mod advance_admission;
pub mod fulfillment;
pub mod performed_join;
pub use advance_admission::WorthQueryOutputDemandAdvanceAdmission;

pub use source_custody::WorthQueryDiscoveredRetentionControlDenial;
pub use source_custody::WorthQueryDiscoveredRetentionReservationForTest;

mod court_doors_for_test;
#[path = "registry/demand_record_court_doors_for_test.rs"]
pub mod demand_record_court_doors_for_test;
pub mod home_retirement;
#[path = "registry/source_custody_court_doors_for_test.rs"]
pub mod source_custody_court_doors_for_test;

impl WorthQueryAcceptedOutputAuthority {
    pub fn publication_view(&self) -> Option<super::OutputPublicationView<'_>> {
        match self {
            Self::Committed(receipt) => Some(receipt.into()),
            Self::Recovered(publication) => Some(publication.view()),
            Self::Stable(_) | Self::Restored(_) => None,
        }
    }
}

pub use consumer_custody::{ConsumerCustody, ConsumerHandoff};
#[cfg(test)]
#[path = "registry_inline_type_budget.rs"]
pub mod inline_type_budget;
