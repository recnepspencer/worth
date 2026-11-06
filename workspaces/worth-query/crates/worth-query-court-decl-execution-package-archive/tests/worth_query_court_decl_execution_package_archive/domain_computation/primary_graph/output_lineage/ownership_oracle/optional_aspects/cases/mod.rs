use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::*;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::OptionalSchema;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::RequiredSchema;
#[test]
fn optional_aspects_archive_birth_preserve_and_cold_complete_roster() {
    crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::archive::roundtrip::<OptionalSchema>();
    let runtime = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::install::<OptionalSchema>(false);
    let initial = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::journey::demand(&runtime, true).expect("actual all-Create producer settles");
    let delivery = initial.readiness_delivery().unwrap();
    assert!(delivery.has_conditional_successor());
    assert!(delivery.semantic_observation_reads() > 0);
    let ids = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::court::readmit(&runtime, &initial, true);
    assert_eq!(ids.0.len(), 4);
    crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::court::edit_source(&runtime);
    let kept = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::journey::demand(&runtime, true).expect("actual successor Preserve settles");
    assert!(!kept.is_stably_reused());
    assert_eq!(
        crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::court::readmit(&runtime, &kept, true),
        ids,
        "complete roster retains original IDs, generations and native incidence"
    );
}
#[test]
fn explicitly_required_whole_aspect_refuses_empty_birth() {
    let runtime = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::install::<RequiredSchema>(false);
    let denial = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::journey::demand(&runtime, true)
        .err()
        .expect("Required whole absence stays illegal");
    assert_eq!(
        denial.commit_denial_kind(),
        Some(WorthQueryApplicationCommitDenialKind::ProviderRejected)
    );
    let detail = format!("{denial:?}");
    assert!(
        detail.contains("empty-payload-with-executable-bindings"),
        "native publication first stop: {detail}"
    );
}
#[test]
fn optional_aspect_required_field_allows_absence_but_refuses_incomplete_present() {
    let absent = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::install::<OptionalSchema>(false);
    crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::required_content_read::assert_absent_required_content_is_readable(&absent);
    let settled =
        crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::journey::demand(&absent, true).expect("Optional StrictFacts may be wholly absent");
    crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::court::readmit(&absent, &settled, true);
    let incomplete = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::install::<OptionalSchema>(true);
    let denial = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::journey::demand(&incomplete, true)
        .err()
        .expect("present StrictFacts lacks Mandatory");
    assert_eq!(
        denial.commit_denial_kind(),
        Some(WorthQueryApplicationCommitDenialKind::ProviderRejected)
    );
    assert!(
        format!("{denial:?}").contains("MissingRequiredField"),
        "this is a present-value refusal"
    );
}
#[test]
fn ordinary_query_completion_cannot_acquire_native_producer_custody() {
    let runtime = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::install::<OptionalSchema>(false);
    let managed = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::journey::demand(&runtime, true).expect("valid native ownership baseline");
    let ordinary = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::ordinary::commit(&runtime, "owner-source:b");
    crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::court::ordinary::assert_ordinary(&runtime, &managed, &ordinary);
}

#[test]
fn required_aspect_absence_cannot_seal_even_when_its_field_is_optional() {
    let runtime = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::install::<RequiredSchema>(false);
    // Ordinary native work has no producer custody and can create the empty
    // physical record; the tracked Required-aspect read must still refuse.
    crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::ordinary::commit(&runtime, crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::ANCHOR);
    crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::ordinary::required_read_refuses(&runtime);
}
