use crate::runtime::expression::WorthUiAuthoredExpressionMaterial;
use crate::source::WorthUiArtifact;

use super::{WorthUiPreparedDeclarationMaterial, WorthUiSemanticHandoffEvidence};

pub(in crate::runtime::source_ingress) struct WorthUiPreparedSemanticHandoffMaterial {
    artifact: WorthUiArtifact,
    declaration_material: WorthUiPreparedDeclarationMaterial,
    evidence: WorthUiSemanticHandoffEvidence,
    expression_material: WorthUiAuthoredExpressionMaterial,
}

impl WorthUiPreparedSemanticHandoffMaterial {
    pub(in crate::runtime::source_ingress) fn new(
        artifact: WorthUiArtifact,
        declaration_material: WorthUiPreparedDeclarationMaterial,
        evidence: WorthUiSemanticHandoffEvidence,
        expression_material: WorthUiAuthoredExpressionMaterial,
    ) -> Self {
        Self {
            artifact,
            declaration_material,
            evidence,
            expression_material,
        }
    }

    pub(in crate::runtime::source_ingress) fn into_parts(
        self,
    ) -> (
        WorthUiArtifact,
        WorthUiPreparedDeclarationMaterial,
        WorthUiSemanticHandoffEvidence,
        WorthUiAuthoredExpressionMaterial,
    ) {
        (
            self.artifact,
            self.declaration_material,
            self.evidence,
            self.expression_material,
        )
    }
}
