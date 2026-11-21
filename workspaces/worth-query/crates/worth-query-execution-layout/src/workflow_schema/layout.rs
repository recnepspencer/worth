use worth_foundational::facade::AspectFieldLocator;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::indexes::DerivedIndexId;

/// Retained installed vocabulary; request phases share its immutable storage.
/// Keeping the vocabulary inline would copy every locator through read sets,
/// effect programs and their result variants even when no workflow is bound.
#[derive(Clone, Debug)]
pub struct WorthQueryWorkflowLayout {
    fields: std::sync::Arc<WorkflowLayoutFields>,
}

impl WorthQueryWorkflowLayout {
    pub(super) fn new(fields: WorkflowLayoutFields) -> Self {
        Self {
            fields: std::sync::Arc::new(fields),
        }
    }

    /// Index identities are filled only during installation. A cloned layout
    /// keeps its own installed interpretation if another installation changes it.
    pub(super) fn installation_fields_mut(&mut self) -> &mut WorkflowLayoutFields {
        std::sync::Arc::make_mut(&mut self.fields)
    }
}

impl std::ops::Deref for WorthQueryWorkflowLayout {
    type Target = WorkflowLayoutFields;

    fn deref(&self) -> &Self::Target {
        &self.fields
    }
}

#[derive(Clone, Debug)]
pub struct WorkflowLayoutFields {
    pub captured_input: WorkflowCapturedInputLayout,
    pub operation_output: WorkflowOperationOutputLayout,
    pub lineage: WorkflowLineageLayout,
    pub definition: WorkflowDefinitionLayout,
    pub node: WorkflowNodeLayout,
    pub connection: WorkflowConnectionLayout,
    pub instance: WorkflowInstanceLayout,
    pub transition: WorkflowTransitionLayout,
    pub proposal: WorkflowProposalLayout,
    pub proposal_coverage: WorkflowProposalCoverageLayout,
    pub assessment_evidence: WorkflowAssessmentEvidenceLayout,
    pub approval: WorkflowApprovalLayout,
    pub evidence_dependency: WorkflowEvidenceDependencyLayout,
    pub current_definition_relation: KindId,
    pub lineage_definition_relation: KindId,
    pub definition_node_relation: KindId,
    pub definition_connection_relation: KindId,
    pub definition_start_relation: KindId,
    pub connection_source_relation: KindId,
    pub connection_target_relation: KindId,
    pub instance_lineage_relation: KindId,
    pub live_instance_lineage_relation: KindId,
    pub instance_definition_relation: KindId,
    pub instance_subject_relation: KindId,
    pub instance_transition_relation: KindId,
    pub transition_node_relation: KindId,
    pub transition_proposal_relation: KindId,
    pub proposal_coverage_relation: KindId,
    pub transition_assessment_evidence_relation: KindId,
    pub transition_approval_relation: KindId,
    pub approval_proposal_relation: KindId,
    pub approval_evidence_relation: KindId,
    pub evidence_dependency_relation: KindId,
    /// Successor instance to the instance its migration ended.
    pub instance_migrated_from_relation: KindId,
    /// Fresh lifecycle episode to its completed predecessor; not migrated effects.
    pub instance_succeeded_from_relation: KindId,
    /// Successor instance to each receipted transition it inherits as history.
    pub instance_prior_effect_relation: KindId,
}

#[derive(Clone, Debug)]
pub struct WorkflowCapturedInputLayout {
    pub entity_kind: KindId,
    pub membership: KindId,
    pub resource: KindId,
    pub identity_index_id: DerivedIndexId,
    pub identity: AspectFieldLocator,
    pub node: AspectFieldLocator,
    pub binding: AspectFieldLocator,
    pub input_type: AspectFieldLocator,
    pub input: AspectFieldLocator,
    pub key: AspectFieldLocator,
    pub source_kind: AspectFieldLocator,
    pub source: AspectFieldLocator,
    pub source_partition: AspectFieldLocator,
    pub preconditions: AspectFieldLocator,
}

#[derive(Clone, Debug)]
pub struct WorkflowOperationOutputLayout {
    pub entity_kind: KindId,
    pub role: AspectFieldLocator,
    pub entity: AspectFieldLocator,
    pub posture: AspectFieldLocator,
    pub membership: KindId,
    pub target: KindId,
}

#[derive(Clone, Debug)]
pub struct WorkflowProposalCoverageLayout {
    pub entity_kind: KindId,
    pub identity: AspectFieldLocator,
    pub selector: AspectFieldLocator,
    pub subject_partition: AspectFieldLocator,
    pub subject_slot: AspectFieldLocator,
    pub subject_generation: AspectFieldLocator,
}

#[derive(Clone, Debug)]
pub struct WorkflowEvidenceDependencyLayout {
    pub entity_kind: KindId,
    pub fact_kind: AspectFieldLocator,
    pub entity_partition: AspectFieldLocator,
    pub entity_slot: AspectFieldLocator,
    pub entity_generation: AspectFieldLocator,
    pub native_entity_kind: AspectFieldLocator,
    pub aspect: AspectFieldLocator,
    pub field: AspectFieldLocator,
    pub field_presence: AspectFieldLocator,
    pub relation_kind: AspectFieldLocator,
    pub direction: AspectFieldLocator,
    pub native_revision: AspectFieldLocator,
    pub comparison_work_limit: AspectFieldLocator,
}

