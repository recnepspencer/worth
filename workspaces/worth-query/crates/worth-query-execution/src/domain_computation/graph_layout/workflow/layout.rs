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
    pub(in crate::domain_computation) captured_input: WorkflowCapturedInputLayout,
    pub operation_output: WorkflowOperationOutputLayout,
    pub(in crate::domain_computation) lineage: WorkflowLineageLayout,
    pub(in crate::domain_computation) definition: WorkflowDefinitionLayout,
    pub(in crate::domain_computation) node: WorkflowNodeLayout,
    pub(in crate::domain_computation) connection: WorkflowConnectionLayout,
    pub(in crate::domain_computation) instance: WorkflowInstanceLayout,
    pub(in crate::domain_computation) transition: WorkflowTransitionLayout,
    pub(in crate::domain_computation) proposal: WorkflowProposalLayout,
    pub(in crate::domain_computation) proposal_coverage: WorkflowProposalCoverageLayout,
    pub assessment_evidence: WorkflowAssessmentEvidenceLayout,
    pub(in crate::domain_computation) approval: WorkflowApprovalLayout,
    pub evidence_dependency: WorkflowEvidenceDependencyLayout,
    pub(in crate::domain_computation) current_definition_relation: KindId,
    pub(in crate::domain_computation) lineage_definition_relation: KindId,
    pub(in crate::domain_computation) definition_node_relation: KindId,
    pub(in crate::domain_computation) definition_connection_relation: KindId,
    pub(in crate::domain_computation) definition_start_relation: KindId,
    pub(in crate::domain_computation) connection_source_relation: KindId,
    pub(in crate::domain_computation) connection_target_relation: KindId,
    pub(in crate::domain_computation) instance_lineage_relation: KindId,
    pub(in crate::domain_computation) live_instance_lineage_relation: KindId,
    pub(in crate::domain_computation) instance_definition_relation: KindId,
    pub(in crate::domain_computation) instance_subject_relation: KindId,
    pub(in crate::domain_computation) instance_transition_relation: KindId,
    pub(in crate::domain_computation) transition_node_relation: KindId,
    pub(in crate::domain_computation) transition_proposal_relation: KindId,
    pub(in crate::domain_computation) proposal_coverage_relation: KindId,
    pub(in crate::domain_computation) transition_assessment_evidence_relation: KindId,
    pub(in crate::domain_computation) transition_approval_relation: KindId,
    pub(in crate::domain_computation) approval_proposal_relation: KindId,
    pub(in crate::domain_computation) approval_evidence_relation: KindId,
    pub(in crate::domain_computation) evidence_dependency_relation: KindId,
    /// Successor instance to the instance its migration ended.
    pub(in crate::domain_computation) instance_migrated_from_relation: KindId,
    /// Fresh lifecycle episode to its completed predecessor; not migrated effects.
    pub(in crate::domain_computation) instance_succeeded_from_relation: KindId,
    /// Successor instance to each receipted transition it inherits as history.
    pub(in crate::domain_computation) instance_prior_effect_relation: KindId,
}

#[derive(Clone, Debug)]
pub(in crate::domain_computation) struct WorkflowCapturedInputLayout {
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
    pub(in crate::domain_computation) entity_kind: KindId,
    pub(in crate::domain_computation) role: AspectFieldLocator,
    pub(in crate::domain_computation) entity: AspectFieldLocator,
    pub(in crate::domain_computation) posture: AspectFieldLocator,
    pub(in crate::domain_computation) membership: KindId,
    pub(in crate::domain_computation) target: KindId,
}

#[derive(Clone, Debug)]
pub(in crate::domain_computation) struct WorkflowProposalCoverageLayout {
    pub(in crate::domain_computation) entity_kind: KindId,
    pub(in crate::domain_computation) identity: AspectFieldLocator,
    pub(in crate::domain_computation) selector: AspectFieldLocator,
    pub(in crate::domain_computation) subject_partition: AspectFieldLocator,
    pub(in crate::domain_computation) subject_slot: AspectFieldLocator,
    pub(in crate::domain_computation) subject_generation: AspectFieldLocator,
}

