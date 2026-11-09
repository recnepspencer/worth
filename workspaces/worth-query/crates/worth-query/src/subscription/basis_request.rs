use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::declaration::QuerySubscriptionDeclarationArtifact;
use super::evidence_identities::basis_binding_request_identity;
use super::posture::QuerySubscriptionBasisPosture;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum QuerySubscriptionBasisBindingRequestKind {
    CurrentHead,
    BranchHead,
    RuntimeSnapshot,
    PreviewScoped,
    DeniedUnsupportedBasis,
}

impl QuerySubscriptionBasisBindingRequestKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CurrentHead => "current_head",
            Self::BranchHead => "branch_head",
            Self::RuntimeSnapshot => "runtime_snapshot",
            Self::PreviewScoped => "preview_scoped",
            Self::DeniedUnsupportedBasis => "denied_unsupported_basis",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionBasisBindingRequest {
    request_kind: QuerySubscriptionBasisBindingRequestKind,
    source_declaration_identity: WorthQueryEvidenceIdentity,
    scoped_declaration_basis_digest: String,
    evidence_identity: WorthQueryEvidenceIdentity,
}

impl QuerySubscriptionBasisBindingRequest {
    pub(super) fn from_declaration(
        declaration: &QuerySubscriptionDeclarationArtifact,
    ) -> Result<Self, IdentityDerivationDenial> {
        let request_kind = match declaration.basis_posture() {
            QuerySubscriptionBasisPosture::CurrentHead => {
                QuerySubscriptionBasisBindingRequestKind::CurrentHead
            }
            QuerySubscriptionBasisPosture::BranchHead => {
                QuerySubscriptionBasisBindingRequestKind::BranchHead
            }
            QuerySubscriptionBasisPosture::RuntimeHistoricalSnapshot => {
                QuerySubscriptionBasisBindingRequestKind::RuntimeSnapshot
            }
            QuerySubscriptionBasisPosture::PreviewScoped => {
                QuerySubscriptionBasisBindingRequestKind::PreviewScoped
            }
            QuerySubscriptionBasisPosture::DeniedUnsupportedBasis => {
                QuerySubscriptionBasisBindingRequestKind::DeniedUnsupportedBasis
            }
        };
        let source_declaration_identity = declaration.declaration_identity().clone();
        let scoped_declaration_basis_digest = declaration
            .scoped_declaration_basis()
            .expect("bridge lowering rejects declarations without scoped basis proof")
            .scoped_basis_digest()
            .to_string();
        let evidence_identity = basis_binding_request_identity(
            &request_kind,
            &source_declaration_identity,
            declaration.equivalence_identity(),
            &scoped_declaration_basis_digest,
        )?;
        Ok(Self {
            request_kind,
            source_declaration_identity,
            scoped_declaration_basis_digest,
            evidence_identity,
        })
    }

    pub fn request_kind(&self) -> &QuerySubscriptionBasisBindingRequestKind {
        &self.request_kind
    }

    pub fn source_declaration_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.source_declaration_identity
    }

    pub fn scoped_declaration_basis_digest(&self) -> &str {
        &self.scoped_declaration_basis_digest
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.evidence_identity
    }
}
