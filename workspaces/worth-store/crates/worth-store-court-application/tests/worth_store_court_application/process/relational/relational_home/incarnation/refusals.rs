//! Phase 3 Home open refusals as one table (C1.5). Every case first initializes
//! and validly reopens a real directory Home, then damages or contends with it:
//! the open refuses with its own typed reason and issues no Home, and a live
//! Home a competing opener met stays usable.
use crate::durable_run::open;
use std::path::Path;
use worth_execution::ExecutionRequest;
use worth_store_application::{
    ApplicationHomeDescriptorDenial as Descriptor, ApplicationHomeOpenRefusal as Open,
    ApplicationHomePhysicalRefusalKind as Physical, ReadmittedApplicationHome,
};
use worth_store_testkit_application_oracle::facade::home_opening::HomeOpeningRefused as Refused;

type Opened = Result<ReadmittedApplicationHome, Refused>;

struct Case {
    name: &'static str,
    /// Damage the closed Home, or return a live Home the attempt contends with.
    arrange: fn(&Path, &ExecutionRequest<'_, '_>) -> Option<ReadmittedApplicationHome>,
    attempt: fn(&Path, &ExecutionRequest<'_, '_>) -> Opened,
    refused: fn(&Open) -> bool,
}

const CASES: [Case; 7] = [
    Case {
        name: "damaged descriptor version",
        arrange: |location, execution| damage_descriptor(location, execution, "home"),
        attempt: open,
        refused: |cause| {
            matches!(
                cause,
                Open::Descriptor(Descriptor::UnsupportedVersion { observed: 1 })
            )
        },
    },
    Case {
        name: "descriptor under the retired anchor",
        arrange: |location, execution| damage_descriptor(location, execution, "relational.home"),
        attempt: open,
        refused: |cause| {
            matches!(
                cause,
                Open::Descriptor(Descriptor::UnsupportedVersion { observed: 1 })
            )
        },
    },
    Case {
        name: "foreign Home",
        arrange: |_, _| None,
        attempt: open_as_another_runtime,
        refused: |cause| matches!(cause, Open::Descriptor(Descriptor::BindingMismatch)),
    },
    Case {
        name: "unknown root schema",
        arrange: |location, _| {
            schema_eleven(location);
            None
        },
        attempt: open,
        refused: |cause| {
            matches!(cause, Open::Physical(refusal)
                if refusal.kind() == Physical::UnsupportedRootSchema { observed: 11 }
                    && refusal.recovery_effects() == Some(0))
        },
    },
    Case {
        name: "incomplete recovery: record family lost beside other residue",
        arrange: |location, _| {
            for family in ["families/records", "staging/records"] {
                std::fs::remove_dir_all(location.join(family)).unwrap();
            }
            let residue = location.join("families/interrupted-owner");
            std::fs::create_dir_all(&residue).unwrap();
            std::fs::write(residue.join("truth"), b"retained").unwrap();
            None
        },
        attempt: open,
        refused: |cause| matches!(cause, Open::Physical(refusal) if refusal.kind() == Physical::PhysicalStep),
    },
    Case {
        name: "incomplete recovery: only the namespace identity remains",
        arrange: |location, _| {
            for family in ["families", "staging"] {
                std::fs::remove_dir_all(location.join(family)).unwrap();
                std::fs::create_dir(location.join(family)).unwrap();
            }
            None
        },
        attempt: open,
        refused: |cause| matches!(cause, Open::Physical(refusal) if refusal.kind() == Physical::PhysicalStep),
    },
    Case {
        name: "competing opener",
        arrange: |location, execution| Some(open(location, execution).unwrap()),
        attempt: open,
        refused: |cause| matches!(cause, Open::Physical(refusal) if refusal.kind() == Physical::MutationOwnerContended),
    },
];

#[test]
fn each_phase_three_open_refusal_is_typed_and_issues_no_home() {
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    for case in &CASES {
        let parent = tempfile::tempdir().unwrap();
        let location = parent.path().join("home");
        let fresh = open(&location, &execution).expect("an empty directory initializes");
        assert_eq!(fresh.incarnation().get(), 1, "{}", case.name);
        let identity = fresh.home_identity();
        fresh.close(&execution);
        let reopened = open(&location, &execution).expect("a valid Home reopens");
        assert_eq!(reopened.home_identity(), identity, "{}", case.name);
        reopened.close(&execution);
        let live = (case.arrange)(&location, &execution);
        let cause = match (case.attempt)(&location, &execution) {
            Ok(_) => panic!("{}: a refused open issued a serving Home", case.name),
            Err(Refused::BeforeReadmission(cause)) => cause,
            Err(other) => panic!("{}: refusal precedes readmission: {other:?}", case.name),
        };
        assert!((case.refused)(&cause), "{}: {cause:?}", case.name);
        if let Some(live) = live {
            let seed = crate::workload::commit(
                live.relational().runtime(),
                &execution,
                &worth_relational::facade::history::BranchId("main".into()),
                crate::batches::create(0),
            )
            .expect("the contended Home keeps serving");
            live.relational()
                .runtime()
                .snapshots()
                .release_snapshot(&seed.snapshot)
                .unwrap();
            live.close(&execution);
            let after = open(&location, &execution).expect("the live Home closed cleanly");
            assert_eq!(after.home_identity(), identity);
            after.close(&execution);
        }
    }
}

fn damage_descriptor(
    location: &Path,
    execution: &ExecutionRequest<'_, '_>,
    anchor: &str,
) -> Option<ReadmittedApplicationHome> {
    use worth_store::physical_runtime::{
        AnchoredRecordName, AnchoredRecordPublication, PhysicalMutationDeadline,
    };
    let home = open(location, execution).unwrap();
    let records = home.physical_serving_for_test().anchored_records().unwrap();
    let selected = records
        .read(AnchoredRecordName::new("home").unwrap(), 65536)
        .unwrap()
        .unwrap();
    let mut bytes = selected.bytes().to_vec();
    bytes[8..10].copy_from_slice(&1_u16.to_le_bytes());
    let expected = if anchor == "home" {
        selected.version().generation()
    } else {
        0
    };
    records
        .publish(AnchoredRecordPublication::new(
            AnchoredRecordName::new(anchor).unwrap(),
            expected,
            bytes,
            crate::incarnation::anchor_replay::placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        ))
        .unwrap();
    home.close(execution);
    None
}

fn open_as_another_runtime(location: &Path, execution: &ExecutionRequest<'_, '_>) -> Opened {
    let foreign = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .runtime_setup(|runtime| {
            runtime.runtime_name("another-runtime");
        })
        .schema_setup(|setup| {
            setup.schema_registry(crate::schema_transition::registry(
                crate::schema_transition::SOURCE,
            ));
        });
    worth_store_testkit_application_oracle::facade::home_opening::open(
        worth_store_testkit_application_oracle::facade::home_opening::request(location, foreign),
        execution,
    )
}

/// Every schema-12 root becomes an unsupported schema-11 envelope; nothing is
/// interpreted or migrated.
fn schema_eleven(location: &Path) {
    let roots = location.join("families/records/roots");
    for entry in std::fs::read_dir(roots).unwrap() {
        let file = entry.unwrap().path();
        let mut bytes = std::fs::read(&file).unwrap();
        if bytes.get(9) == Some(&12) {
            bytes[9] = 11;
            std::fs::write(&file, bytes).unwrap();
        }
    }
}