#[derive(Clone, Debug)]
pub struct WorkflowApprovalLayout {
    pub entity_kind: KindId,
    pub identity: AspectFieldLocator,
    pub protocol_version: AspectFieldLocator,
    pub instance_identity: AspectFieldLocator,
    pub definition_content_identity: AspectFieldLocator,
    pub program_revision: AspectFieldLocator,
    pub decision: AspectFieldLocator,
    pub approval_operation: AspectFieldLocator,
    pub target_operation: AspectFieldLocator,
    pub installed_capability_identity: AspectFieldLocator,
    pub approver_partition: AspectFieldLocator,
    pub approver_slot: AspectFieldLocator,
    pub approver_generation: AspectFieldLocator,
    pub scope_partition: AspectFieldLocator,
    pub scope_slot: AspectFieldLocator,
    pub scope_generation: AspectFieldLocator,
    pub grant_partition: AspectFieldLocator,
    pub grant_slot: AspectFieldLocator,
    pub grant_generation: AspectFieldLocator,
    pub authorization_decision: AspectFieldLocator,
    pub authorization_request: AspectFieldLocator,
    pub authorization_principal: AspectFieldLocator,
    pub capability_authority_identity: AspectFieldLocator,
    pub authorization_lineage: AspectFieldLocator,
    pub authorization_support: AspectFieldLocator,
    pub authorization_dependencies: AspectFieldLocator,
    pub action: AspectFieldLocator,
    pub purpose: AspectFieldLocator,
    pub validity_timeline: AspectFieldLocator,
    pub authorization_sample: AspectFieldLocator,
    pub expiry: AspectFieldLocator,
}

#[derive(Clone, Debug)]
pub struct WorkflowAssessmentEvidenceLayout {
    pub entity_kind: KindId,
    pub identity: AspectFieldLocator,
    pub protocol_version: AspectFieldLocator,
    pub producer: AspectFieldLocator,
    pub family: AspectFieldLocator,
    pub query: AspectFieldLocator,
    pub parameter_type: AspectFieldLocator,
    pub result_type: AspectFieldLocator,
    pub binding: AspectFieldLocator,
    pub subject_partition: AspectFieldLocator,
    pub subject_slot: AspectFieldLocator,
    pub subject_generation: AspectFieldLocator,
    pub proposal_identity: AspectFieldLocator,
    pub coverage_identity: AspectFieldLocator,
    pub source_identity: AspectFieldLocator,
    pub passing: AspectFieldLocator,
    pub publication_identity: AspectFieldLocator,
    pub output_content_identity: AspectFieldLocator,
    /// The program revision the evidence was collected under. Program
    /// adoption can carry an instance forward; evidence from an earlier
    /// revision is never reused under a later one.
    pub program_revision: AspectFieldLocator,
    /// The logical bytes this evidence and its dependency facts retain,
    /// charged against the lineage's evidence ceiling when it settled.
    pub retained_bytes: AspectFieldLocator,
}

#[derive(Clone, Debug)]
pub struct WorkflowLineageLayout {
    pub entity_kind: KindId,
    pub identity: AspectFieldLocator,
    pub spec: AspectFieldLocator,
    pub protocol_version: AspectFieldLocator,
    pub identity_index_id: DerivedIndexId,
}

#[derive(Clone, Debug)]
pub struct WorkflowDefinitionLayout {
    pub entity_kind: KindId,
    pub content_identity: AspectFieldLocator,
    /// The revision the definition was published under; never rewritten.
    pub program_revision: AspectFieldLocator,
    /// The latest revision adoption carried the definition to, when any.
    pub carried_revision: AspectFieldLocator,
    pub maximum_nodes: AspectFieldLocator,
    pub maximum_connections: AspectFieldLocator,
    pub maximum_effects: AspectFieldLocator,
    pub global_actions: AspectFieldLocator,
    pub maximum_component_occurrences: AspectFieldLocator,
    pub maximum_component_depth: AspectFieldLocator,
    pub maximum_node_provenance: AspectFieldLocator,
    pub maximum_connection_provenance: AspectFieldLocator,
    pub maximum_port_provenance: AspectFieldLocator,
    pub maximum_canonical_bytes: AspectFieldLocator,
    /// The declared total deadline in milliseconds; absent when none.
    pub total_deadline_milliseconds: AspectFieldLocator,
    pub content_identity_index_id: DerivedIndexId,
}

