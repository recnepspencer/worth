use std::path::Path;

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::MediaShutdownOutcome;

use super::child_dispatch::{emit, run_role};
use super::{admit_runtime, media_admission};

#[path = "partial_effects/abrupt_writer.rs"]
mod abrupt_writer;
#[path = "partial_effects/counter_evidence.rs"]
mod counter_evidence;
#[path = "partial_effects/fault_cases.rs"]
mod fault_cases;
#[path = "partial_effects/fault_schedule.rs"]
mod fault_schedule;
#[path = "partial_effects/independent_observation.rs"]
mod independent_observation;
use abrupt_writer::{event, run_abrupt_writer};
use counter_evidence::{assert_counter_conservation, emit_counter_report, inspection_counters};
use fault_cases::{FaultCase, CI_CASES, RELEASE_CASES};
use independent_observation::{
    expected_manifest, manifest_projection, observe_namespace, observe_tree,
};

#[derive(Debug, PartialEq, Eq)]
struct CaseProjection {
    outcome: String,
    root_exists: bool,
    identity_visible: bool,
    manifest: Vec<String>,
    counter_projection: Option<String>,
    attempted: Option<u64>,
    terminal: Option<u64>,
    partial: Option<u64>,
    indeterminate: Option<u64>,
    fault_role: String,
    fault_ordinal: u64,
    fault_operation: String,
    fault_handle: String,
}

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

fn run_case(root: &Path, case: FaultCase) -> CaseProjection {
    let expected_manifest = expected_manifest(case.manifest);
    let abrupt_boundary = if case.abrupt {
        Some(run_abrupt_writer(root, case.id))
    } else {
        None
    };
    let report = if case.abrupt {
        None
    } else {
        Some(run_role(
            "faulted-media-writer",
            root,
            &[("WORTH_STORE_C4_FAULT_CASE", case.id)],
        ))
    };
    if let Some(report) = &report {
        assert_eq!(
            report.value("outcome"),
            case.expected_outcome,
            "{} returned the wrong typed outcome",
            case.id
        );
        assert_counter_conservation(report);
        assert_fault_localization(report, case);
        if case.expected_outcome == "failed" {
            assert_eq!(
                report.number("live_files"),
                0,
                "{} leaked a file handle",
                case.id
            );
            assert_eq!(
                report.number("live_directories"),
                0,
                "{} leaked a directory handle",
                case.id
            );
            assert_eq!(
                report.number("ownership_releases"),
                report.number("ownership_acquisitions"),
                "{} violated ownership conservation",
                case.id
            );
        }
    }
    if let Some(boundary) = &abrupt_boundary {
        assert_abrupt_localization(boundary, case);
    }
    let root_exists = root.exists();
    let manifest = root_exists.then(|| observe_tree(root));
    let identity_visible = root.join("namespace/identity").is_file();
    assert_eq!(
        identity_visible,
        case.manifest.identity_visible(),
        "{} identity posture",
        case.id
    );
    if case.id == "before-root-creation" {
        assert!(!root_exists);
    }
    if case.id == "short-identity-prefix" {
        let report = report.as_ref().unwrap();
        assert_eq!(report.number("partial"), 1);
        assert_eq!(report.number("positioned_write_partial"), 1);
        assert_eq!(report.number("retry_attempts"), 1);
    }
    if case.id == "directory-barrier-indeterminate" {
        let report = report.as_ref().unwrap();
        assert_eq!(report.number("directory_syncs"), 0);
        assert_eq!(report.number("directory_sync_denied"), 1);
    }
    if identity_visible {
        assert!(observe_namespace(root).contains("namespace_version=1"));
        let reopened = run_role("fault-reopener", root, &[]);
        assert_eq!(reopened.value("outcome"), "success");
        assert_eq!(reopened.value("conserved"), "true");
    }
    let terminal = report.as_ref().map(|report| {
        report.number("completed")
            + report.number("denied")
            + report.number("partial")
            + report.number("indeterminate")
    });
    let manifest = manifest
        .as_deref()
        .map_or_else(Vec::new, manifest_projection);
    assert_eq!(
        manifest, expected_manifest,
        "{} left the wrong exact physical media state",
        case.id
    );
    CaseProjection {
        outcome: report.as_ref().map_or_else(
            || "abrupt-death".into(),
            |report| report.value("outcome").into(),
        ),
        root_exists,
        identity_visible,
        manifest,
        counter_projection: report
            .as_ref()
            .map(|report| report.value("counter_projection").into()),
        attempted: report.as_ref().map(|report| report.number("attempted")),
        terminal,
        partial: report.as_ref().map(|report| report.number("partial")),
        indeterminate: report.as_ref().map(|report| report.number("indeterminate")),
        fault_role: report.as_ref().map_or_else(
            || event_field(abrupt_boundary.as_deref().unwrap(), "role").into(),
            |report| report.value("fault_role").into(),
        ),
        fault_ordinal: report.as_ref().map_or_else(
            || {
                event_field(abrupt_boundary.as_deref().unwrap(), "ordinal")
                    .parse()
                    .unwrap()
            },
            |report| report.number("fault_ordinal"),
        ),
        fault_operation: report.as_ref().map_or_else(
            || event_field(abrupt_boundary.as_deref().unwrap(), "operation").into(),
            |report| report.value("fault_operation").into(),
        ),
        fault_handle: report.as_ref().map_or_else(
            || event_field(abrupt_boundary.as_deref().unwrap(), "handle").into(),
            |report| report.value("fault_handle").into(),
        ),
    }
}