#[derive(Clone, Debug)]
pub struct WorkflowEvidenceDependencyLayout {
    pub(in crate::domain_computation) entity_kind: KindId,
    pub(in crate::domain_computation) fact_kind: AspectFieldLocator,
    pub(in crate::domain_computation) entity_partition: AspectFieldLocator,
    pub(in crate::domain_computation) entity_slot: AspectFieldLocator,
    pub(in crate::domain_computation) entity_generation: AspectFieldLocator,
    pub(in crate::domain_computation) native_entity_kind: AspectFieldLocator,
    pub(in crate::domain_computation) aspect: AspectFieldLocator,
    pub(in crate::domain_computation) field: AspectFieldLocator,
    pub(in crate::domain_computation) field_presence: AspectFieldLocator,
    pub(in crate::domain_computation) relation_kind: AspectFieldLocator,
    pub(in crate::domain_computation) direction: AspectFieldLocator,
    pub(in crate::domain_computation) native_revision: AspectFieldLocator,
    pub(in crate::domain_computation) comparison_work_limit: AspectFieldLocator,
}

#[derive(Clone, Debug)]
pub struct WorkflowApprovalLayout {
    pub(in crate::domain_computation) entity_kind: KindId,
    pub(in crate::domain_computation) identity: AspectFieldLocator,
    pub(in crate::domain_computation) protocol_version: AspectFieldLocator,
    pub(in crate::domain_computation) instance_identity: AspectFieldLocator,
    pub(in crate::domain_computation) definition_content_identity: AspectFieldLocator,
    pub(in crate::domain_computation) program_revision: AspectFieldLocator,
    pub(in crate::domain_computation) decision: AspectFieldLocator,
    pub(in crate::domain_computation) approval_operation: AspectFieldLocator,
    pub(in crate::domain_computation) target_operation: AspectFieldLocator,
    pub(in crate::domain_computation) installed_capability_identity: AspectFieldLocator,
    pub(in crate::domain_computation) approver_partition: AspectFieldLocator,
    pub(in crate::domain_computation) approver_slot: AspectFieldLocator,
    pub(in crate::domain_computation) approver_generation: AspectFieldLocator,
    pub(in crate::domain_computation) scope_partition: AspectFieldLocator,
    pub(in crate::domain_computation) scope_slot: AspectFieldLocator,
    pub(in crate::domain_computation) scope_generation: AspectFieldLocator,
    pub(in crate::domain_computation) grant_partition: AspectFieldLocator,
    pub(in crate::domain_computation) grant_slot: AspectFieldLocator,
    pub(in crate::domain_computation) grant_generation: AspectFieldLocator,
    pub(in crate::domain_computation) authorization_decision: AspectFieldLocator,
    pub(in crate::domain_computation) authorization_request: AspectFieldLocator,
    pub(in crate::domain_computation) authorization_principal: AspectFieldLocator,
    pub(in crate::domain_computation) capability_authority_identity: AspectFieldLocator,
    pub(in crate::domain_computation) authorization_lineage: AspectFieldLocator,
    pub(in crate::domain_computation) authorization_support: AspectFieldLocator,
    pub(in crate::domain_computation) authorization_dependencies: AspectFieldLocator,
    pub(in crate::domain_computation) action: AspectFieldLocator,
    pub(in crate::domain_computation) purpose: AspectFieldLocator,
    pub(in crate::domain_computation) validity_timeline: AspectFieldLocator,
    pub(in crate::domain_computation) authorization_sample: AspectFieldLocator,
    pub(in crate::domain_computation) expiry: AspectFieldLocator,
}

