use super::{help_requested, Arguments};
use crate::product::{CiTestLane, TestProduct};

#[test]
fn owner_requires_and_retains_its_package() {
    let parsed = Arguments::parse(["owner".into(), "-p".into(), "worth-store".into()]).unwrap();
    assert_eq!(
        parsed.product,
        TestProduct::Owner {
            package: "worth-store".into()
        }
    );
}

#[test]
fn owner_without_a_package_is_rejected() {
    assert_eq!(
        Arguments::parse(["owner".into()]),
        Err("owner requires -p <package>".into())
    );
}

#[test]
fn ci_requires_a_partition_and_validates_shards() {
    let parsed = Arguments::parse([
        "ci".into(),
        "--partition".into(),
        "scenario".into(),
        "--shard-index".into(),
        "1".into(),
        "--shard-count".into(),
        "3".into(),
    ])
    .unwrap();
    assert_eq!(
        parsed.product,
        TestProduct::Ci {
            lane: CiTestLane::Scenario,
            shard: Some((1, 3))
        }
    );
    assert!(Arguments::parse([
        "ci".into(),
        "--partition".into(),
        "scenario".into(),
        "--shard-index".into(),
        "3".into(),
        "--shard-count".into(),
        "3".into(),
    ])
    .is_err());

    let process =
        Arguments::parse(["ci".into(), "--partition".into(), "process-scenario".into()]).unwrap();
    assert_eq!(
        process.product,
        TestProduct::Ci {
            lane: CiTestLane::ProcessScenario,
            shard: None,
        }
    );
}

#[test]
fn shard_arguments_must_be_supplied_as_a_pair() {
    for lone_argument in ["--shard-index", "--shard-count"] {
        let arguments = [
            "ci".into(),
            "--partition".into(),
            "scenario".into(),
            lone_argument.into(),
            "1".into(),
        ];
        assert_eq!(
            Arguments::parse(arguments),
            Err("--shard-index and --shard-count must be supplied together".into())
        );
    }
}

#[test]
fn ordinary_options_are_retained() {
    let parsed = Arguments::parse([
        "smoke".into(),
        "--list".into(),
        "--target-root".into(),
        "target/store".into(),
    ])
    .unwrap();
    assert!(parsed.list);
    assert_eq!(
        parsed.target_root.unwrap(),
        std::path::Path::new("target/store")
    );
}

#[test]
fn help_is_only_a_top_level_request() {
    assert!(help_requested(&["-h".into()]));
    assert!(help_requested(&["--help".into()]));
    assert!(!help_requested(&["smoke".into(), "--help".into()]));
}
