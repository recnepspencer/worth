pub mod diagnostic_fields;

use crate::diagnostics::data::{
    DeterminismExpectation, DiagnosticCode, DiagnosticsArtifactKind, DiagnosticsScope,
    RelationalDiagnosticArtifact, RelationalDiagnosticsEntry,
};
use crate::identity::data::KindId;
use crate::schema::data::aspect_merge_policy::AspectMergePolicyDeclaration;
use crate::schema::data::identity_declarations::IdentityBasisDeclaration;
use worth_foundational::facade::AspectKey;

use super::{
    AspectBinding, AspectContractPlanRevision, KindAspectContractDeclarations,
    LoweredAspectContractPlan,
};
use diagnostic_fields::{declaration_trace_diagnostic_fields, lowering_trace_diagnostic_fields};

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct AspectDeclarationTrace {
    pub kind_id: KindId,
    pub plan_revision: AspectContractPlanRevision,
    pub declarations: Vec<AspectDeclarationTraceRow>,
    pub identity_declarations: Vec<IdentityBasisDeclaration>,
    pub merge_policy_declarations: Vec<AspectMergePolicyDeclaration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AspectDeclarationTraceRow {
    pub aspect_key: AspectKey,
    pub binding: AspectBinding,
    pub contract_identity: u64,
    pub contract_revision: u64,
    pub aspect_shape: worth_foundational::AspectShape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct AspectLoweringTrace {
    pub kind_id: KindId,
    pub plan_revision: AspectContractPlanRevision,
    pub bindings: Vec<AspectLoweringTraceRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AspectLoweringTraceRow {
    pub aspect_key: AspectKey,
    pub target: AspectBinding,
    pub aspect_shape: worth_foundational::AspectShape,
}

impl KindAspectContractDeclarations {
    pub fn declaration_trace(&self, kind_id: KindId) -> AspectDeclarationTrace {
        AspectDeclarationTrace {
            kind_id,
            plan_revision: self.plan_revision,
            declarations: self
                .aspects
                .iter()
                .map(|aspect| AspectDeclarationTraceRow {
                    aspect_key: aspect.aspect_key(),
                    binding: aspect.binding.clone(),
                    contract_identity: aspect.contract.identity().0,
                    contract_revision: aspect.contract.revision().0,
                    aspect_shape: aspect.contract.shape().clone(),
                })
                .collect(),
            identity_declarations: self.identity_declarations.clone(),
            merge_policy_declarations: self.merge_policy_declarations.clone(),
        }
    }
}

impl LoweredAspectContractPlan {
    pub fn lowering_trace(&self) -> AspectLoweringTrace {
        AspectLoweringTrace {
            kind_id: self.kind_id,
            plan_revision: self.plan_revision,
            bindings: self
                .executable_bindings
                .iter()
                .map(|binding| AspectLoweringTraceRow {
                    aspect_key: binding.aspect_key().clone(),
                    target: binding.target.clone(),
                    aspect_shape: binding.aspect_shape(),
                })
                .collect(),
        }
    }
}

impl AspectDeclarationTrace {
    pub fn diagnostic_artifact(&self) -> RelationalDiagnosticArtifact {
        let fields = declaration_trace_diagnostic_fields(self);
        RelationalDiagnosticArtifact::new(
            DiagnosticsScope::Schema,
            DiagnosticsArtifactKind::DetailedTrace,
            DeterminismExpectation::Required,
            vec![RelationalDiagnosticsEntry::new(
                DiagnosticCode::AspectDeclarationTraced,
                "aspect declaration trace derived from canonical schema declarations",
                fields,
            )],
        )
    }
}

impl AspectLoweringTrace {
    pub fn diagnostic_artifact(&self) -> RelationalDiagnosticArtifact {
        let fields = lowering_trace_diagnostic_fields(self);
        RelationalDiagnosticArtifact::new(
            DiagnosticsScope::Schema,
            DiagnosticsArtifactKind::DetailedTrace,
            DeterminismExpectation::Required,
            vec![RelationalDiagnosticsEntry::new(
                DiagnosticCode::AspectLoweringTraced,
                "aspect lowering trace derived from canonical lowered aspect plan",
                fields,
            )],
        )
    }
}
