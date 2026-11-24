use std::sync::Arc;

use crate::merge::data::{LoweredMergePlan, MergePolicyAspectDigestRow, MergePolicyDigestBasis};

pub(super) fn merge_policy_digest_basis(plan: &LoweredMergePlan) -> MergePolicyDigestBasis {
    MergePolicyDigestBasis {
        records: Arc::from(
            plan.policy_records
                .iter()
                .map(|record| record.record.clone())
                .collect::<Vec<_>>(),
        ),
        proof_boundaries: Arc::from(
            plan.policy_records
                .iter()
                .map(|record| record.proof_boundary)
                .collect::<Vec<_>>(),
        ),
        applied_policies: Arc::from(
            plan.policy_records
                .iter()
                .map(|record| record.applied_policies.clone())
                .collect::<Vec<_>>(),
        ),
        aspect_rows: Arc::from(
            plan.policy_records
                .iter()
                .map(policy_aspect_rows_for_digest)
                .collect::<Vec<_>>(),
        ),
    }
}

fn policy_aspect_rows_for_digest(
    record: &crate::merge::data::MergePolicyResolutionRecord,
) -> Arc<[MergePolicyAspectDigestRow]> {
    Arc::from(
        record
            .aspect_resolutions
            .iter()
            .map(|aspect| MergePolicyAspectDigestRow {
                aspect_key: aspect.aspect_key.clone(),
                comparison: aspect.comparison,
                applied_policy: aspect.applied_policy.clone(),
                policy_ownership: aspect
                    .applied_policy
                    .as_ref()
                    .map(|policy| policy.ownership_class()),
                decision_boundary: aspect.decision_boundary,
                resolved_value_strategy: aspect.resolved_value_strategy.clone(),
            })
            .collect::<Vec<_>>(),
    )
}
