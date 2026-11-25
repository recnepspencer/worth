//! Exact owned reconstruction parts for one portable conditional declaration.

use super::WorthQueryPortableConditionalNodeDeclaration;
use crate::domain_operation::conditional_node::{
    WorthQueryArtifactPosture, WorthQueryArtifactReuseEquivalence, WorthQueryComparatorRequirement,
    WorthQueryConditionalEvaluationCondition, WorthQueryConditionalNodeContext,
    WorthQueryConditionalNodeOutput, WorthQueryConditionalNodeRole, WorthQueryConditionalTrigger,
    WorthQueryMaintenancePosture, WorthQueryOutputEquivalenceRequirement,
    WorthQueryOutputRelationship, WorthQuerySemanticTruthDependency,
};
use worth_query_declaration::archive_decode_authority::ArchiveDecodeAuthority;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableConditionalNodeParts {
    pub identity: String,
    pub role: WorthQueryConditionalNodeRole,
    pub dependencies: Vec<WorthQuerySemanticTruthDependency>,
    pub outputs: Vec<WorthQueryConditionalNodeOutput>,
    pub required_context: Vec<WorthQueryConditionalNodeContext>,
    pub condition: WorthQueryConditionalEvaluationCondition,
    pub trigger: WorthQueryConditionalTrigger,
    pub dependency_comparator: WorthQueryComparatorRequirement,
    pub output_equivalence: WorthQueryOutputEquivalenceRequirement,
    pub artifact_reuse_equivalence: WorthQueryArtifactReuseEquivalence,
    pub maintenance: WorthQueryMaintenancePosture,
    pub artifact: WorthQueryArtifactPosture,
    pub output_relationship: WorthQueryOutputRelationship,
}

impl WorthQueryPortableConditionalNodeDeclaration {
    /// Constructs an authority-free declaration from decoded, untrusted fields.
    ///
    /// Unlike the authoring constructor, this does not canonicalize or validate
    /// package meaning. Fresh package reconstruction remains the semantic gate.
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableConditionalNodeParts,
    ) -> Self {
        Self {
            identity: parts.identity,
            role: parts.role,
            dependencies: parts.dependencies,
            outputs: parts.outputs,
            required_context: parts.required_context,
            condition: parts.condition,
            trigger: parts.trigger,
            dependency_comparator: parts.dependency_comparator,
            output_equivalence: parts.output_equivalence,
            artifact_reuse_equivalence: parts.artifact_reuse_equivalence,
            maintenance: parts.maintenance,
            artifact: parts.artifact,
            output_relationship: parts.output_relationship,
        }
    }

    pub fn into_parts(self) -> WorthQueryPortableConditionalNodeParts {
        WorthQueryPortableConditionalNodeParts {
            identity: self.identity,
            role: self.role,
            dependencies: self.dependencies,
            outputs: self.outputs,
            required_context: self.required_context,
            condition: self.condition,
            trigger: self.trigger,
            dependency_comparator: self.dependency_comparator,
            output_equivalence: self.output_equivalence,
            artifact_reuse_equivalence: self.artifact_reuse_equivalence,
            maintenance: self.maintenance,
            artifact: self.artifact,
            output_relationship: self.output_relationship,
        }
    }
}
