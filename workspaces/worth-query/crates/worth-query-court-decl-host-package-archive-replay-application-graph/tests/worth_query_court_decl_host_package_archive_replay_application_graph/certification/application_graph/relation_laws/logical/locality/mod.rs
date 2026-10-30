pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::relation_laws::logical::locality::*;

#[test]
fn logical_local_rule_business_acquisitions_ignore_ten_and_ten_thousand_unrelated_cohorts() {
    assert_eq!(
        run(10),
        run(10_000),
        "actual local preparation/evaluation business counts, not charged work"
    );
}
