#[allow(unused_imports)]
use crate::physical_record_journeys::maintenance_interference::artifact_scope::*;

#[test]
fn rewrite_scope_selects_one_four_and_sixteen_artifacts() {
    for count in [1_u32, 4, 16] {
        select_and_rewrite(count);
    }
}

#[test]
fn artifact_scope_above_sixteen_one_page_segments_is_refused_before_media() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = open_one_page_segments(&root);
    let policy = one_page_placement();
    publish_one_record_per_segment(&serving, &policy, 17);
    let writes = serving
        .media_counters()
        .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite);
    let denial = serving
        .record_submission()
        .plan_inline_artifact_rewrites(17)
        .expect_err("seventeen artifacts fit in 256 KiB");
    assert_eq!(
        denial,
        worth_store::physical_runtime::InlineArtifactRewritePlanDenial::PhysicalPressure
    );
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        writes
    );
    serving.close();
}
