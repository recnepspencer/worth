use crate::authoring::WorthQueryGraphReadDomainOperationDeclaration;
use crate::domain_installation::WorthQueryInstalledGraphReadOperation;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::planning::ExecutionPlanBundle;
use crate::policy_plan::PolicyAwareCurrentPlan;
use crate::relationship_proof::RelationshipProofAdmission;

use super::{WorthQueryReadBuiltInOperator, WorthQueryReadGraphFamily, WorthQueryReadScopeClass};

#[allow(clippy::too_many_arguments)]
pub(super) fn derive_read_graph_identity(
    family: &WorthQueryReadGraphFamily,
    scope_class: &WorthQueryReadScopeClass,
    built_in_operators: &[WorthQueryReadBuiltInOperator],
    domain_graph_operations: &[WorthQueryGraphReadDomainOperationDeclaration],
    installed_operation_bindings: &[WorthQueryInstalledGraphReadOperation],
    declared_traversal_clause_count: usize,
    declared_traversal_depth_limit: usize,
    relationship_proof_admission: Option<&RelationshipProofAdmission>,
    policy_aware_plan: Option<&PolicyAwareCurrentPlan>,
    execution_plan: &ExecutionPlanBundle,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let family_label = match family {
        WorthQueryReadGraphFamily::Detail => "detail",
        WorthQueryReadGraphFamily::Collection => "collection",
    };
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::ReadGraphDigest)
            .field_shape(WorthQueryEvidenceTag::new("family"), family_label)
            .field_shape(WorthQueryEvidenceTag::new("scope"), scope_class.as_str())
            .field_value(
                WorthQueryEvidenceTag::new("plan"),
                execution_plan.query().plan_digest().as_str(),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("built_in_operator"),
                built_in_operators
                    .iter()
                    .map(WorthQueryReadBuiltInOperator::as_str),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("domain_operation"),
                domain_graph_operations
                    .iter()
                    .map(WorthQueryGraphReadDomainOperationDeclaration::digest_part),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("installed_operation_authority"),
                installed_operation_bindings
                    .iter()
                    .map(|binding| binding.authority().witness_identity().as_str()),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("declared_traversal_count"),
                declared_traversal_clause_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("declared_traversal_depth"),
                declared_traversal_depth_limit,
            )
            .optional_value(
                WorthQueryEvidenceTag::new("relationship_proof_admission"),
                relationship_proof_admission.map(|admission| admission.identity().as_str()),
            )
            .optional_value(
                WorthQueryEvidenceTag::new("policy_aware_plan"),
                policy_aware_plan.map(|plan| plan.core().digest().as_str()),
            )
            .seal()?,
    )
}
