use super::*;
use installation::{OptionalSchema, RequiredSchema};
#[test]
fn optional_aspects_archive_birth_preserve_and_cold_complete_roster() {
    archive::roundtrip::<OptionalSchema>();
    let runtime = installation::install::<OptionalSchema>(false);
    let initial = journey::demand(&runtime, true).expect("actual all-Create producer settles");
    let delivery = initial.readiness_delivery().unwrap();
    assert!(delivery.has_conditional_successor());
    assert!(delivery.semantic_observation_reads() > 0);
    let ids = court::readmit(&runtime, &initial, true);
    assert_eq!(ids.0.len(), 4);
    court::edit_source(&runtime);
    let kept = journey::demand(&runtime, true).expect("actual successor Preserve settles");
    assert!(!kept.is_stably_reused());
    assert_eq!(
        court::readmit(&runtime, &kept, true),
        ids,
        "complete roster retains original IDs, generations and native incidence"
    );
}
#[test]
fn explicitly_required_whole_aspect_refuses_empty_birth() {
    let runtime = installation::install::<RequiredSchema>(false);
    let denial = journey::demand(&runtime, true)
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
    let absent = installation::install::<OptionalSchema>(false);
    super::required_content_read::assert_absent_required_content_is_readable(&absent);
    let settled =
        journey::demand(&absent, true).expect("Optional StrictFacts may be wholly absent");
    court::readmit(&absent, &settled, true);
    let incomplete = installation::install::<OptionalSchema>(true);
    let denial = journey::demand(&incomplete, true)
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
    let runtime = installation::install::<OptionalSchema>(false);
    let managed = journey::demand(&runtime, true).expect("valid native ownership baseline");
    let ordinary = ordinary::commit(&runtime, "owner-source:b");
    court::assert_ordinary(&runtime, &managed, &ordinary);
}

#[test]
fn required_aspect_absence_cannot_seal_even_when_its_field_is_optional() {
    let runtime = installation::install::<RequiredSchema>(false);
    // Ordinary native work has no producer custody and can create the empty
    // physical record; the tracked Required-aspect read must still refuse.
    ordinary::commit(&runtime, installation::ANCHOR);
    ordinary::required_read_refuses(&runtime);
}
