#[allow(unused_imports)] use crate::physical_media_journeys::partial_effects::*;
use {std::path::Path};

use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::MediaShutdownOutcome};

use {crate::child_dispatch::emit, crate::child_dispatch::run_role};
use {crate::physical_media_journeys::admit_runtime, crate::physical_media_journeys::media_admission};






use {crate::partial_effects::abrupt_writer::event, crate::partial_effects::abrupt_writer::run_abrupt_writer};
use {crate::physical_media_journeys::partial_effects::counter_evidence::assert_counter_conservation, crate::physical_media_journeys::partial_effects::counter_evidence::emit_counter_report, crate::physical_media_journeys::partial_effects::counter_evidence::inspection_counters};
use {crate::partial_effects::fault_cases::FaultCase, crate::partial_effects::fault_cases::CI_CASES, crate::partial_effects::fault_cases::RELEASE_CASES};
use {crate::physical_media_journeys::partial_effects::independent_observation::expected_manifest, crate::physical_media_journeys::partial_effects::independent_observation::manifest_projection, crate::physical_media_journeys::partial_effects::independent_observation::observe_namespace, crate::physical_media_journeys::partial_effects::independent_observation::observe_tree};

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
