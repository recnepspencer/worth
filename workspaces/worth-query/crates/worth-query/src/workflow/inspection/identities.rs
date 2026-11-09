#[cfg(test)]
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::super::foundation::QueryWorkflowDeclaration;
#[cfg(test)]
use super::super::lowering::LoweredMergeWorkflowDeclaration;
use super::evidence::WorkflowAuthorityOutcomeArtifact;
use super::model::WorkflowAuthorityOutcomeFamily;

#[cfg(test)]
pub(super) fn conflict_scope_identity(
    declaration: &QueryWorkflowDeclaration,
    merge_declaration: &LoweredMergeWorkflowDeclaration,
    merge_class_family: &str,
    merge_class: &str,
    merge_class_admission: &str,
    row_digest: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "workflow_conflict_scope_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("conflict_declaration"),
                declaration.report().declaration_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("merge_declaration"),
                merge_declaration
                    .declaration()
                    .report()
                    .declaration_identity(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("target"),
                &merge_declaration.merge_request().target_branch().0,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("source"),
                &merge_declaration.merge_request().source_branch().0,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("merge_intent"),
                merge_declaration.merge_intent().as_str(),
            )
            .field_shape(WorthQueryEvidenceTag::new("row_digest"), row_digest)
            .field_shape(
                WorthQueryEvidenceTag::new("merge_class_family"),
                merge_class_family,
            )
            .field_shape(WorthQueryEvidenceTag::new("merge_class"), merge_class)
            .field_shape(
                WorthQueryEvidenceTag::new("merge_class_admission"),
                merge_class_admission,
            )
            .seal()?,
    )
}

#[cfg(test)]
pub(super) fn post_merge_scope_identity(
    declaration: &QueryWorkflowDeclaration,
    outcome: &WorkflowAuthorityOutcomeArtifact,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "workflow_post_merge_scope_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("inspection_declaration"),
                declaration.report().declaration_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("authoritative_outcome"),
                outcome.authoritative_outcome_identity(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                outcome.family().as_str(),
            )
            .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens")
}

pub(super) fn workflow_authority_request_identity(
    family: WorkflowAuthorityOutcomeFamily,
    request_identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "workflow_authority_request_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
            .field_evidence_identity(WorthQueryEvidenceTag::new("request"), request_identity)
            .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens")
}

pub(super) fn workflow_authoritative_outcome_identity(
    declaration: &QueryWorkflowDeclaration,
    family: &WorkflowAuthorityOutcomeFamily,
    authority_request_identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "workflow_authoritative_outcome_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("declaration"),
                declaration.report().declaration_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("binding"),
                declaration.report().binding_identity(),
            )
            .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("authority_request"),
                authority_request_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("basis"),
                declaration.binding().basis_identity(),
            )
            .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens")
}

pub(super) fn delivery_or_failure_identity(
    outcome: &WorkflowAuthorityOutcomeArtifact,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "workflow_delivery_or_failure_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("outcome"),
                outcome.authoritative_outcome_identity(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("freshness"),
                outcome.freshness_outcome().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("budget"),
                outcome.budget_outcome().as_str(),
            )
            .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens")
}

pub(super) fn workflow_replay_bundle_identity(
    outcome: &WorkflowAuthorityOutcomeArtifact,
    delivery_or_failure_identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "workflow_replay_bundle_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("query"),
                outcome.source_query_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("plan"),
                outcome.source_plan_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("basis"),
                outcome.source_basis_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("declaration"),
                outcome.source_declaration_identity(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("authority_target"),
                outcome.authority_target_family().as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("request"),
                outcome.authority_request_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("outcome"),
                outcome.authoritative_outcome_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("delivery"),
                delivery_or_failure_identity,
            )
            .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens")
}