#[derive(Clone, Debug)]
pub struct WorkflowNodeLayout {
    pub entity_kind: KindId,
    pub path: AspectFieldLocator,
    pub kind: AspectFieldLocator,
    pub member: AspectFieldLocator,
    pub input_type: AspectFieldLocator,
    pub operation_binding: AspectFieldLocator,
    pub workflow_execution_requirements: AspectFieldLocator,
    pub inbound_origin: AspectFieldLocator,
    pub inbound_contract: AspectFieldLocator,
    pub inbound_wait: AspectFieldLocator,
    pub parameter_type: AspectFieldLocator,
    pub result_type: AspectFieldLocator,
    pub assessment_binding: AspectFieldLocator,
    pub assessment_subject: AspectFieldLocator,
    pub assessment_applicability_relation: AspectFieldLocator,
    pub assessment_applicability_from: AspectFieldLocator,
    pub assessment_applicability_to: AspectFieldLocator,
    pub condition_binding: AspectFieldLocator,
    /// Present exactly on expression conditions; see `definition::condition`.
    pub condition_operands: AspectFieldLocator,
    pub capability_type: AspectFieldLocator,
    pub approval_operation: AspectFieldLocator,
    pub approval_capability_identity: AspectFieldLocator,
    pub requires_authority: AspectFieldLocator,
}

#[derive(Clone, Debug)]
pub struct WorkflowConnectionLayout {
    pub entity_kind: KindId,
    pub family: AspectFieldLocator,
    pub variant: AspectFieldLocator,
    pub retry_reason: AspectFieldLocator,
    pub retry_maximum_attempts: AspectFieldLocator,
    pub back_action: WorkflowBackActionLayout,
}

#[derive(Clone, Debug)]
pub struct WorkflowBackActionLayout {
    pub operation: AspectFieldLocator,
    pub input_type: AspectFieldLocator,
    pub binding: AspectFieldLocator,
    pub requirements: AspectFieldLocator,
}

#[derive(Clone, Debug)]
pub struct WorkflowInstanceLayout {
    pub start_advancer: crate::workflow_actor_fields::WorkflowActorFields,
    pub ending_actor: crate::workflow_actor_fields::WorkflowEndingActorFields,
    pub entity_kind: KindId,
    pub identity: AspectFieldLocator,
    pub protocol_version: AspectFieldLocator,
    pub branch_occurrence: AspectFieldLocator,
    /// Immutable canonical revision of this occurrence's genuine start.
    pub started_program_revision: AspectFieldLocator,
    /// Mutable execution revision; Carry may advance it.
    pub program_revision: AspectFieldLocator,
    pub definition_content_identity: AspectFieldLocator,
    pub subject_partition: AspectFieldLocator,
    pub subject_slot: AspectFieldLocator,
    pub subject_generation: AspectFieldLocator,
    pub state: AspectFieldLocator,
    /// Where a migration successor began, when not at its definition's start.
    pub resume_node_path: AspectFieldLocator,
    /// The intent that explicitly cancelled the instance; absent when program
    /// adoption cancelled it or it never ended by cancellation.
    pub cancellation_identity: AspectFieldLocator,
    /// The transition whose external operation last committed into its
    /// owner's custody. The owner settles it later; until a transition with
    /// this identity settles, the instance cannot be cancelled.
    pub owner_custody: AspectFieldLocator,
    /// Steps the instance's migration sources took before it; absent when
    /// none. One lineage spends one step budget, so a successor never
    /// starts it afresh.
    pub inherited_steps: AspectFieldLocator,
    /// The Unix-epoch millisecond at which the lineage's earliest total
    /// deadline elapses; absent when no definition it ran declared one.
    pub deadline: AspectFieldLocator,
    /// Assessment evidence bytes the instance's migration sources retained
    /// before it; absent when none. One lineage retains one evidence budget.
    pub inherited_evidence_bytes: AspectFieldLocator,
    pub identity_index_id: DerivedIndexId,
}

#[derive(Clone, Debug)]
pub struct WorkflowTransitionLayout {
    pub identity_index_id: DerivedIndexId,
    pub output_binding: AspectFieldLocator,
    pub output_count: AspectFieldLocator,
    pub advancer: crate::workflow_actor_fields::WorkflowActorFields,
    pub executor: crate::workflow_actor_fields::WorkflowActorFields,
    pub entity_kind: KindId,
    pub identity: AspectFieldLocator,
    pub protocol_version: AspectFieldLocator,
    pub occurrence: AspectFieldLocator,
    pub outcome: AspectFieldLocator,
    pub operation_receipt_identity: AspectFieldLocator,
    pub live_membership_partition: AspectFieldLocator,
    pub live_membership_slot: AspectFieldLocator,
    pub live_membership_generation: AspectFieldLocator,
}

#[derive(Clone, Debug)]
pub struct WorkflowProposalLayout {
    pub captured_identity: AspectFieldLocator,
    pub captured_count: AspectFieldLocator,
    pub proposer: crate::workflow_actor_fields::WorkflowActorFields,
    pub entity_kind: KindId,
    pub identity: AspectFieldLocator,
    pub protocol_version: AspectFieldLocator,
    pub operation: AspectFieldLocator,
    pub input_type: AspectFieldLocator,
    pub input_identity: AspectFieldLocator,
    pub source_identity: AspectFieldLocator,
    pub node_path: AspectFieldLocator,
    pub coverage_count: AspectFieldLocator,
    pub identity_index_id: DerivedIndexId,
}

mod court_doors_for_test;
