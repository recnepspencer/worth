//! Sealed test doors over this module's private members.
use super::*;

impl WorkflowLayoutFields {
    #[doc(hidden)]
    pub fn approval_for_test(&self) -> &WorkflowApprovalLayout {
        &self.approval
    }
}

impl WorkflowLayoutFields {
    #[doc(hidden)]
    pub fn definition_for_test(&self) -> &WorkflowDefinitionLayout {
        &self.definition
    }
}

impl WorkflowLayoutFields {
    #[doc(hidden)]
    pub fn definition_node_relation_for_test(&self) -> &KindId {
        &self.definition_node_relation
    }
}

impl WorkflowLayoutFields {
    #[doc(hidden)]
    pub fn definition_start_relation_for_test(&self) -> &KindId {
        &self.definition_start_relation
    }
}

impl WorkflowAssessmentEvidenceLayout {
    #[doc(hidden)]
    pub fn entity_kind_for_test(&self) -> &KindId {
        &self.entity_kind
    }
}

impl WorkflowEvidenceDependencyLayout {
    #[doc(hidden)]
    pub fn entity_kind_for_test(&self) -> &KindId {
        &self.entity_kind
    }
}

impl WorkflowOperationOutputLayout {
    #[doc(hidden)]
    pub fn entity_kind_for_test(&self) -> &KindId {
        &self.entity_kind
    }
}

impl WorkflowLayoutFields {
    #[doc(hidden)]
    pub fn evidence_dependency_relation_for_test(&self) -> &KindId {
        &self.evidence_dependency_relation
    }
}

impl WorkflowEvidenceDependencyLayout {
    #[doc(hidden)]
    pub fn fact_kind_for_test(&self) -> &AspectFieldLocator {
        &self.fact_kind
    }
}

impl WorkflowEvidenceDependencyLayout {
    #[doc(hidden)]
    pub fn field_presence_for_test(&self) -> &AspectFieldLocator {
        &self.field_presence
    }
}

impl WorkflowAssessmentEvidenceLayout {
    #[doc(hidden)]
    pub fn identity_for_test(&self) -> &AspectFieldLocator {
        &self.identity
    }
}

impl WorkflowLayoutFields {
    #[doc(hidden)]
    pub fn instance_for_test(&self) -> &WorkflowInstanceLayout {
        &self.instance
    }
}

impl WorkflowLayoutFields {
    #[doc(hidden)]
    pub fn instance_definition_relation_for_test(&self) -> &KindId {
        &self.instance_definition_relation
    }
}

impl WorkflowLayoutFields {
    #[doc(hidden)]
    pub fn instance_lineage_relation_for_test(&self) -> &KindId {
        &self.instance_lineage_relation
    }
}

impl WorkflowLayoutFields {
    #[doc(hidden)]
    pub fn instance_transition_relation_for_test(&self) -> &KindId {
        &self.instance_transition_relation
    }
}

impl WorkflowLayoutFields {
    #[doc(hidden)]
    pub fn lineage_for_test(&self) -> &WorkflowLineageLayout {
        &self.lineage
    }
}

impl WorkflowLayoutFields {
    #[doc(hidden)]
    pub fn lineage_definition_relation_for_test(&self) -> &KindId {
        &self.lineage_definition_relation
    }
}

impl WorkflowOperationOutputLayout {
    #[doc(hidden)]
    pub fn membership_for_test(&self) -> &KindId {
        &self.membership
    }
}

impl WorkflowEvidenceDependencyLayout {
    #[doc(hidden)]
    pub fn native_entity_kind_for_test(&self) -> &AspectFieldLocator {
        &self.native_entity_kind
    }
}

impl WorkflowEvidenceDependencyLayout {
    #[doc(hidden)]
    pub fn native_revision_for_test(&self) -> &AspectFieldLocator {
        &self.native_revision
    }
}

impl WorkflowLayoutFields {
    #[doc(hidden)]
    pub fn node_for_test(&self) -> &WorkflowNodeLayout {
        &self.node
    }
}

impl WorkflowOperationOutputLayout {
    #[doc(hidden)]
    pub fn posture_for_test(&self) -> &AspectFieldLocator {
        &self.posture
    }
}

impl WorkflowAssessmentEvidenceLayout {
    #[doc(hidden)]
    pub fn publication_identity_for_test(&self) -> &AspectFieldLocator {
        &self.publication_identity
    }
}

impl WorkflowAssessmentEvidenceLayout {
    #[doc(hidden)]
    pub fn retained_bytes_for_test(&self) -> &AspectFieldLocator {
        &self.retained_bytes
    }
}

impl WorkflowOperationOutputLayout {
    #[doc(hidden)]
    pub fn role_for_test(&self) -> &AspectFieldLocator {
        &self.role
    }
}

impl WorkflowOperationOutputLayout {
    #[doc(hidden)]
    pub fn target_for_test(&self) -> &KindId {
        &self.target
    }
}

