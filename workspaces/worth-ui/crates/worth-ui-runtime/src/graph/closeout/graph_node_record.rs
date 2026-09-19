use crate::declaration::{
    UiDeclarationIdentity, UiDeclarationPlanningOperatorKind, UiDeclarationRepetitionPosture,
    UiDeclarationStructuralDigest, UiDeclarationStructuralRole, UiDeclaredMeasurementBasisSource,
    UiDeclaredMeasurementConstraintModifier,
};
use crate::graph::{
    UiGraphAttachmentPosture, UiGraphNode, UiGraphNodeIdentity, UiGraphParticipationPosture,
    UiRepeatedInstanceBasis,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiGraphNodeRecord {
    graph_node_identity: UiGraphNodeIdentity,
    declaration_identity: UiDeclarationIdentity,
    structural_digest: UiDeclarationStructuralDigest,
    structural_role: UiDeclarationStructuralRole,
    operator_kind: UiDeclarationPlanningOperatorKind,
    repetition_posture: UiDeclarationRepetitionPosture,
    measurement_constraint_modifier: Option<UiDeclaredMeasurementConstraintModifier>,
    measurement_basis_source: Option<UiDeclaredMeasurementBasisSource>,
    authored_provenance_digest: u64,
    repeated_instance_basis: UiRepeatedInstanceBasis,
    attachment_posture: UiGraphAttachmentPosture,
    participation_posture: UiGraphParticipationPosture,
    has_appearance_attachment: bool,
}

impl UiGraphNodeRecord {
    pub(crate) const fn has_appearance_attachment(&self) -> bool {
        self.has_appearance_attachment
    }

    pub fn graph_node_identity(&self) -> UiGraphNodeIdentity {
        self.graph_node_identity
    }

    pub fn declaration_identity(&self) -> &UiDeclarationIdentity {
        &self.declaration_identity
    }

    pub fn repeated_instance_basis(&self) -> &UiRepeatedInstanceBasis {
        &self.repeated_instance_basis
    }

    pub fn authored_provenance_digest(&self) -> u64 {
        self.authored_provenance_digest
    }

    pub fn structural_role(&self) -> UiDeclarationStructuralRole {
        self.structural_role
    }

    pub fn operator_kind(&self) -> UiDeclarationPlanningOperatorKind {
        self.operator_kind
    }

    pub fn structural_digest(&self) -> UiDeclarationStructuralDigest {
        self.structural_digest
    }

    pub fn repetition_posture(&self) -> UiDeclarationRepetitionPosture {
        self.repetition_posture
    }

    pub fn measurement_constraint_modifier(
        &self,
    ) -> Option<UiDeclaredMeasurementConstraintModifier> {
        self.measurement_constraint_modifier
    }

    pub fn measurement_basis_source(&self) -> Option<UiDeclaredMeasurementBasisSource> {
        self.measurement_basis_source
    }

    pub fn attachment_posture(&self) -> UiGraphAttachmentPosture {
        self.attachment_posture
    }

    pub fn participation_posture(&self) -> UiGraphParticipationPosture {
        self.participation_posture
    }
}

impl From<&UiGraphNode> for UiGraphNodeRecord {
    fn from(node: &UiGraphNode) -> Self {
        Self {
            graph_node_identity: node.graph_node_identity(),
            declaration_identity: node.declaration_identity().clone(),
            structural_digest: node.structural_digest(),
            structural_role: node.structural_role(),
            operator_kind: node.operator_kind(),
            repetition_posture: node.repetition_posture(),
            measurement_constraint_modifier: node.measurement_constraint_modifier(),
            measurement_basis_source: node.measurement_basis_source(),
            authored_provenance_digest: node.authored_provenance_digest(),
            repeated_instance_basis: node.repeated_instance_basis().clone(),
            attachment_posture: node.attachment_posture(),
            participation_posture: node.participation_posture(),
            has_appearance_attachment: node.appearance_role_attachment().is_some(),
        }
    }
}
