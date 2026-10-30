#[allow(unused_imports)]
use crate::physical_record_journeys::durability_admission::managed_mutation::usable_growth::*;

use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::selected_segment_rewrite::completed, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::selected_segment_rewrite::prepare_rewrite};
use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};

#[test]
fn append_is_denied_one_byte_over_usable_growth() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (format, placement, _) = configuration();
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    serving.certification_limit_candidate_growth_bytes(page_bytes.saturating_sub(1));
    let writes = positioned_writes(&serving);
    let outcome = prepare(&serving, placement, [41; 32], b"growth-append").execute();
    assert_denied_before_effects(&serving, outcome, writes, "one-byte-over append");
    serving.close();
}

#[test]
fn append_is_denied_when_usable_growth_is_only_the_data_page() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (format, placement, _) = configuration();
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    serving.certification_limit_candidate_growth_bytes(page_bytes);
    let writes = positioned_writes(&serving);
    let outcome = prepare(&serving, placement, [46; 32], b"wal-root-growth").execute();
    assert_denied_before_effects(
        &serving,
        outcome,
        writes,
        "append without WAL and root budget",
    );
    serving.close();
}

#[test]
fn selected_segment_rewrite_is_denied_one_byte_over_usable_growth() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (format, placement, _) = configuration();
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    completed(prepare(&serving, placement, [39; 32], b"growth-source").execute());
    serving.certification_limit_candidate_growth_bytes(page_bytes.saturating_sub(1));
    let writes = positioned_writes(&serving);
    let outcome = prepare_rewrite(&serving, placement, [40; 32]).execute();
    assert_denied_before_effects(&serving, outcome, writes, "one-byte-over rewrite");
    serving.close();
}
