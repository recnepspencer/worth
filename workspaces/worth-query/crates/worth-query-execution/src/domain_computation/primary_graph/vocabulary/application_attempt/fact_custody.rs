//! Moved whole from `domain_computation::primary_graph::application_attempt::workflow::schema::fact_custody`: the items below and every impl they own.
use worth_relational::facade::runtime::CustomInvariantSemanticVersion;

// The installed identity predates approval custody; retain its name across
// semantic versions so diagnostics preserve the same rule lineage.
pub(in crate::domain_computation::primary_graph) const RULE_ID: &str =
    "worth-query.workflow.publication-immutability";

pub(in crate::domain_computation::primary_graph) const RULE_VERSION:
    CustomInvariantSemanticVersion = CustomInvariantSemanticVersion::new(1, 6);

pub(in crate::domain_computation::primary_graph) const fn fact_custody_receipt_contract(
) -> (&'static str, u16, u16, Option<u64>) {
    (RULE_ID, RULE_VERSION.major, RULE_VERSION.minor, None)
}
