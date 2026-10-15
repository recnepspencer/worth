#[cfg(test)]
use super::WorthQueryGraphReadPolicyTenantAuthorityRequest;
use super::{
    WorthQueryGraphReadAccessAuthorityCounters, WorthQueryGraphReadAccessAuthorityDenial,
    WorthQueryGraphReadAccessAuthorityDenialKind, WorthQueryGraphReadAccessAuthorityReceipt,
    WorthQueryGraphReadAccessAuthorityRequest,
};
#[cfg(test)]
use crate::policy_basis::admit_policy_tenant_context;
use crate::policy_basis::{AdmittedPolicyTenantContext, PolicyExecutionModeRequest};
use crate::relationship_proof::RelationshipProofAdmission;
use crate::runtime::{
    WorthQueryBranchBasisAdmission, WorthQueryGraphReadBasisBinding,
    WorthQueryGraphReadBasisPosture, WorthQueryGraphReadPolicyTenantPosture,
    WorthQueryGraphReadPolicyTenantProofBinding,
    WorthQueryGraphReadRelationshipProofBindingPosture, WorthQueryPreviewBasisAdmission,
    WorthQueryReadGraph,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphReadAccessBasisScopeKind {
    RuntimeCurrent,
    Preview,
    Branch,
}

impl WorthQueryGraphReadAccessBasisScopeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RuntimeCurrent => "runtime_current",
            Self::Preview => "preview",
            Self::Branch => "branch",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadAccessBasisScope {
    kind: WorthQueryGraphReadAccessBasisScopeKind,
    admission_digest: Option<String>,
}

impl WorthQueryGraphReadAccessBasisScope {
    pub fn kind(&self) -> WorthQueryGraphReadAccessBasisScopeKind {
        self.kind
    }

    pub fn admission_digest(&self) -> Option<&str> {
        self.admission_digest.as_deref()
    }

    pub(crate) fn runtime_current() -> Self {
        Self {
            kind: WorthQueryGraphReadAccessBasisScopeKind::RuntimeCurrent,
            admission_digest: None,
        }
    }

    fn preview(basis: &WorthQueryPreviewBasisAdmission) -> Self {
        Self {
            kind: WorthQueryGraphReadAccessBasisScopeKind::Preview,
            admission_digest: Some(basis.admission_digest().as_str().to_string()),
        }
    }

    fn branch(basis: &WorthQueryBranchBasisAdmission) -> Self {
        Self {
            kind: WorthQueryGraphReadAccessBasisScopeKind::Branch,
            admission_digest: Some(basis.admission_digest().as_str().to_string()),
        }
    }

    pub(crate) fn digest_part(&self) -> String {
        format!(
            "basis_scope:{}:{}",
            self.kind.as_str(),
            self.admission_digest.as_deref().unwrap_or("none")
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadAccessAuthorityContext {
    basis_scope: WorthQueryGraphReadAccessBasisScope,
    policy_tenant: Option<AdmittedPolicyTenantContext>,
    relationship_proof: Option<RelationshipProofAdmission>,
    receipt: WorthQueryGraphReadAccessAuthorityReceipt,
}

impl WorthQueryGraphReadAccessAuthorityContext {
    pub fn basis_scope(&self) -> &WorthQueryGraphReadAccessBasisScope {
        &self.basis_scope
    }

    pub fn policy_tenant(&self) -> Option<&AdmittedPolicyTenantContext> {
        self.policy_tenant.as_ref()
    }

    pub fn relationship_proof(&self) -> Option<&RelationshipProofAdmission> {
        self.relationship_proof.as_ref()
    }

    pub fn receipt(&self) -> &WorthQueryGraphReadAccessAuthorityReceipt {
        &self.receipt
    }

    pub(crate) fn runtime_current_compatibility() -> Self {
        let basis_scope = WorthQueryGraphReadAccessBasisScope::runtime_current();
        let receipt = WorthQueryGraphReadAccessAuthorityReceipt::runtime_current_compatibility();
        Self {
            basis_scope,
            policy_tenant: None,
            relationship_proof: None,
            receipt,
        }
    }

    pub(crate) fn bind_for_read_graph(
        &self,
        read_graph: &WorthQueryReadGraph,
    ) -> (
        WorthQueryGraphReadBasisBinding,
        WorthQueryGraphReadPolicyTenantProofBinding,
    ) {
        let basis_binding = WorthQueryGraphReadBasisBinding::new(
            read_graph.digest(),
            *read_graph.schema_basis().digest(),
            basis_posture_from_scope(&self.basis_scope),
        );
        let explicit_relationship_proof = self
            .relationship_proof
            .as_ref()
            .map(|admission| admission.identity().as_str().to_string());
        let relationship_proof_admission_digest =
            explicit_relationship_proof.clone().or_else(|| {
                read_graph
                    .relationship_proof_admission()
                    .map(|admission| admission.identity().as_str().to_string())
            });
        let relationship_posture = if explicit_relationship_proof.is_some() {
            relationship_posture_from_scope(&self.basis_scope)
        } else if relationship_proof_admission_digest.is_some() {
            WorthQueryGraphReadRelationshipProofBindingPosture::DescriptorAdmittedSyntheticRuntime
        } else {
            WorthQueryGraphReadRelationshipProofBindingPosture::NotRequired
        };
        let policy_posture = match self.basis_scope.kind {
            WorthQueryGraphReadAccessBasisScopeKind::RuntimeCurrent => {
                if self.policy_tenant.is_some() {
                    WorthQueryGraphReadPolicyTenantPosture::AdmittedCurrentRead
                } else {
                    WorthQueryGraphReadPolicyTenantPosture::SyntheticRuntimeCurrentRead
                }
            }
            WorthQueryGraphReadAccessBasisScopeKind::Preview => {
                WorthQueryGraphReadPolicyTenantPosture::AdmittedPreviewRead
            }
            WorthQueryGraphReadAccessBasisScopeKind::Branch => {
                WorthQueryGraphReadPolicyTenantPosture::AdmittedBranchRead
            }
        };
        let policy_tenant_proof_binding = WorthQueryGraphReadPolicyTenantProofBinding::new(
            read_graph.digest(),
            policy_posture,
            relationship_posture,
            relationship_proof_admission_digest,
            self.policy_tenant
                .as_ref()
                .map(|context| context.bundle().digest().as_str().to_string()),
            self.receipt.digest().to_string(),
        );
        (basis_binding, policy_tenant_proof_binding)
    }
}

pub fn admit_graph_read_access_authority(
    request: WorthQueryGraphReadAccessAuthorityRequest,
) -> Result<WorthQueryGraphReadAccessAuthorityContext, WorthQueryGraphReadAccessAuthorityDenial> {
    let (basis_scope, policy_tenant, relationship_proof) = match request {
        WorthQueryGraphReadAccessAuthorityRequest::CurrentHead {
            policy_tenant,
            relationship_proof,
        } => (
            WorthQueryGraphReadAccessBasisScope::runtime_current(),
            policy_tenant,
            relationship_proof,
        ),
        WorthQueryGraphReadAccessAuthorityRequest::Preview {
            basis,
            policy_tenant,
            relationship_proof,
        } => (
            WorthQueryGraphReadAccessBasisScope::preview(&basis),
            policy_tenant,
            relationship_proof,
        ),
        WorthQueryGraphReadAccessAuthorityRequest::Branch {
            basis,
            policy_tenant,
            relationship_proof,
        } => (
            WorthQueryGraphReadAccessBasisScope::branch(&basis),
            policy_tenant,
            relationship_proof,
        ),
    };

    if relationship_proof.is_some() && policy_tenant.is_none() {
        return Err(WorthQueryGraphReadAccessAuthorityDenial::new(
            WorthQueryGraphReadAccessAuthorityDenialKind::RelationshipProofRequiresPolicyTenantContext,
            "relationship proof admission is only valid with its admitted policy/tenant context",
            WorthQueryGraphReadAccessAuthorityCounters::denied(false, true),
        ));
    }

    if let Some(policy_tenant) = policy_tenant.as_ref() {
        let expected_mode = policy_execution_mode_for_basis_scope(&basis_scope);
        if policy_tenant.bundle().execution_mode() != expected_mode {
            return Err(WorthQueryGraphReadAccessAuthorityDenial::new(
                WorthQueryGraphReadAccessAuthorityDenialKind::PolicyTenantBasisScopeMismatch,
                format!(
                    "policy/tenant admission mode {} cannot authorize {} graph-read basis scope",
                    policy_tenant.bundle().execution_mode().as_str(),
                    basis_scope.kind().as_str()
                ),
                WorthQueryGraphReadAccessAuthorityCounters::denied(
                    true,
                    relationship_proof.is_some(),
                ),
            ));
        }
    }

    if let (Some(policy_tenant), Some(relationship_proof)) =
        (policy_tenant.as_ref(), relationship_proof.as_ref())
        && (relationship_proof.policy_digest() != policy_tenant.bundle().policy_digest()
            || relationship_proof.tenant_schema_basis_digest()
                != policy_tenant.bundle().tenant_schema_basis_digest())
    {
        return Err(WorthQueryGraphReadAccessAuthorityDenial::new(
            WorthQueryGraphReadAccessAuthorityDenialKind::RelationshipProofPolicyTenantMismatch,
            "relationship proof admission must belong to the same admitted policy/tenant context",
            WorthQueryGraphReadAccessAuthorityCounters::denied(true, true),
        ));
    }

    let policy_tenant_digest = policy_tenant
        .as_ref()
        .map(|context| context.bundle().digest().as_str().to_string());
    let relationship_proof_digest = relationship_proof
        .as_ref()
        .map(|admission| admission.identity().as_str().to_string());
    let counters = WorthQueryGraphReadAccessAuthorityCounters::admitted(
        policy_tenant_digest.is_some(),
        relationship_proof_digest.is_some(),
    );
    let receipt = WorthQueryGraphReadAccessAuthorityReceipt::new(
        basis_scope.clone(),
        policy_tenant_digest,
        relationship_proof_digest,
        counters,
    );
    Ok(WorthQueryGraphReadAccessAuthorityContext {
        basis_scope,
        policy_tenant,
        relationship_proof,
        receipt,
    })
}

#[cfg(test)]
pub fn admit_graph_read_access_authority_from_policy_tenant_request(
    request: WorthQueryGraphReadPolicyTenantAuthorityRequest,
) -> Result<WorthQueryGraphReadAccessAuthorityContext, WorthQueryGraphReadAccessAuthorityDenial> {
    let policy_tenant = admit_policy_tenant_context(
        request.query(),
        request.policy().clone(),
        request.tenant().clone(),
        request.branch().clone(),
        request.schema().clone(),
        request.execution_mode(),
    )
    .map_err(|error| {
        WorthQueryGraphReadAccessAuthorityDenial::new(
            WorthQueryGraphReadAccessAuthorityDenialKind::PolicyTenantDenied,
            error.message(),
            WorthQueryGraphReadAccessAuthorityCounters::denied(true, false),
        )
    })?;

    admit_graph_read_access_authority(
        WorthQueryGraphReadAccessAuthorityRequest::current_head().with_policy_tenant(policy_tenant),
    )
}

fn policy_execution_mode_for_basis_scope(
    scope: &WorthQueryGraphReadAccessBasisScope,
) -> PolicyExecutionModeRequest {
    match scope.kind {
        WorthQueryGraphReadAccessBasisScopeKind::RuntimeCurrent
        | WorthQueryGraphReadAccessBasisScopeKind::Preview => {
            PolicyExecutionModeRequest::CurrentRead
        }
        WorthQueryGraphReadAccessBasisScopeKind::Branch => PolicyExecutionModeRequest::BranchRead,
    }
}

fn basis_posture_from_scope(
    scope: &WorthQueryGraphReadAccessBasisScope,
) -> WorthQueryGraphReadBasisPosture {
    match scope.kind {
        WorthQueryGraphReadAccessBasisScopeKind::RuntimeCurrent => {
            WorthQueryGraphReadBasisPosture::RuntimeCurrent
        }
        WorthQueryGraphReadAccessBasisScopeKind::Preview => {
            WorthQueryGraphReadBasisPosture::PreviewAdmitted
        }
        WorthQueryGraphReadAccessBasisScopeKind::Branch => {
            WorthQueryGraphReadBasisPosture::BranchAdmitted
        }
    }
}

fn relationship_posture_from_scope(
    scope: &WorthQueryGraphReadAccessBasisScope,
) -> WorthQueryGraphReadRelationshipProofBindingPosture {
    match scope.kind {
        WorthQueryGraphReadAccessBasisScopeKind::RuntimeCurrent => {
            WorthQueryGraphReadRelationshipProofBindingPosture::DescriptorAdmittedRuntimeCurrent
        }
        WorthQueryGraphReadAccessBasisScopeKind::Preview => {
            WorthQueryGraphReadRelationshipProofBindingPosture::DescriptorAdmittedPreview
        }
        WorthQueryGraphReadAccessBasisScopeKind::Branch => {
            WorthQueryGraphReadRelationshipProofBindingPosture::DescriptorAdmittedBranch
        }
    }
}
