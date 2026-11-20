//! Phase 3 Home open refusals by descriptor, ownership, schema, recovery state
//! and contention, as one table (C1.5; damaged checkpoint and journal bodies
//! stay in `open_refusals`). Every case first initializes, acknowledges one
//! commit and validly reopens a real directory Home, then damages or contends
//! with it: the open refuses with its own typed reason, issues no Home and
//! leaves the stored files and acknowledged data as they were.
use crate::durable_run::open;
use std::path::{Path, PathBuf};
use worth_execution::ExecutionRequest;
use worth_store_application::{
    ApplicationHomeDescriptorDenial as Descriptor, ApplicationHomeOpenRefusal as Open,
    ApplicationHomePhysicalRefusalKind as Physical, ReadmittedApplicationHome,
};
use worth_store_testkit_application_oracle::facade::home_opening::HomeOpeningRefused as Refused;

type Opened = Result<ReadmittedApplicationHome, Refused>;

/// What an arrangement leaves for the refused open to keep intact.
#[derive(Default)]
struct Arranged {
    /// A live Home the attempt contends with; it keeps serving afterwards.
    live: Option<ReadmittedApplicationHome>,
    /// Files whose bytes the refusal must not change.
    unchanged: Vec<(PathBuf, Vec<u8>)>,
    /// Paths the refusal must not create.
    absent: Vec<&'static str>,
}

struct Case {
    name: &'static str,
    arrange: fn(&Path, &ExecutionRequest<'_, '_>) -> Arranged,
    attempt: fn(&Path, &ExecutionRequest<'_, '_>) -> Opened,
    refused: fn(&Open) -> bool,
    /// The Home's own runtime still reads back the acknowledged data.
    readable_after: bool,
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
        readable_after: false,
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
        readable_after: false,
    },
    Case {
        name: "foreign Home",
        arrange: |_, _| Arranged::default(),
        attempt: open_as_another_runtime,
        refused: |cause| matches!(cause, Open::Descriptor(Descriptor::BindingMismatch)),
        readable_after: true,
    },
    Case {
        name: "unknown root schema",
        arrange: |location, _| Arranged {
            unchanged: schema_eleven(location),
            ..Arranged::default()
        },
        attempt: open,
        refused: |cause| {
            matches!(cause, Open::Physical(refusal)
                if refusal.kind() == Physical::UnsupportedRootSchema { observed: 11 }
                    && refusal.recovery_effects() == Some(0))
        },
        readable_after: false,
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
            Arranged {
                unchanged: kept([residue.join("truth")]),
                absent: vec!["families/records"],
                ..Arranged::default()
            }
        },
        attempt: open,
        refused: |cause| matches!(cause, Open::Physical(refusal) if refusal.kind() == Physical::PhysicalStep),
        readable_after: false,
    },
    Case {
        name: "incomplete recovery: only the namespace identity remains",
        arrange: |location, _| {
            for family in ["families", "staging"] {
                std::fs::remove_dir_all(location.join(family)).unwrap();
                std::fs::create_dir(location.join(family)).unwrap();
            }
            Arranged {
                unchanged: kept([location.join("namespace/identity")]),
                absent: vec!["families/records"],
                ..Arranged::default()
            }
        },
        attempt: open,
        refused: |cause| matches!(cause, Open::Physical(refusal) if refusal.kind() == Physical::PhysicalStep),
        readable_after: false,
    },
    Case {
        name: "competing opener",
        arrange: |location, execution| Arranged {
            live: Some(open(location, execution).unwrap()),
            ..Arranged::default()
        },
        attempt: open,
        refused: |cause| matches!(cause, Open::Physical(refusal) if refusal.kind() == Physical::MutationOwnerContended),
        readable_after: true,
    },
];

#[test]
fn each_phase_three_open_refusal_is_typed_issues_no_home_and_keeps_the_store() {
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    for case in &CASES {
        let parent = tempfile::tempdir().unwrap();
        let location = parent.path().join("home");
        let fresh = open(&location, &execution).expect("an empty directory initializes");
        assert_eq!(fresh.incarnation().get(), 1, "{}", case.name);
        let identity = fresh.home_identity();
        commit(&fresh, &execution, 0);
        fresh.close(&execution);
        let reopened = open(&location, &execution).expect("a valid Home reopens");
        assert_eq!(reopened.home_identity(), identity, "{}", case.name);
        let mut acknowledged = artifacts(&reopened, &execution);
        assert_eq!(acknowledged.len(), 1, "{}", case.name);
        reopened.close(&execution);
        let arranged = (case.arrange)(&location, &execution);
        let cause = match (case.attempt)(&location, &execution) {
            Ok(_) => panic!("{}: a refused open issued a serving Home", case.name),
            Err(Refused::BeforeReadmission(cause)) => cause,
            Err(other) => panic!("{}: refusal precedes readmission: {other:?}", case.name),
        };
        assert!((case.refused)(&cause), "{}: {cause:?}", case.name);
        for (path, bytes) in &arranged.unchanged {
            assert_eq!(
                &std::fs::read(path).unwrap(),
                bytes,
                "{}: {path:?}",
                case.name
            );
        }
        for path in &arranged.absent {
            assert!(
                !location.join(path).exists(),
                "{}: created {path}",
                case.name
            );
        }
        if let Some(live) = arranged.live {
            commit(&live, &execution, 1);
            acknowledged = artifacts(&live, &execution);
            live.close(&execution);
        }
        if case.readable_after {
            let after = open(&location, &execution).expect("the Home is still its runtime's");
            assert_eq!(after.home_identity(), identity, "{}", case.name);
            assert_eq!(artifacts(&after, &execution), acknowledged, "{}", case.name);
            after.close(&execution);
        }
    }
}

fn commit(home: &ReadmittedApplicationHome, execution: &ExecutionRequest<'_, '_>, round: usize) {
    let seed = crate::workload::commit(
        home.relational().runtime(),
        execution,
        &worth_relational::facade::history::BranchId("main".into()),
        crate::batches::create(round),
    )
    .expect("the Home serves");
    home.relational()
        .runtime()
        .snapshots()
        .release_snapshot(&seed.snapshot)
        .unwrap();
}

fn artifacts(
    home: &ReadmittedApplicationHome,
    execution: &ExecutionRequest<'_, '_>,
) -> Vec<Vec<u8>> {
    crate::workload::artifacts_of(execution, home.relational().runtime())
}

fn kept<const N: usize>(paths: [PathBuf; N]) -> Vec<(PathBuf, Vec<u8>)> {
    paths
        .into_iter()
        .map(|path| {
            let bytes = std::fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect()
}

fn damage_descriptor(
    location: &Path,
    execution: &ExecutionRequest<'_, '_>,
    anchor: &str,
) -> Arranged {
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
    Arranged::default()
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
/// interpreted, migrated or rewritten.
fn schema_eleven(location: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let roots = location.join("families/records/roots");
    let mut files: Vec<PathBuf> = std::fs::read_dir(roots)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    files.sort();
    for file in &files {
        let mut bytes = std::fs::read(file).unwrap();
        if bytes.get(9) == Some(&12) {
            bytes[9] = 11;
            std::fs::write(file, bytes).unwrap();
        }
    }
    files
        .into_iter()
        .map(|file| {
            let bytes = std::fs::read(&file).unwrap();
            (file, bytes)
        })
        .collect()
}
