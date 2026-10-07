use super::{
    read_composition::WorthQueryReadScopeClass, WorthQueryReadBuiltInOperator,
    WorthQueryReadBuiltInOperatorDenial, WorthQueryReadBuiltInOperatorDenialReason,
    WorthQueryReadRelationshipProofDenial,
};
use crate::domain_installation::WorthQueryInstalledGraphReadOperationBindingDenial;
use crate::runtime::{
    WorthQueryGraphReadAccessAdmission, WorthQueryGraphReadAccessExecutionCounters,
    WorthQueryGraphReadPersistentArtifactAudit,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryReadDenialKind {
    Handle(worth_query_execution::facade::primary_graph::WorthQueryHandleDenial),
    InvalidRoot,
    BuiltInOperatorDenied,
    RelationshipProofAdmissionDenied,
    ScopeShapeDenied,
    InstalledOperationBindingDenied,
    AuthoringDenied,
    CanonicalizationDenied,
    ValidationDenied,
    PlanningDenied,
    BasisResolutionDenied,
    BasisPreflightDenied,
    ExecutionDenied,
}

impl WorthQueryReadDenialKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Handle(_) => "handle_denied",
            Self::InvalidRoot => "invalid_root",
            Self::BuiltInOperatorDenied => "built_in_operator_denied",
            Self::RelationshipProofAdmissionDenied => "relationship_proof_admission_denied",
            Self::ScopeShapeDenied => "scope_shape_denied",
            Self::InstalledOperationBindingDenied => "installed_operation_binding_denied",
            Self::AuthoringDenied => "authoring_denied",
            Self::CanonicalizationDenied => "canonicalization_denied",
            Self::ValidationDenied => "validation_denied",
            Self::PlanningDenied => "planning_denied",
            Self::BasisResolutionDenied => "basis_resolution_denied",
            Self::BasisPreflightDenied => "basis_preflight_denied",
            Self::ExecutionDenied => "execution_denied",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryReadScopeShapeMismatch {
    expected: WorthQueryReadScopeClass,
    actual: WorthQueryReadScopeClass,
}

impl WorthQueryReadScopeShapeMismatch {
    pub fn expected(&self) -> &WorthQueryReadScopeClass {
        &self.expected
    }

    pub fn actual(&self) -> &WorthQueryReadScopeClass {
        &self.actual
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryReadAccessPlanBindingMismatch {
    admitted_read_graph_digest: String,
    execution_read_graph_digest: String,
    provided_plan_digest: String,
    provided_admission_digest: String,
}

impl WorthQueryReadAccessPlanBindingMismatch {
    pub fn admitted_read_graph_digest(&self) -> &str {
        &self.admitted_read_graph_digest
    }

    pub fn execution_read_graph_digest(&self) -> &str {
        &self.execution_read_graph_digest
    }

    pub fn provided_plan_digest(&self) -> &str {
        &self.provided_plan_digest
    }

    pub fn provided_admission_digest(&self) -> &str {
        &self.provided_admission_digest
    }

    #[cfg(test)]
    pub(in crate::runtime) fn new(
        admitted_read_graph_digest: impl Into<String>,
        execution_read_graph_digest: impl Into<String>,
        provided_plan_digest: impl Into<String>,
        provided_admission_digest: impl Into<String>,
    ) -> Self {
        Self {
            admitted_read_graph_digest: admitted_read_graph_digest.into(),
            execution_read_graph_digest: execution_read_graph_digest.into(),
            provided_plan_digest: provided_plan_digest.into(),
            provided_admission_digest: provided_admission_digest.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryReadDenial {
    kind: WorthQueryReadDenialKind,
    message: String,
    built_in_operator_denial: Option<Box<WorthQueryReadBuiltInOperatorDenial>>,
    relationship_proof_denial: Option<Box<WorthQueryReadRelationshipProofDenial>>,
    scope_shape_mismatch: Option<Box<WorthQueryReadScopeShapeMismatch>>,
    installed_operation_binding_denial:
        Option<Box<WorthQueryInstalledGraphReadOperationBindingDenial>>,
    access_plan_binding_mismatch: Option<Box<WorthQueryReadAccessPlanBindingMismatch>>,
    graph_read_access_admission: Option<Box<WorthQueryGraphReadAccessAdmission>>,
    graph_read_access_execution_counters: Option<Box<WorthQueryGraphReadAccessExecutionCounters>>,
    graph_read_persistent_artifact_audit: Option<Box<WorthQueryGraphReadPersistentArtifactAudit>>,
}

impl WorthQueryReadDenial {
    pub fn kind(&self) -> &WorthQueryReadDenialKind {
        &self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn built_in_operator_denial(&self) -> Option<&WorthQueryReadBuiltInOperatorDenial> {
        self.built_in_operator_denial.as_deref()
    }

    pub fn relationship_proof_denial(&self) -> Option<&WorthQueryReadRelationshipProofDenial> {
        self.relationship_proof_denial.as_deref()
    }

    pub fn scope_shape_mismatch(&self) -> Option<&WorthQueryReadScopeShapeMismatch> {
        self.scope_shape_mismatch.as_deref()
    }

    pub fn installed_operation_binding_denial(
        &self,
    ) -> Option<&WorthQueryInstalledGraphReadOperationBindingDenial> {
        self.installed_operation_binding_denial.as_deref()
    }

    pub fn access_plan_binding_mismatch(&self) -> Option<&WorthQueryReadAccessPlanBindingMismatch> {
        self.access_plan_binding_mismatch.as_deref()
    }

    pub fn graph_read_access_admission(&self) -> Option<&WorthQueryGraphReadAccessAdmission> {
        self.graph_read_access_admission.as_deref()
    }

    pub fn graph_read_access_execution_counters(
        &self,
    ) -> Option<&WorthQueryGraphReadAccessExecutionCounters> {
        self.graph_read_access_execution_counters.as_deref()
    }

    pub fn graph_read_persistent_artifact_audit(
        &self,
    ) -> Option<&WorthQueryGraphReadPersistentArtifactAudit> {
        self.graph_read_persistent_artifact_audit.as_deref()
    }

    pub(crate) fn new(kind: WorthQueryReadDenialKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            built_in_operator_denial: None,
            relationship_proof_denial: None,
            scope_shape_mismatch: None,
            installed_operation_binding_denial: None,
            access_plan_binding_mismatch: None,
            graph_read_access_admission: None,
            graph_read_access_execution_counters: None,
            graph_read_persistent_artifact_audit: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_access_plan_binding_mismatch(
        mut self,
        access_plan_binding_mismatch: WorthQueryReadAccessPlanBindingMismatch,
    ) -> Self {
        self.access_plan_binding_mismatch = Some(Box::new(access_plan_binding_mismatch));
        self
    }

    pub(crate) fn with_graph_read_access_admission(
        mut self,
        graph_read_access_admission: WorthQueryGraphReadAccessAdmission,
    ) -> Self {
        self.graph_read_access_admission = Some(Box::new(graph_read_access_admission));
        self
    }

    pub(crate) fn with_graph_read_access_execution_counters(
        mut self,
        graph_read_access_execution_counters: WorthQueryGraphReadAccessExecutionCounters,
    ) -> Self {
        self.graph_read_access_execution_counters =
            Some(Box::new(graph_read_access_execution_counters));
        self
    }

    pub(crate) fn with_graph_read_persistent_artifact_audit(
        mut self,
        graph_read_persistent_artifact_audit: WorthQueryGraphReadPersistentArtifactAudit,
    ) -> Self {
        self.graph_read_persistent_artifact_audit =
            Some(Box::new(graph_read_persistent_artifact_audit));
        self
    }

    pub(crate) fn with_graph_read_persistent_artifact_audit_for_admission(
        self,
        admission: &WorthQueryGraphReadAccessAdmission,
    ) -> Self {
        if admission.persistent_index_requirement().is_some() {
            return self.with_graph_read_persistent_artifact_audit(
                WorthQueryGraphReadPersistentArtifactAudit::declaration_only_stop(),
            );
        }
        self
    }

    pub(in crate::runtime) fn new_built_in_operator_denied(
        operator: WorthQueryReadBuiltInOperator,
        reason: WorthQueryReadBuiltInOperatorDenialReason,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind: WorthQueryReadDenialKind::BuiltInOperatorDenied,
            message: message.into(),
            built_in_operator_denial: Some(Box::new(WorthQueryReadBuiltInOperatorDenial::new(
                operator, reason,
            ))),
            relationship_proof_denial: None,
            scope_shape_mismatch: None,
            installed_operation_binding_denial: None,
            access_plan_binding_mismatch: None,
            graph_read_access_admission: None,
            graph_read_access_execution_counters: None,
            graph_read_persistent_artifact_audit: None,
        }
    }

    pub(in crate::runtime) fn new_relationship_proof_admission_denied(
        denial: WorthQueryReadRelationshipProofDenial,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind: WorthQueryReadDenialKind::RelationshipProofAdmissionDenied,
            message: message.into(),
            built_in_operator_denial: None,
            relationship_proof_denial: Some(Box::new(denial)),
            scope_shape_mismatch: None,
            installed_operation_binding_denial: None,
            access_plan_binding_mismatch: None,
            graph_read_access_admission: None,
            graph_read_access_execution_counters: None,
            graph_read_persistent_artifact_audit: None,
        }
    }

    pub(in crate::runtime) fn new_scope_shape_denied(
        expected: WorthQueryReadScopeClass,
        actual: WorthQueryReadScopeClass,
    ) -> Self {
        let message = format!(
            "requested `{}` read shape lowers to `{}` under current kernel classification",
            expected.as_str(),
            actual.as_str()
        );
        Self {
            kind: WorthQueryReadDenialKind::ScopeShapeDenied,
            message,
            built_in_operator_denial: None,
            relationship_proof_denial: None,
            scope_shape_mismatch: Some(Box::new(WorthQueryReadScopeShapeMismatch {
                expected,
                actual,
            })),
            installed_operation_binding_denial: None,
            access_plan_binding_mismatch: None,
            graph_read_access_admission: None,
            graph_read_access_execution_counters: None,
            graph_read_persistent_artifact_audit: None,
        }
    }

    pub(in crate::runtime) fn new_installed_operation_binding_denied(
        denial: WorthQueryInstalledGraphReadOperationBindingDenial,
    ) -> Self {
        Self {
            kind: WorthQueryReadDenialKind::InstalledOperationBindingDenied,
            message: format!("installed graph-read operation binding failed: {denial:?}"),
            built_in_operator_denial: None,
            relationship_proof_denial: None,
            scope_shape_mismatch: None,
            installed_operation_binding_denial: Some(Box::new(denial)),
            access_plan_binding_mismatch: None,
            graph_read_access_admission: None,
            graph_read_access_execution_counters: None,
            graph_read_persistent_artifact_audit: None,
        }
    }
}

impl std::fmt::Display for WorthQueryReadDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "read composition {}: {}",
            self.kind.as_str(),
            self.message
        )
    }
}

impl std::error::Error for WorthQueryReadDenial {}

impl From<worth_query_execution::facade::primary_graph::WorthQueryHandleDenial>
    for WorthQueryReadDenial
{
    fn from(denial: worth_query_execution::facade::primary_graph::WorthQueryHandleDenial) -> Self {
        Self::new(WorthQueryReadDenialKind::Handle(denial), denial.to_string())
    }
}