#[derive(Clone, Debug)]
pub struct WorkflowAssessmentEvidenceLayout {
    pub(in crate::domain_computation) entity_kind: KindId,
    pub(in crate::domain_computation) identity: AspectFieldLocator,
    pub(in crate::domain_computation) protocol_version: AspectFieldLocator,
    pub(in crate::domain_computation) producer: AspectFieldLocator,
    pub(in crate::domain_computation) family: AspectFieldLocator,
    pub(in crate::domain_computation) query: AspectFieldLocator,
    pub(in crate::domain_computation) parameter_type: AspectFieldLocator,
    pub(in crate::domain_computation) result_type: AspectFieldLocator,
    pub(in crate::domain_computation) binding: AspectFieldLocator,
    pub(in crate::domain_computation) subject_partition: AspectFieldLocator,
    pub(in crate::domain_computation) subject_slot: AspectFieldLocator,
    pub(in crate::domain_computation) subject_generation: AspectFieldLocator,
    pub(in crate::domain_computation) proposal_identity: AspectFieldLocator,
    pub(in crate::domain_computation) coverage_identity: AspectFieldLocator,
    pub(in crate::domain_computation) source_identity: AspectFieldLocator,
    pub(in crate::domain_computation) passing: AspectFieldLocator,
    pub(in crate::domain_computation) publication_identity: AspectFieldLocator,
    pub(in crate::domain_computation) output_content_identity: AspectFieldLocator,
    /// The program revision the evidence was collected under. Program
    /// adoption can carry an instance forward; evidence from an earlier
    /// revision is never reused under a later one.
    pub(in crate::domain_computation) program_revision: AspectFieldLocator,
    /// The logical bytes this evidence and its dependency facts retain,
    /// charged against the lineage's evidence ceiling when it settled.
    pub(in crate::domain_computation) retained_bytes: AspectFieldLocator,
}

#[derive(Clone, Debug)]
pub struct WorkflowLineageLayout {
    pub(in crate::domain_computation) entity_kind: KindId,
    pub(in crate::domain_computation) identity: AspectFieldLocator,
    pub(in crate::domain_computation) spec: AspectFieldLocator,
    pub(in crate::domain_computation) protocol_version: AspectFieldLocator,
    pub(in crate::domain_computation) identity_index_id: DerivedIndexId,
}

#[derive(Clone, Debug)]
pub struct WorkflowDefinitionLayout {
    pub(in crate::domain_computation) entity_kind: KindId,
    pub(in crate::domain_computation) content_identity: AspectFieldLocator,
    /// The revision the definition was published under; never rewritten.
    pub(in crate::domain_computation) program_revision: AspectFieldLocator,
    /// The latest revision adoption carried the definition to, when any.
    pub(in crate::domain_computation) carried_revision: AspectFieldLocator,
    pub(in crate::domain_computation) maximum_nodes: AspectFieldLocator,
    pub(in crate::domain_computation) maximum_connections: AspectFieldLocator,
    pub(in crate::domain_computation) maximum_effects: AspectFieldLocator,
    pub(in crate::domain_computation) global_actions: AspectFieldLocator,
    pub(in crate::domain_computation) maximum_component_occurrences: AspectFieldLocator,
    pub(in crate::domain_computation) maximum_component_depth: AspectFieldLocator,
    pub(in crate::domain_computation) maximum_node_provenance: AspectFieldLocator,
    pub(in crate::domain_computation) maximum_connection_provenance: AspectFieldLocator,
    pub(in crate::domain_computation) maximum_port_provenance: AspectFieldLocator,
    pub(in crate::domain_computation) maximum_canonical_bytes: AspectFieldLocator,
    /// The declared total deadline in milliseconds; absent when none.
    pub(in crate::domain_computation) total_deadline_milliseconds: AspectFieldLocator,
    pub(in crate::domain_computation) content_identity_index_id: DerivedIndexId,
}

