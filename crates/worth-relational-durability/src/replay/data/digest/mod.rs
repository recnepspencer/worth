mod certification_terms;
pub mod descriptor_terms;
pub mod lineage_terms;
pub mod primitive_terms;
pub mod surface_terms;

pub use certification_terms::{
    digest_diagnostics_batch_surface, digest_patch_batch_surface,
    digest_schema_transition_decision, digest_subscriber_boundary_cdc_surface,
    digest_subscriber_continuation_counter_pair, digest_subscriber_continuation_summary,
};
pub use descriptor_terms::{
    digest_schema_continuation_descriptor, digest_schema_continuation_summary,
    digest_schema_lineage_summary, digest_schema_reconciliation_descriptor,
    digest_schema_reconciliation_summary, digest_schema_transition_descriptor,
    digest_schema_transition_summary,
};
pub use lineage_terms::{
    digest_lineage_decision_log_surface, digest_lineage_decision_summary,
    digest_lineage_event_batch_surface, digest_lineage_event_summary,
};
pub(crate) use surface_terms::digest_patch_surface;
pub use surface_terms::{
    digest_branch_head_summary, digest_branch_head_surface, digest_canonical_patch_summary,
    digest_canonical_patch_surface, digest_derived_index_summary, digest_derived_index_surface,
    digest_diagnostics_summary, digest_diagnostics_surface, digest_history_summary,
    digest_history_surface, digest_snapshot_summary, digest_snapshot_surface,
    digest_strategy_replay_descriptor, digest_strategy_replay_summary,
};

#[cfg(test)]
pub mod tests {
    use super::{digest_history_surface, primitive_terms::ReplayDigestBuilder};
    use worth_relational_contracts::internal_port::history::data::commit_receipt::OrderedParentList;
    use worth_relational_contracts::internal_port::history::data::coordinates::{
        BranchId, CommitId,
    };

    #[test]
    fn replay_digest_domain_tags_separate_equal_primitive_bytes() {
        let first = ReplayDigestBuilder::new("domain.one").u64(7).finish();
        let second = ReplayDigestBuilder::new("domain.two").u64(7).finish();

        assert_ne!(first, second);
    }

    #[test]
    fn replay_history_digest_respects_authoritative_parent_order() {
        let ordered = digest_history_surface(
            &OrderedParentList::from_authoritative(vec![CommitId(1), CommitId(2)]),
            &[BranchId("feature".to_string())],
            &[CommitId(9)],
        );
        let reversed = digest_history_surface(
            &OrderedParentList::from_authoritative(vec![CommitId(2), CommitId(1)]),
            &[BranchId("feature".to_string())],
            &[CommitId(9)],
        );

        assert_ne!(ordered, reversed);
    }
}
