#[allow(unused_imports)]
use crate::physical_media_journeys::mutation_contention::*;

#[test]
fn mutation_contention_confinement_and_readmission() {
    let parent = tempfile::tempdir().unwrap();
    let first = run_campaign(&parent.path().join("campaign-one"));
    let second = run_campaign(&parent.path().join("campaign-two"));
    assert_eq!(first, second);
}
