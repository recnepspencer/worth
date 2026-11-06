use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::identity::{CanonicalQueryDigest, PlanDigest, ValidatedQueryDigest};
use crate::memory_workspace::WorthQuerySnapshotIdentity;

use super::context_binding::WorkflowBasisFamily;
use super::declaration_model::WorkflowDeclarationRequest;

pub(super) fn workflow_context_binding_identity(
    source_identity: &WorthQueryEvidenceIdentity,
    query_identity: &WorthQueryEvidenceIdentity,
    basis_family: WorkflowBasisFamily,
    basis_identity: &WorthQueryEvidenceIdentity,
    runtime_snapshot_identity: Option<&WorthQuerySnapshotIdentity>,
    binding_scope: Option<&WorkflowBindingScopeField<'_>>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let mut identity =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_evidence_identity(WorthQueryEvidenceTag::new("source"), source_identity)
            .field_evidence_identity(WorthQueryEvidenceTag::new("query"), query_identity)
            .field_shape(
                WorthQueryEvidenceTag::new("basis_family"),
                basis_family.as_str(),
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("basis"), basis_identity);
    if let Some(runtime_snapshot_identity) = runtime_snapshot_identity {
        identity = identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("runtime_snapshot"),
            &runtime_snapshot_identity.evidence_identity(),
        );
    }
    if let Some(binding_scope) = binding_scope {
        identity = match binding_scope {
            WorkflowBindingScopeField::Unscoped => {
                identity.field_shape(WorthQueryEvidenceTag::new("scope"), "unscoped")
            }
            WorkflowBindingScopeField::Shape(label) => {
                identity.field_shape(WorthQueryEvidenceTag::new("scope"), *label)
            }
            WorkflowBindingScopeField::Identity(scope_identity) => identity
                .field_evidence_identity(WorthQueryEvidenceTag::new("scope"), scope_identity),
        };
    }
    Ok(identity.seal()?)
}

pub(crate) enum WorkflowBindingScopeField<'a> {
    Unscoped,
    Shape(&'a str),
    Identity(&'a WorthQueryEvidenceIdentity),
}

pub(super) fn workflow_scope_from_label(label: &str) -> WorkflowBindingScopeField<'_> {
    if label == "unscoped" {
        WorkflowBindingScopeField::Unscoped
    } else {
        WorkflowBindingScopeField::Shape(label)
    }
}

pub(super) fn apply_binding_scope_field(
    identity: crate::evidence_identity::WorthQueryEvidenceIdentityEncoder,
    scope: &WorkflowBindingScopeField<'_>,
) -> crate::evidence_identity::WorthQueryEvidenceIdentityEncoder {
    match scope {
        WorkflowBindingScopeField::Unscoped => {
            identity.field_shape(WorthQueryEvidenceTag::new("scope"), "unscoped")
        }
        WorkflowBindingScopeField::Shape(label) => {
            identity.field_shape(WorthQueryEvidenceTag::new("scope"), *label)
        }
        WorkflowBindingScopeField::Identity(scope_identity) => {
            identity.field_evidence_identity(WorthQueryEvidenceTag::new("scope"), scope_identity)
        }
    }
}

pub(super) fn binding_scope_for_context_binding<'a>(
    scope: &'a WorkflowBindingScopeField<'a>,
) -> Option<&'a WorkflowBindingScopeField<'a>> {
    match scope {
        WorkflowBindingScopeField::Unscoped => None,
        _ => Some(scope),
    }
}

pub(crate) fn workflow_context_source_identity(
    source_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "workflow_context_source_v1",
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("source"), source_identity)
            .seal()?,
    )
}

pub(crate) fn workflow_context_query_identity(
    query_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "workflow_context_query_v1",
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("query"), query_identity)
            .seal()?,
    )
}

pub(crate) fn workflow_context_basis_identity(
    basis_family: &WorkflowBasisFamily,
    basis_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "workflow_context_basis_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("basis_family"),
                basis_family.as_str(),
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("basis"), basis_identity)
            .seal()?,
    )
}

pub(crate) fn workflow_canonical_query_digest_evidence(
    digest: &CanonicalQueryDigest,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "workflow_canonical_query_digest_evidence_v1",
            )
            .field_value(
                WorthQueryEvidenceTag::new("canonical_query_digest"),
                digest.as_str(),
            )
            .seal()?,
    )
}

pub(crate) fn workflow_validated_query_digest_evidence(
    digest: &ValidatedQueryDigest,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "workflow_validated_query_digest_evidence_v1",
            )
            .field_value(
                WorthQueryEvidenceTag::new("validated_query_digest"),
                digest.as_str(),
            )
            .seal()?,
    )
}

pub(super) fn workflow_plan_digest_evidence(
    digest: &PlanDigest,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "workflow_plan_digest_evidence_v1",
            )
            .field_value(WorthQueryEvidenceTag::new("plan_digest"), digest.as_str())
            .seal()?,
    )
}

pub(super) fn workflow_declaration_identity(
    binding_identity: &WorthQueryEvidenceIdentity,
    request: &WorkflowDeclarationRequest,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "workflow_declaration_v1",
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("binding"), binding_identity)
            .field_shape(
                WorthQueryEvidenceTag::new("declaration_family"),
                request.declaration_family().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("authority_target_family"),
                request.authority_target_family().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("cost_class"),
                request.cost_class().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("budget_class"),
                request.budget_class().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("freshness_policy"),
                request.freshness_policy().as_str(),
            )
            .seal()?,
    )
}