impl WorkflowLayoutFields {
    #[doc(hidden)]
    pub fn transition_for_test(&self) -> &WorkflowTransitionLayout {
        &self.transition
    }
}

impl WorkflowLayoutFields {
    #[doc(hidden)]
    pub fn transition_node_relation_for_test(&self) -> &KindId {
        &self.transition_node_relation
    }
}

impl WorkflowApprovalLayout {
    #[doc(hidden)]
    pub fn authorization_dependencies_for_test(&self) -> &AspectFieldLocator {
        &self.authorization_dependencies
    }
}

impl WorkflowApprovalLayout {
    #[doc(hidden)]
    pub fn authorization_lineage_for_test(&self) -> &AspectFieldLocator {
        &self.authorization_lineage
    }
}

impl WorkflowApprovalLayout {
    #[doc(hidden)]
    pub fn authorization_principal_for_test(&self) -> &AspectFieldLocator {
        &self.authorization_principal
    }
}

impl WorkflowApprovalLayout {
    #[doc(hidden)]
    pub fn authorization_request_for_test(&self) -> &AspectFieldLocator {
        &self.authorization_request
    }
}

impl WorkflowApprovalLayout {
    #[doc(hidden)]
    pub fn authorization_support_for_test(&self) -> &AspectFieldLocator {
        &self.authorization_support
    }
}

impl WorkflowDefinitionLayout {
    #[doc(hidden)]
    pub fn entity_kind_for_test(&self) -> &KindId {
        &self.entity_kind
    }
}

impl WorkflowInstanceLayout {
    #[doc(hidden)]
    pub fn entity_kind_for_test(&self) -> &KindId {
        &self.entity_kind
    }
}

impl WorkflowLineageLayout {
    #[doc(hidden)]
    pub fn entity_kind_for_test(&self) -> &KindId {
        &self.entity_kind
    }
}

impl WorkflowNodeLayout {
    #[doc(hidden)]
    pub fn entity_kind_for_test(&self) -> &KindId {
        &self.entity_kind
    }
}

impl WorkflowTransitionLayout {
    #[doc(hidden)]
    pub fn entity_kind_for_test(&self) -> &KindId {
        &self.entity_kind
    }
}

impl WorkflowTransitionLayout {
    #[doc(hidden)]
    pub fn identity_for_test(&self) -> &AspectFieldLocator {
        &self.identity
    }
}

impl WorkflowTransitionLayout {
    #[doc(hidden)]
    pub fn live_membership_generation_for_test(&self) -> &AspectFieldLocator {
        &self.live_membership_generation
    }
}

impl WorkflowTransitionLayout {
    #[doc(hidden)]
    pub fn live_membership_partition_for_test(&self) -> &AspectFieldLocator {
        &self.live_membership_partition
    }
}

impl WorkflowTransitionLayout {
    #[doc(hidden)]
    pub fn live_membership_slot_for_test(&self) -> &AspectFieldLocator {
        &self.live_membership_slot
    }
}

impl WorkflowTransitionLayout {
    #[doc(hidden)]
    pub fn occurrence_for_test(&self) -> &AspectFieldLocator {
        &self.occurrence
    }
}

impl WorkflowTransitionLayout {
    #[doc(hidden)]
    pub fn outcome_for_test(&self) -> &AspectFieldLocator {
        &self.outcome
    }
}

impl WorkflowTransitionLayout {
    #[doc(hidden)]
    pub fn output_count_for_test(&self) -> &AspectFieldLocator {
        &self.output_count
    }
}

impl WorkflowTransitionLayout {
    #[doc(hidden)]
    pub fn protocol_version_for_test(&self) -> &AspectFieldLocator {
        &self.protocol_version
    }
}

impl WorkflowEvidenceDependencyLayout {
    #[doc(hidden)]
    pub fn aspect_for_test(&self) -> &AspectFieldLocator {
        &self.aspect
    }
}

impl WorkflowEvidenceDependencyLayout {
    #[doc(hidden)]
    pub fn field_for_test(&self) -> &AspectFieldLocator {
        &self.field
    }
}

impl WorkflowTransitionLayout {
    #[doc(hidden)]
    pub fn identity_index_id_for_test(&self) -> &DerivedIndexId {
        &self.identity_index_id
    }
}

impl WorkflowInstanceLayout {
    #[doc(hidden)]
    pub fn program_revision_for_test(&self) -> &AspectFieldLocator {
        &self.program_revision
    }
}

impl WorkflowInstanceLayout {
    #[doc(hidden)]
    pub fn start_advancer_for_test(
        &self,
    ) -> &super::super::super::actor::record::WorkflowActorFields {
        &self.start_advancer
    }
}

impl WorkflowInstanceLayout {
    #[doc(hidden)]
    pub fn started_program_revision_for_test(&self) -> &AspectFieldLocator {
        &self.started_program_revision
    }
}

impl WorkflowInstanceLayout {
    #[doc(hidden)]
    pub fn subject_slot_for_test(&self) -> &AspectFieldLocator {
        &self.subject_slot
    }
}
