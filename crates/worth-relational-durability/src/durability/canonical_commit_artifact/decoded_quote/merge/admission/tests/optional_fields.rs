//! Missing actual None fields preserve the same owner-validated merge meaning.
use crate::durability::canonical_commit_artifact::decoded_quote::omission_for_test::{
    omit_nil_record_field, replace_record,
};
use contracts::merge::data::artifacts::schema_reconciliation_witness_rows::RelationalSchemaReconciliationWitnessRow;
use contracts::merge::data::policy::AspectPolicyResolutionRecord;
use contracts::transactions::data::outcomes::plan_artifacts::PublishedMergeExecutionAuthority;
use worth_relational_contracts::internal_port as contracts;

pub(super) fn omit(source: &PublishedMergeExecutionAuthority) -> Vec<u8> {
    let row = &source
        .execution_summary
        .schema_reconciliation_witness
        .rows()[0];
    assert!(row.correspondence_linkage().is_none());
    let changed = omit_nil_record_field(row, "correspondence_linkage", 0);
    assert_eq!(
        &rmp_serde::from_slice::<RelationalSchemaReconciliationWitnessRow>(&changed).unwrap(),
        row
    );
    let body = rmp_serde::to_vec_named(source).unwrap();
    let body = replace_record(&body, row, &changed);
    let resolution = &source
        .execution_summary
        .strategy_witness
        .aspect_policy_rows()[0]
        .aspect_resolutions()[0];
    assert!(resolution.applied_policy.is_none());
    let changed = omit_nil_record_field(resolution, "applied_policy", 0);
    assert_eq!(
        &rmp_serde::from_slice::<AspectPolicyResolutionRecord>(&changed).unwrap(),
        resolution
    );
    replace_record(&body, resolution, &changed)
}