#[derive(Clone, Debug)]
pub struct WorkflowNodeLayout {
    pub(in crate::domain_computation) entity_kind: KindId,
    pub(in crate::domain_computation) path: AspectFieldLocator,
    pub(in crate::domain_computation) kind: AspectFieldLocator,
    pub(in crate::domain_computation) member: AspectFieldLocator,
    pub(in crate::domain_computation) input_type: AspectFieldLocator,
    pub(in crate::domain_computation) operation_binding: AspectFieldLocator,
    pub(in crate::domain_computation) workflow_execution_requirements: AspectFieldLocator,
    pub(in crate::domain_computation) inbound_origin: AspectFieldLocator,
    pub(in crate::domain_computation) inbound_contract: AspectFieldLocator,
    pub(in crate::domain_computation) inbound_wait: AspectFieldLocator,
    pub(in crate::domain_computation) parameter_type: AspectFieldLocator,
    pub(in crate::domain_computation) result_type: AspectFieldLocator,
    pub(in crate::domain_computation) assessment_binding: AspectFieldLocator,
    pub(in crate::domain_computation) assessment_subject: AspectFieldLocator,
    pub(in crate::domain_computation) assessment_applicability_relation: AspectFieldLocator,
    pub(in crate::domain_computation) assessment_applicability_from: AspectFieldLocator,
    pub(in crate::domain_computation) assessment_applicability_to: AspectFieldLocator,
    pub(in crate::domain_computation) condition_binding: AspectFieldLocator,
    /// Present exactly on expression conditions; see `definition::condition`.
    pub(in crate::domain_computation) condition_operands: AspectFieldLocator,
    pub(in crate::domain_computation) capability_type: AspectFieldLocator,
    pub(in crate::domain_computation) approval_operation: AspectFieldLocator,
    pub(in crate::domain_computation) approval_capability_identity: AspectFieldLocator,
    pub(in crate::domain_computation) requires_authority: AspectFieldLocator,
}

#[derive(Clone, Debug)]
pub(in crate::domain_computation) struct WorkflowConnectionLayout {
    pub(in crate::domain_computation) entity_kind: KindId,
    pub(in crate::domain_computation) family: AspectFieldLocator,
    pub(in crate::domain_computation) variant: AspectFieldLocator,
    pub(in crate::domain_computation) retry_reason: AspectFieldLocator,
    pub(in crate::domain_computation) retry_maximum_attempts: AspectFieldLocator,
    pub(in crate::domain_computation) back_action: WorkflowBackActionLayout,
}

#[derive(Clone, Debug)]
pub(in crate::domain_computation) struct WorkflowBackActionLayout {
    pub(in crate::domain_computation) operation: AspectFieldLocator,
    pub(in crate::domain_computation) input_type: AspectFieldLocator,
    pub(in crate::domain_computation) binding: AspectFieldLocator,
    pub(in crate::domain_computation) requirements: AspectFieldLocator,
}

