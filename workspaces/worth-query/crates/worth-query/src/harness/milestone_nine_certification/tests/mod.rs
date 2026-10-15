mod admission;
mod phase_four;

use super::MilestoneNineCertificationBundle;
use crate::harness::certification::digest_parts;

fn semantic_signature(bundle: &MilestoneNineCertificationBundle) -> String {
    digest_parts(&[
        format!("query:{}", bundle.canonical_query_digest),
        format!("policy:{}", bundle.policy_digest),
        format!("result:{}", bundle.result_digest),
        format!("tenant_truth:{}", bundle.tenant_truth_basis_digest),
        format!("tenant_schema:{}", bundle.tenant_schema_basis_digest),
        format!("branch:{}", bundle.branch_access_digest),
        format!("schema:{}", bundle.schema_variant_digest),
        format!("authorized:{}", bundle.authorized_projection_digest),
        format!("shape:{}", bundle.narrowed_result_shape_digest),
        format!("proof:{}", bundle.relationship_proof_digest),
        format!("plan:{}", bundle.policy_plan_digest),
        format!("seam:{}", bundle.policy_execution_seam_digest),
        format!("delivery:{}", bundle.delivery_digest),
    ])
}
