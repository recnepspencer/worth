//! A recovered original's release killed before its publication stays
//! retained; killed after it commits it stays released and is never reclaimed.
use super::*;
const RELEASE: &str = "WORTH_WORLD_RELEASE_CUT";
const RELEASE_CHILD: &str =
    "process::application::world_durable_release::release_cut::recovered_release_child";

#[test]
fn a_killed_release_retains_before_publication_and_stays_released_after_commit() {
    for cut in ["before", "after"] {
        let directory = tempfile::tempdir().unwrap();
        let location = directory.path().join("home");
        let markers = directory.path().join("markers");
        std::fs::create_dir(&markers).unwrap();
        let environment = [(LOCATION, location.as_path()), (MARKERS, markers.as_path())];
        assert_eq!(
            crate::killed_child::run_and_kill(
                CHILD,
                &environment,
                &markers.join("ready"),
                &markers.join("finished")
            ),
            crate::killed_child::ChildEnd::KilledAtPark
        );
        std::fs::remove_file(markers.join("ready")).unwrap();
        let mut environment = environment.to_vec();
        environment.push((RELEASE, std::path::Path::new(cut)));
        assert_eq!(
            crate::killed_child::run_and_kill(
                RELEASE_CHILD,
                &environment,
                &markers.join("ready"),
                &markers.join("finished")
            ),
            crate::killed_child::ChildEnd::KilledAtPark,
            "{cut}"
        );
        let original = std::fs::read_to_string(markers.join("original")).unwrap();
        let custody: u64 = original.lines().nth(2).unwrap().parse().unwrap();
        let receipt = std::fs::read_to_string(markers.join("receipt")).unwrap();
        let request = crate::workload::serial_request();
        let execution = ExecutionRequest::serial(&request);
        let fixture = open(&location, &execution);
        let recovery = fixture.home.world().recovery_port();
        let rows = fixture
            .home
            .world()
            .inspection_port()
            .recovery_page(None, maximum())
            .unwrap();
        if cut == "before" {
            assert_eq!(
                rows.rows().len(),
                1,
                "an unpublished release retains the original"
            );
            let claim = recovery
                .inspect_recovered_effects(rows.rows()[0].handle())
                .unwrap();
            assert_eq!(claim.identity().ordinal(), custody);
            assert_eq!(format!("{:?}", claim.native_receipt().unwrap()), receipt);
            let outcome = recovery.release_recovered_effects(&execution, claim);
            drop((released(&recovery, &execution, outcome), rows));
        } else {
            assert!(
                rows.rows().is_empty(),
                "a committed release is never reclaimed"
            );
            drop(rows);
        }
        close(fixture.home, &execution);
        let fixture = open(&location, &execution);
        let world = fixture.home.world();
        assert!(world
            .inspection_port()
            .recovery_page(None, maximum())
            .unwrap()
            .rows()
            .is_empty());
        let page = world
            .recovery_port()
            .released_original_page(&execution, None, maximum())
            .unwrap();
        let [original] = page.rows() else {
            panic!("{cut}: exactly the one released original");
        };
        assert_eq!(original.identity().ordinal(), custody);
        assert_eq!(format!("{:?}", original.native_receipt().unwrap()), receipt);
        assert_eq!(
            world
                .observation_port()
                .observe_root_product_branch()
                .unwrap()
                .reference_generation()
                .get(),
            0
        );
        assert_eq!(
            std::fs::read_to_string(markers.join("handler-runs")).unwrap(),
            "1"
        );
        drop(page);
        close(fixture.home, &execution);
    }
}

fn released(
    recovery: &RuntimeWorldRecoveryPort,
    execution: &ExecutionRequest<'_, '_>,
    mut outcome: WorldRecoveredReleaseOutcome,
) -> WorldReleasedEffects {
    let backstop = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        match outcome {
            WorldRecoveredReleaseOutcome::Released(released) => return released,
            WorldRecoveredReleaseOutcome::Pending(claim) => {
                assert!(std::time::Instant::now() < backstop);
                std::thread::sleep(std::time::Duration::from_millis(1));
                outcome = recovery.release_recovered_effects(execution, claim);
            }
            other => panic!("the recovered original releases: {other:?}"),
        }
    }
}

#[test]
fn recovered_release_child() {
    let Some(cut) = std::env::var_os(RELEASE) else {
        return;
    };
    let location = std::path::PathBuf::from(std::env::var_os(LOCATION).unwrap());
    let markers = std::path::PathBuf::from(std::env::var_os(MARKERS).unwrap());
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open(&location, &execution);
    let recovery = fixture.home.world().recovery_port();
    let page = fixture
        .home
        .world()
        .inspection_port()
        .recovery_page(None, maximum())
        .unwrap();
    let claim = recovery
        .inspect_recovered_effects(page.rows()[0].handle())
        .unwrap();
    std::fs::write(
        markers.join("receipt"),
        format!("{:?}", claim.native_receipt().unwrap()),
    )
    .unwrap();
    use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint as Seam;
    let seam = match cut.to_str().unwrap() {
        "before" => Seam::BeforeWalAppend,
        "after" => Seam::AfterRootReplacement,
        other => panic!("unknown release cut {other}"),
    };
    let gate = fixture
        .home
        .physical_serving_for_test()
        .certification_pause_physical_mutation_at(seam);
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            let request = crate::workload::serial_request();
            let execution = ExecutionRequest::serial(&request);
            let outcome = recovery.release_recovered_effects(&execution, claim);
            drop(released(&recovery, &execution, outcome));
            Ok(())
        });
        assert!(crate::process_kill::await_mutation_seam(
            &gate,
            &worker,
            std::time::Instant::now() + crate::killed_child::PARK_DEADLINE
        ));
        crate::killed_child::announce(&markers.join("ready"), b"release at its seam");
        crate::killed_child::park();
    });
}