fn assert_fault_localization(report: &super::child_dispatch::ChildReport, case: FaultCase) {
    let Some(expected) = case.expected_fault else {
        assert_eq!(report.number("fault_matches"), 0);
        assert_eq!(report.value("fault_role"), "none");
        assert_eq!(report.value("fault_terminal"), "none");
        return;
    };
    assert_eq!(report.number("fault_matches"), 1, "{} match count", case.id);
    assert_eq!(
        report.value("fault_role"),
        expected.role.metric_name(),
        "{} role",
        case.id
    );
    assert_eq!(
        report.number("fault_ordinal"),
        expected.ordinal,
        "{} ordinal",
        case.id
    );
    assert_eq!(
        report.value("fault_operation") != "none",
        expected.operation_bound
    );
    assert!(report.number("fault_role_attempts") >= expected.ordinal);
    assert_eq!(report.value("fault_terminal"), expected.terminal);
    let requested = report.number("fault_requested_bytes");
    let completed = report.number("fault_completed_bytes");
    assert!(
        completed <= requested,
        "{} fault prefix overflowed",
        case.id
    );
    match expected.terminal {
        "denied" => assert_eq!(completed, 0, "{} denied after bytes", case.id),
        "partial" => assert_eq!(
            completed,
            match case.id {
                "short-identity-prefix" => 17,
                "qualification-positioned-write" => 31,
                _ => panic!("unclassified partial fault case: {}", case.id),
            },
            "{} completed the wrong prefix",
            case.id
        ),
        "indeterminate" => assert_eq!(completed, requested, "{} lost effect width", case.id),
        other => panic!("unclassified fault terminal {other}"),
    }
}

fn assert_abrupt_localization(boundary: &str, case: FaultCase) {
    let Some(expected) = case.expected_fault else {
        assert_eq!(event_field(boundary, "role"), "none");
        return;
    };
    assert_eq!(event_field(boundary, "role"), expected.role.metric_name());
    assert_eq!(
        event_field(boundary, "ordinal"),
        expected.ordinal.to_string()
    );
    assert_eq!(
        event_field(boundary, "operation") != "none",
        expected.operation_bound
    );
}

fn event_field<'a>(event: &'a str, name: &str) -> &'a str {
    event
        .split(';')
        .find_map(|field| field.strip_prefix(&format!("{name}=")))
        .unwrap()
}

pub(super) fn run_child(role: &str, root: &Path) {
    match role {
        "faulted-media-writer" => faulted_writer(root),
        "fault-reopener" => fault_reopener(root),
        _ => panic!("unsupported fault role"),
    }
}

fn faulted_writer(root: &Path) {
    let case = std::env::var("WORTH_STORE_C4_FAULT_CASE").unwrap();
    let (admission, gate) = fault_schedule::fault_admission(&case);
    if let Some(gate) = gate {
        let observer = gate.clone();
        std::thread::spawn(move || {
            observer.wait_until_reached();
            let context = observer.reached_context();
            event(&format!(
                "fault-boundary;role={};ordinal={};operation={};handle={}",
                context.map_or("none", |value| value.role().metric_name()),
                context.map_or(0, |value| value.role_ordinal()),
                context
                    .and_then(|value| value.operation())
                    .map_or_else(|| "none".into(), |value| value.value().to_string()),
                context
                    .and_then(|value| value.handle())
                    .map_or_else(|| "none".into(), |value| value.generation().to_string()),
            ));
        });
    }
    match admit_runtime(root)
        .try_admit_filesystem_media(admission)
        .into_raw()
    {
        TransitionOutcome::Success(media) => {
            let store = hex(&media.store_identity().bytes());
            let observer = media.observer();
            let shutdown = media.close();
            assert!(matches!(shutdown, MediaShutdownOutcome::Released(_)));
            emit_counter_report("success", Some(store), observer.media_counters());
        }
        TransitionOutcome::Denied(denial) => {
            let counters = *denial.reason().counters();
            denial.into_runtime().abort();
            emit_counter_report("denied", None, counters);
        }
        TransitionOutcome::Failed(failure) => {
            let counters = inspection_counters(failure.cause());
            emit_counter_report("failed", None, counters);
        }
        _ => panic!("fault case produced an unsupported transition category"),
    }
}

fn fault_reopener(root: &Path) {
    let media = match admit_runtime(root)
        .try_admit_filesystem_media(media_admission())
        .into_raw()
    {
        TransitionOutcome::Success(media) => media,
        _ => panic!("fault residue was not safely re-admissible"),
    };
    let observer = media.observer();
    assert!(matches!(media.close(), MediaShutdownOutcome::Released(_)));
    emit(&[
        ("outcome", "success".into()),
        (
            "conserved",
            observer.media_counters().is_conserved().to_string(),
        ),
    ]);
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
