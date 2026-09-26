use worth_foundational::facade::AspectFieldLocator;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::indexes::DerivedIndexId;

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryWorkflowLayout {
    pub(in crate::domain_computation::primary_graph) lineage: WorkflowLineageLayout,
    pub(in crate::domain_computation::primary_graph) definition: WorkflowDefinitionLayout,
    pub(in crate::domain_computation::primary_graph) node: WorkflowNodeLayout,
    pub(in crate::domain_computation::primary_graph) connection: WorkflowConnectionLayout,
    pub(in crate::domain_computation::primary_graph) instance: WorkflowInstanceLayout,
    pub(in crate::domain_computation::primary_graph) transition: WorkflowTransitionLayout,
    pub(in crate::domain_computation::primary_graph) proposal: WorkflowProposalLayout,
    pub(in crate::domain_computation::primary_graph) proposal_coverage:
        WorkflowProposalCoverageLayout,
    pub(in crate::domain_computation::primary_graph) assessment_evidence:
        WorkflowAssessmentEvidenceLayout,
    pub(in crate::domain_computation::primary_graph) approval: WorkflowApprovalLayout,
    pub(in crate::domain_computation::primary_graph) evidence_dependency:
        WorkflowEvidenceDependencyLayout,
    pub(in crate::domain_computation::primary_graph) current_definition_relation: KindId,
    pub(in crate::domain_computation::primary_graph) lineage_definition_relation: KindId,
    pub(in crate::domain_computation::primary_graph) definition_node_relation: KindId,
    pub(in crate::domain_computation::primary_graph) definition_connection_relation: KindId,
    pub(in crate::domain_computation::primary_graph) definition_start_relation: KindId,
    pub(in crate::domain_computation::primary_graph) connection_source_relation: KindId,
    pub(in crate::domain_computation::primary_graph) connection_target_relation: KindId,
    pub(in crate::domain_computation::primary_graph) instance_lineage_relation: KindId,
    pub(in crate::domain_computation::primary_graph) live_instance_lineage_relation: KindId,
    pub(in crate::domain_computation::primary_graph) instance_definition_relation: KindId,
    pub(in crate::domain_computation::primary_graph) instance_transition_relation: KindId,
    pub(in crate::domain_computation::primary_graph) transition_node_relation: KindId,
    pub(in crate::domain_computation::primary_graph) transition_proposal_relation: KindId,
    pub(in crate::domain_computation::primary_graph) proposal_coverage_relation: KindId,
    pub(in crate::domain_computation::primary_graph) transition_assessment_evidence_relation:
        KindId,
    pub(in crate::domain_computation::primary_graph) transition_approval_relation: KindId,
    pub(in crate::domain_computation::primary_graph) approval_proposal_relation: KindId,
    pub(in crate::domain_computation::primary_graph) approval_evidence_relation: KindId,
    pub(in crate::domain_computation::primary_graph) evidence_dependency_relation: KindId,
}

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct WorkflowProposalCoverageLayout {
    pub(in crate::domain_computation::primary_graph) entity_kind: KindId,
    pub(in crate::domain_computation::primary_graph) identity: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) selector: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) subject_partition: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) subject_slot: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) subject_generation: AspectFieldLocator,
}

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct WorkflowEvidenceDependencyLayout {
    pub(in crate::domain_computation::primary_graph) entity_kind: KindId,
    pub(in crate::domain_computation::primary_graph) fact_kind: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) entity_partition: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) entity_slot: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) entity_generation: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) aspect: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) field: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) field_presence: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) relation_kind: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) direction: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) native_revision: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) comparison_work_limit: AspectFieldLocator,
}

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct WorkflowApprovalLayout {
    pub(in crate::domain_computation::primary_graph) entity_kind: KindId,
    pub(in crate::domain_computation::primary_graph) identity: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) protocol_version: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) instance_identity: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) definition_content_identity:
        AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) program_revision: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) decision: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) approval_operation: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) target_operation: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) installed_capability_identity:
        AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) approver_partition: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) approver_slot: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) approver_generation: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) scope_partition: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) scope_slot: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) scope_generation: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) grant_partition: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) grant_slot: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) grant_generation: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) authorization_decision: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) authorization_request: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) authorization_principal: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) capability_authority_identity:
        AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) authorization_lineage: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) authorization_support: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) authorization_dependencies: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) action: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) purpose: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) validity_timeline: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) authorization_sample: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) expiry: AspectFieldLocator,
}

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct WorkflowAssessmentEvidenceLayout {
    pub(in crate::domain_computation::primary_graph) entity_kind: KindId,
    pub(in crate::domain_computation::primary_graph) identity: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) protocol_version: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) producer: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) family: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) query: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) parameter_type: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) result_type: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) binding: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) subject_partition: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) subject_slot: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) subject_generation: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) proposal_identity: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) coverage_identity: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) source_identity: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) passing: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) publication_identity: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) output_content_identity: AspectFieldLocator,
}

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct WorkflowLineageLayout {
    pub(in crate::domain_computation::primary_graph) entity_kind: KindId,
    pub(in crate::domain_computation::primary_graph) identity: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) spec: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) protocol_version: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) identity_index_id: DerivedIndexId,
}

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct WorkflowDefinitionLayout {
    pub(in crate::domain_computation::primary_graph) entity_kind: KindId,
    pub(in crate::domain_computation::primary_graph) content_identity: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) program_revision: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) maximum_nodes: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) maximum_connections: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) maximum_effects: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) maximum_component_occurrences:
        AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) maximum_component_depth: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) maximum_node_provenance: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) maximum_connection_provenance:
        AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) maximum_port_provenance: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) maximum_canonical_bytes: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) content_identity_index_id: DerivedIndexId,
}

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct WorkflowNodeLayout {
    pub(in crate::domain_computation::primary_graph) entity_kind: KindId,
    pub(in crate::domain_computation::primary_graph) path: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) kind: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) member: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) input_type: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) operation_binding: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) parameter_type: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) result_type: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) assessment_binding: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) assessment_subject: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) assessment_applicability_relation:
        AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) assessment_applicability_from:
        AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) assessment_applicability_to:
        AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) condition_binding: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) capability_type: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) approval_operation: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) approval_capability_identity:
        AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) requires_authority: AspectFieldLocator,
}

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct WorkflowConnectionLayout {
    pub(in crate::domain_computation::primary_graph) entity_kind: KindId,
    pub(in crate::domain_computation::primary_graph) family: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) variant: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) retry_reason: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) retry_maximum_attempts: AspectFieldLocator,
}

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct WorkflowInstanceLayout {
    pub(in crate::domain_computation::primary_graph) entity_kind: KindId,
    pub(in crate::domain_computation::primary_graph) identity: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) protocol_version: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) branch_occurrence: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) program_revision: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) definition_content_identity:
        AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) subject_partition: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) subject_slot: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) subject_generation: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) state: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) identity_index_id: DerivedIndexId,
}

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct WorkflowTransitionLayout {
    pub(in crate::domain_computation::primary_graph) entity_kind: KindId,
    pub(in crate::domain_computation::primary_graph) identity: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) protocol_version: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) occurrence: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) outcome: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) operation_receipt_identity: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) live_membership_partition: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) live_membership_slot: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) live_membership_generation: AspectFieldLocator,
}

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct WorkflowProposalLayout {
    pub(in crate::domain_computation::primary_graph) entity_kind: KindId,
    pub(in crate::domain_computation::primary_graph) identity: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) protocol_version: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) operation: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) input_type: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) input_identity: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) source_identity: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) node_path: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) coverage_count: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) identity_index_id: DerivedIndexId,
}
