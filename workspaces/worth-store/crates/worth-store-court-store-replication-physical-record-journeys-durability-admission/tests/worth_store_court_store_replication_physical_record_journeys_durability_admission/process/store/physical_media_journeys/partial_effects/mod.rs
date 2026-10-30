#[allow(unused_imports)]
use crate::physical_media_journeys::partial_effects::*;

use {
    crate::partial_effects::fault_cases::CI_CASES,
    crate::partial_effects::fault_cases::RELEASE_CASES,
};

#[test]
fn partial_effects_and_barrier_honesty() {
    let parent = tempfile::tempdir().unwrap();
    for case in CI_CASES {
        println!("C4_SEAM {} run=one", case.id);
        let first = run_case(&parent.path().join(format!("{}-one", case.id)), case);
        println!("C4_SEAM {} run=two", case.id);
        let second = run_case(&parent.path().join(format!("{}-two", case.id)), case);
        assert_eq!(
            first, second,
            "fault schedule {} was not deterministic",
            case.id
        );
    }
    if std::env::var_os("WORTH_STORE_C4_FULL_SEAMS").is_some() {
        for case in RELEASE_CASES {
            println!("C4_SEAM {} run=one", case.id);
            let first = run_case(&parent.path().join(format!("{}-one", case.id)), case);
            println!("C4_SEAM {} run=two", case.id);
            let second = run_case(&parent.path().join(format!("{}-two", case.id)), case);
            assert_eq!(first, second, "release seam {} drifted", case.id);
        }
    }
}
