use crate::merge::data::LoweredMergePlan;

pub(super) fn merge_request_summary(plan: &LoweredMergePlan) -> String {
    format!(
        "{}:{}:{:?}:{:?}:{:?}:{:?}",
        plan.request.target_branch().0,
        plan.request.source_branch().0,
        plan.request.merge_intent(),
        plan.request.correspondence_posture(),
        plan.request.schema_reconciliation_posture(),
        plan.request.topology_intent()
    )
}

pub(super) fn merge_ancestry_summary(plan: &LoweredMergePlan) -> String {
    format!(
        "base:{};target_commits:{};source_commits:{};target_records:{};source_records:{};identity_candidates:{};exact:{};missing:{};validated_schema_correspondences:{};classified_records:{};concurrent:{};source_only:{};policy_auto:{};policy_reject:{};lowered_admitted:{};lowered_blocked:{};lowered_rejected:{}",
        plan.ancestry.merge_base_commit_id.0,
        plan.ancestry.target.unique_commit_count,
        plan.ancestry.source.unique_commit_count,
        plan.ancestry.target.touched_record_count,
        plan.ancestry.source.touched_record_count,
        plan.identity_summary.candidate_count,
        plan.identity_summary.exact_match_count,
        plan.identity_summary.missing_target_count,
        plan.identity_summary
            .schema_declared_correspondence
            .validated_count,
        plan.conflict_summary.classified_record_count,
        plan.causal_summary.concurrent_count,
        plan.causal_summary.source_only_count,
        plan.policy_summary.auto_resolved_count,
        plan.policy_summary.reject_count,
        plan.lowered_summary.admitted_count,
        plan.lowered_summary.blocked_count,
        plan.lowered_summary.rejected_count
    )
}