#[derive(Clone, Debug)]
pub struct WorkflowInstanceLayout {
    pub(in crate::domain_computation) start_advancer:
        crate::domain_computation::graph_layout::workflow::actor_fields::WorkflowActorFields,
    pub(in crate::domain_computation) end_advancer:
        crate::domain_computation::graph_layout::workflow::actor_fields::WorkflowActorFields,
    pub(in crate::domain_computation) entity_kind: KindId,
    pub(in crate::domain_computation) identity: AspectFieldLocator,
    pub(in crate::domain_computation) protocol_version: AspectFieldLocator,
    pub(in crate::domain_computation) branch_occurrence: AspectFieldLocator,
    /// Immutable canonical revision of this occurrence's genuine start.
    pub(in crate::domain_computation) started_program_revision: AspectFieldLocator,
    /// Mutable execution revision; Carry may advance it.
    pub(in crate::domain_computation) program_revision: AspectFieldLocator,
    pub(in crate::domain_computation) definition_content_identity: AspectFieldLocator,
    pub(in crate::domain_computation) subject_partition: AspectFieldLocator,
    pub(in crate::domain_computation) subject_slot: AspectFieldLocator,
    pub(in crate::domain_computation) subject_generation: AspectFieldLocator,
    pub(in crate::domain_computation) state: AspectFieldLocator,
    /// Where a migration successor began, when not at its definition's start.
    pub(in crate::domain_computation) resume_node_path: AspectFieldLocator,
    /// The intent that explicitly cancelled the instance; absent when program
    /// adoption cancelled it or it never ended by cancellation.
    pub(in crate::domain_computation) cancellation_identity: AspectFieldLocator,
    /// The transition whose external operation last committed into its
    /// owner's custody. The owner settles it later; until a transition with
    /// this identity settles, the instance cannot be cancelled.
    pub(in crate::domain_computation) owner_custody: AspectFieldLocator,
    /// Steps the instance's migration sources took before it; absent when
    /// none. One lineage spends one step budget, so a successor never
    /// starts it afresh.
    pub(in crate::domain_computation) inherited_steps: AspectFieldLocator,
    /// The Unix-epoch millisecond at which the lineage's earliest total
    /// deadline elapses; absent when no definition it ran declared one.
    pub(in crate::domain_computation) deadline: AspectFieldLocator,
    /// Assessment evidence bytes the instance's migration sources retained
    /// before it; absent when none. One lineage retains one evidence budget.
    pub(in crate::domain_computation) inherited_evidence_bytes: AspectFieldLocator,
    pub(in crate::domain_computation) identity_index_id: DerivedIndexId,
}

#[derive(Clone, Debug)]
pub struct WorkflowTransitionLayout {
    pub(in crate::domain_computation) identity_index_id: DerivedIndexId,
    pub(in crate::domain_computation) output_binding: AspectFieldLocator,
    pub(in crate::domain_computation) output_count: AspectFieldLocator,
    pub advancer:
        crate::domain_computation::graph_layout::workflow::actor_fields::WorkflowActorFields,
    pub(in crate::domain_computation) executor:
        crate::domain_computation::graph_layout::workflow::actor_fields::WorkflowActorFields,
    pub(in crate::domain_computation) entity_kind: KindId,
    pub(in crate::domain_computation) identity: AspectFieldLocator,
    pub(in crate::domain_computation) protocol_version: AspectFieldLocator,
    pub(in crate::domain_computation) occurrence: AspectFieldLocator,
    pub(in crate::domain_computation) outcome: AspectFieldLocator,
    pub(in crate::domain_computation) operation_receipt_identity: AspectFieldLocator,
    pub(in crate::domain_computation) live_membership_partition: AspectFieldLocator,
    pub(in crate::domain_computation) live_membership_slot: AspectFieldLocator,
    pub(in crate::domain_computation) live_membership_generation: AspectFieldLocator,
}

#[derive(Clone, Debug)]
pub(in crate::domain_computation) struct WorkflowProposalLayout {
    pub(in crate::domain_computation) captured_identity: AspectFieldLocator,
    pub(in crate::domain_computation) captured_count: AspectFieldLocator,
    pub(in crate::domain_computation) proposer:
        crate::domain_computation::graph_layout::workflow::actor_fields::WorkflowActorFields,
    pub(in crate::domain_computation) entity_kind: KindId,
    pub(in crate::domain_computation) identity: AspectFieldLocator,
    pub(in crate::domain_computation) protocol_version: AspectFieldLocator,
    pub(in crate::domain_computation) operation: AspectFieldLocator,
    pub(in crate::domain_computation) input_type: AspectFieldLocator,
    pub(in crate::domain_computation) input_identity: AspectFieldLocator,
    pub(in crate::domain_computation) source_identity: AspectFieldLocator,
    pub(in crate::domain_computation) node_path: AspectFieldLocator,
    pub(in crate::domain_computation) coverage_count: AspectFieldLocator,
    pub(in crate::domain_computation) identity_index_id: DerivedIndexId,
}

mod court_doors_for_test;
