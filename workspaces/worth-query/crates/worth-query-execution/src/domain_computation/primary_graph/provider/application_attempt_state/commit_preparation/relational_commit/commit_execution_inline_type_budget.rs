//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryCommittedApplicationSession>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/provider/application_attempt_state/commit_preparation/relational_commit/commit_execution.rs", "WorthQueryCommittedApplicationSession", 128 * 1024);
        owner.field::<WorthQueryPrimaryGraphApplicationAttempt>("attempt");
        owner.field::<WorthQueryPrimaryMutationWorkCounters>("work");
        owner.field::<Option<crate::domain_computation::primary_graph::application_query::derived_view::PredicatePreparationOwner>>("predicate");
        owner.field::<worth_relational::facade::indexes::DerivedIndexMaintenanceWork>(
            "index_maintenance_work",
        );
        owner.field::<Option<crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage>>("retained_preimage");
        owner.field::<WorthQueryPreImageRetentionWork>("preimage_retention_work");
        owner.field::<worth_relational::facade::snapshots::SnapshotHandle>("before");
        owner
            .field::<worth_relational::facade::branch::AdmittedRelationalBranchBasis>("next_basis");
        owner.field::<std::sync::Arc<worth_relational::facade::transactions::CommitResult>>(
            "committed",
        );
        owner.field::<PublishedSnapshotCustody>("published_snapshot_custody");
        owner.field::<Option<PreparedTouchedRecords>>("prepared_touched_records");
        owner.field::<crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationReceipt>("product_publication");
        owner.field::<Option<managed_views::PreparedViewPublication>>("managed_views");
        owner.field::<Option<super::super::PreparedSourceFactRebase>>("source_fact_rebase");
        owner
            .field::<Option<crate::domain_computation::primary_graph::PreparedPrerequisiteClaims>>(
                "required_prerequisites",
            );
        owner.field::<Option<
            crate::domain_computation::primary_graph::output_lineage::PreparedOutputLineageSlot,
        >>("prepared_lineage_slot");
        owner.field::<Option<
            crate::domain_computation::primary_graph::output_lineage::PreparedNativeOutputWitness,
        >>("prepared_output_witness");
    }
}
