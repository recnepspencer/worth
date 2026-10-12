use super::throughput_definition::{
    assert_profile_report, operational_digest_for, ordinary_definition, profiles,
    ORDINARY_OUTPUT_FLOOR, PERFORMANCE_BATCHES,
};

#[test]
fn ordinary_six_profile_courtroom_preserves_operational_truth() {
    let definition = ordinary_definition();
    let mut reports = Vec::new();
    let mut digests = Vec::new();
    for profile in profiles() {
        let (report, digest, inventory) =
            operational_digest_for(profile, definition.clone(), PERFORMANCE_BATCHES);
        assert_profile_report(&report, ORDINARY_OUTPUT_FLOOR as usize, PERFORMANCE_BATCHES);
        if profile.expects_optional_observation() {
            assert!(
                !inventory.is_idle_zero(),
                "{} must capture requested observation surfaces: {inventory:?}",
                profile.name
            );
        } else {
            assert!(
                inventory.is_idle_zero(),
                "{} must not retain optional observation work: {inventory:?}",
                profile.name
            );
        }
        reports.push((profile.name, report.semantic_work_rows.clone()));
        digests.push((profile.name, digest));
    }
    let first_digest = digests[0].1;
    let first_work = &reports[0].1;
    for (name, digest) in &digests[1..] {
        assert_eq!(*digest, first_digest, "operational digest drift in {name}");
    }
    for (name, work) in &reports[1..] {
        assert_eq!(work, first_work, "semantic work identity drift in {name}");
    }
}
