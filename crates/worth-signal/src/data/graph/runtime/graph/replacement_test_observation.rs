use std::sync::Arc;

use crate::data::aspect::{Aspect, AspectMask, AspectVersion};
use crate::data::conditional_execution::SignalConditionalVersionObservation;

use crate::data::graph::runtime::scratch::ScratchLeaseKind;
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::binding::DependencyRevision;
use crate::data::proof::invalidation::progression::InvalidationWorkBindingAxes;
use crate::data::proof::invalidation::progression::{
    InvalidationOriginBinding, InvalidationReadinessEpoch, InvalidationStageOrder,
};
use crate::data::telemetry::SignalInvalidationRealizedCounters;
use crate::logic::evaluation::EvaluationRequestMode;
use crate::logic::planner::{
    EligibleTask, EligibleTaskAdmission, StageBarrier, StageCursor, TaskReason,
};
use crate::logic::transaction::{SignalObservationCompletion, SignalObservationRequest};
use crate::schema::data::SignalSchemaRegistry;

use super::SignalGraph;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SignalGraphRetainedObservation {
    pub(crate) serialized_authority: serde_json::Value,
    pub(crate) schema_registry: SignalSchemaRegistry,
    pub(crate) cause_readmission_required: bool,
    pub(crate) traversal: SignalTraversalReplacementObservation,
    pub(crate) conditional_dependency_versions: Vec<(NodeId, SignalConditionalVersionObservation)>,
    pub(crate) authorization_policy_identities: Vec<[u8; 32]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SignalTraversalReplacementObservation {
    scratch_lease: Option<ScratchLeaseKind>,
    node_buffer_a: Vec<NodeId>,
    node_buffer_b: Vec<NodeId>,
    planner_targets: Vec<NodeId>,
    planner_tasks: Vec<EligibleTask>,
    planner_stages: Vec<StageCursor>,
    topology_node_buffer: Vec<NodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SignalGraphCloneLocalObservation {
    pub(crate) lifecycle_token_identity: usize,
    pub(crate) graph_instance_id: u64,
    pub(crate) schema_registry_identity: usize,
    pub(crate) aspect_lowering_owner_present: bool,
    pub(crate) invalidation_readiness_epoch: u64,
    pub(crate) observation_active_generation: u64,
    pub(crate) observation_active_request: SignalObservationRequest,
    pub(crate) completed_execution_boundaries: u64,
    pub(crate) last_completion: Option<SignalObservationCompletion>,
    pub(crate) observation_cleanup_identity: Option<usize>,
    pub(crate) performed_counters: SignalInvalidationRealizedCounters,
    pub(crate) performed_work: Vec<InvalidationWorkBindingAxes>,
    pub(crate) pending_repeated_admissions: Vec<(NodeId, u64)>,
}

impl SignalGraph {
    pub(crate) fn replacement_retained_observation(&self) -> SignalGraphRetainedObservation {
        SignalGraphRetainedObservation {
            serialized_authority: serde_json::to_value(self)
                .expect("Signal graph retained authority serializes for test observation"),
            schema_registry: (*self.schema_registry).clone(),
            cause_readmission_required: self.cause_readmission_required,
            traversal: SignalTraversalReplacementObservation {
                scratch_lease: self.traversal.scratch_lease,
                node_buffer_a: self.traversal.scratch.node_buffer_a.clone(),
                node_buffer_b: self.traversal.scratch.node_buffer_b.clone(),
                planner_targets: self.traversal.scratch.planner_targets.clone(),
                planner_tasks: self.traversal.scratch.planner_tasks.clone(),
                planner_stages: self.traversal.scratch.planner_stages.clone(),
                topology_node_buffer: self.traversal.topology_node_buffer.clone(),
            },
            conditional_dependency_versions: self
                .conditional_dependency_versions
                .iter()
                .map(|(node, versions)| (*node, *versions))
                .collect(),
            authorization_policy_identities: self
                .authorization_policy_identities
                .iter()
                .copied()
                .collect(),
        }
    }

    pub(crate) fn replacement_clone_local_observation(&self) -> SignalGraphCloneLocalObservation {
        SignalGraphCloneLocalObservation {
            lifecycle_token_identity: Arc::as_ptr(&self.lifecycle_token) as usize,
            graph_instance_id: self.instance_id,
            schema_registry_identity: Arc::as_ptr(&self.schema_registry) as usize,
            aspect_lowering_owner_present: self.aspect_lowering_owner.is_some(),
            invalidation_readiness_epoch: self.invalidation_readiness_epoch,
            observation_active_generation: self.observation_sessions.active_generation(),
            observation_active_request: self.observation_sessions.active_request(),
            completed_execution_boundaries: self
                .observation_sessions
                .completed_execution_boundaries(),
            last_completion: self.observation_sessions.last_completion(),
            observation_cleanup_identity: self
                .observation_capture_cleanup
                .as_ref()
                .map(|cleanup| Arc::as_ptr(cleanup) as usize),
            performed_counters: self.invalidation_performed_counters.snapshot(),
            performed_work: self.invalidation_performed_work.snapshot(),
            pending_repeated_admissions: self
                .pending_repeated_invalidation_admissions
                .iter()
                .map(|(node, count)| (*node, *count))
                .collect(),
        }
    }

    pub(crate) fn populate_replacement_clone_contract(&mut self, node: NodeId) {
        self.traversal.scratch_lease = Some(ScratchLeaseKind::Churn);
        self.traversal.scratch.node_buffer_a.push(node);
        self.traversal.scratch.node_buffer_b.push(node);
        self.traversal.scratch.planner_targets.push(node);
        self.traversal.scratch.planner_tasks.push(EligibleTask {
            node,
            request_mode: EvaluationRequestMode::Default,
            direct_request: true,
            reason: TaskReason::RequestedTarget,
            admission: EligibleTaskAdmission::default(),
        });
        self.traversal.scratch.planner_stages.push(StageCursor {
            index: 0,
            start: 0,
            end: 1,
            barrier: Some(StageBarrier::StageBoundary),
        });
        self.traversal.topology_node_buffer.push(node);
        self.conditional_dependency_versions.insert(
            node,
            SignalConditionalVersionObservation::new(
                AspectMask::from([Aspect::new(0), Aspect::new(1), Aspect::new(2)]),
                AspectVersion::from_updates([
                    (Aspect::new(0), 3),
                    (Aspect::new(1), 5),
                    (Aspect::new(2), 8),
                ]),
            ),
        );
        self.authorization_policy_identities.insert([0xA5; 32]);
        self.claim_aspect_lowering_owner(&crate::data::aspect::SignalAspectLoweringOwner::fresh())
            .expect("replacement fixture claims one lowering owner");
        self.invalidation_readiness_epoch = 41;
        let observation_generation = self
            .observation_sessions
            .begin(SignalObservationRequest::counters().with_performed_work());
        assert_ne!(
            observation_generation, 0,
            "replacement fixture begins a live observation generation"
        );
        self.observation_sessions
            .record_completed_execution_boundary();
        self.observation_sessions
            .record_completion(SignalObservationCompletion::Completed);
        self.invalidation_performed_counters.set(
            crate::data::telemetry::InvalidationPerformedCounter::NodesEvaluated,
            17,
        );
        let binding = InvalidationWorkBindingAxes {
            graph_instance: self.instance_id,
            target: node,
            dependency_revision: DependencyRevision(29),
            origin: InvalidationOriginBinding::StructuralMutation { ordinal: 31 },
            readiness_epoch: InvalidationReadinessEpoch(41),
            stage_order: InvalidationStageOrder { stage: 2, order: 7 },
        };
        self.prepare_invalidation_performed_work(
            &binding,
            &mut crate::logic::evaluation::EvaluationWork::Ordinary,
        )
        .unwrap()
        .expect("fixture captures performed work")
        .commit();
        self.pending_repeated_invalidation_admissions
            .insert(node, 9);
    }
}
