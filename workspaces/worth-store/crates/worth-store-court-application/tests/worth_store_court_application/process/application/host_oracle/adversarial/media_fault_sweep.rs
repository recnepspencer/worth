//! C4b.1: Genuine filesystem media fault sweep (`FailBefore` &
//! `IndeterminateAfterEffect` across every observed `(MediaOperationRole, nth)`)
//! plus typed pre-service refusal on unrecoverable media damage.
use super::super::{opening, *};
use super::oracle_verifier::{
    build_reference, open_dir, prepare_prefix, verify_reopen_and_continue, ReferenceSnapshots,
    TARGET_KEY, TARGET_VAL,
};
use super::{build_id, RestorableHome, BUILD_PRIMARY};
use host_interruption::opening::AdversarialPlacement;
use host_observation::{
    adversarial_verdict::RecoveredPrefix, opening::PlacementOpenConfig, requests,
};
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::WorthQueryApplicationHomeLocation as Location;
use worth_store::physical_runtime::{
    certification::{CertificationMediaFaultActivation, MediaFaultDirective, MediaOperationRole},
    FilesystemAccessPosture, FilesystemMediaAdmission,
};

const MEDIA_CHILD: &str =
    "process::application::host_oracle::adversarial::media_fault_sweep::host_media_fault_child";
const ENV_HOME: &str = "WORTH_ADV_MEDIA_HOME";
const ENV_MARKERS: &str = "WORTH_ADV_MEDIA_MARKERS";
const ENV_SPEC: &str = "WORTH_ADV_MEDIA_SPEC";

pub(super) type MediaCase = (usize, u64, &'static str);

#[test]
fn host_mutation_media_fault_sweep_recovers_to_memory_oracle() {
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let reference = build_reference(&execution);
    let saved = RestorableHome::prepare(|loc| prepare_prefix(loc, &execution));
    let counts = measure_op_media_roles(saved.location(), &execution, |home, _| {
        assert!(requests::set(home.runtime(), TARGET_KEY, TARGET_VAL)
            .receipt()
            .is_some());
    });
    assert!(
        !counts.is_empty(),
        "Host mutation must perform identified media ops"
    );

    let cases = enumerate_admissible_cases(&counts);
    assert!(!cases.is_empty());
    let verdicts = run_parallel_media_cases(saved, &cases, &reference);
    assert_eq!(verdicts.len(), cases.len());
}

pub(super) fn enumerate_admissible_cases(counts: &[(usize, u64)]) -> Vec<MediaCase> {
    let mut cases = Vec::new();
    for &(role_idx, count) in counts {
        let role = MediaOperationRole::ALL[role_idx];
        for nth in 1..=count {
            for dir_tag in ["fail_before", "indeterminate"] {
                if media_rule_admissible(role, decode_directive(dir_tag)) {
                    cases.push((role_idx, nth, dir_tag));
                }
            }
        }
    }
    cases
}

fn run_parallel_media_cases(
    primary_home: RestorableHome,
    cases: &[MediaCase],
    reference: &ReferenceSnapshots,
) -> Vec<RecoveredPrefix> {
    let workers = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(4)
        .clamp(1, 8)
        .min(cases.len().max(1));
    let mut homes = Vec::with_capacity(workers);
    homes.push(primary_home);
    if workers > 1 {
        let extra: Vec<_> = std::thread::scope(|scope| {
            let handles: Vec<_> = (1..workers)
                .map(|_| {
                    scope.spawn(|| {
                        let req = opening::serial();
                        let exec = ExecutionRequest::serial(&req);
                        RestorableHome::prepare(|loc| prepare_prefix(loc, &exec))
                    })
                })
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });
        homes.extend(extra);
    }
    super::parallel::run_parallel_worker_cases("media", &homes, cases, |home, &case| {
        let req = opening::serial();
        let exec = ExecutionRequest::serial(&req);
        run_one_media_case(home, case, reference, &exec)
    })
}

fn run_one_media_case(
    home: &RestorableHome,
    case: MediaCase,
    reference: &ReferenceSnapshots,
    execution: &ExecutionRequest<'_, '_>,
) -> RecoveredPrefix {
    let (role_idx, nth, dir_tag) = case;
    home.restore();
    let markers = tempfile::tempdir().unwrap();
    let ready = markers.path().join("ready");
    let finished = markers.path().join("finished");
    let spec_str = format!("{role_idx}:{nth}:{dir_tag}");
    let spec = PathBuf::from(&spec_str);
    let end = crate::killed_child::run_and_kill(
        MEDIA_CHILD,
        &[
            (ENV_HOME, home.location()),
            (ENV_MARKERS, markers.path()),
            (ENV_SPEC, &spec),
        ],
        &ready,
        &finished,
    );
    assert_eq!(end, crate::killed_child::ChildEnd::KilledAtPark);
    let marker = std::fs::read_to_string(&ready).unwrap_or_default();
    assert_eq!(
        marker, spec_str,
        "media ready marker must match active spec"
    );
    let label = format!("{:?}#{nth}:{dir_tag}", MediaOperationRole::ALL[role_idx]);
    verify_reopen_and_continue(home.location(), reference, execution, &label)
}

pub(super) fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_owned()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "non-string panic".into()
    }
}

pub(super) fn measure_op_media_roles(
    location: &Path,
    execution: &ExecutionRequest<'_, '_>,
    run_op: impl FnOnce(&host_observation::OracleHome, &ExecutionRequest<'_, '_>),
) -> Vec<(usize, u64)> {
    let media =
        FilesystemMediaAdmission::certification(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = media.fault_schedule_authority();
    let activations: Vec<_> = MediaOperationRole::ALL
        .iter()
        .map(|_| authority.one_shot_activation())
        .collect();
    let rules = MediaOperationRole::ALL
        .iter()
        .zip(&activations)
        .map(|(role, act)| {
            authority
                .rule(*role, 1, decode_directive("fail_before"))
                .for_nth_identified_operation_after_activation(act.clone(), NonZeroU64::MAX)
        })
        .collect();
    let schedule = authority.schedule(rules).unwrap();
    let placement = AdversarialPlacement::new().with_media(media.with_fault_schedule(schedule));
    let (home, _) = open_dir(
        location,
        Some(placement),
        Arc::new(AtomicUsize::new(0)),
        execution,
    );
    for act in &activations {
        act.arm().unwrap();
    }
    run_op(&home, execution);
    let counts: Vec<_> = activations
        .iter()
        .enumerate()
        .map(|(idx, act)| (idx, act.matching_operation_count()))
        .filter(|(_, c)| *c > 0)
        .collect();
    home.close(execution).unwrap().reclaim();
    counts
}

pub(super) fn open_with_one_shot_fault(
    location: &Path,
    role_idx: usize,
    nth: u64,
    dir_tag: &str,
    execution: &ExecutionRequest<'_, '_>,
) -> (
    host_observation::OracleHome,
    CertificationMediaFaultActivation,
) {
    let role = MediaOperationRole::ALL[role_idx];
    let media =
        FilesystemMediaAdmission::certification(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = media.fault_schedule_authority();
    let activation = authority.one_shot_activation();
    let rule = authority
        .rule(role, 1, decode_directive(dir_tag))
        .for_nth_identified_operation_after_activation(
            activation.clone(),
            NonZeroU64::new(nth).unwrap(),
        );
    let schedule = authority.schedule(vec![rule]).unwrap();
    let (home, _) = open_dir(
        location,
        Some(AdversarialPlacement::new().with_media(media.with_fault_schedule(schedule))),
        Arc::new(AtomicUsize::new(0)),
        execution,
    );
    (home, activation)
}

pub(super) fn decode_directive(tag: &str) -> MediaFaultDirective {
    match tag {
        "fail_before" => MediaFaultDirective::FailBefore {
            kind: std::io::ErrorKind::Other,
            raw_os_error: None,
        },
        "indeterminate" => MediaFaultDirective::IndeterminateAfterEffect,
        other => panic!("unknown directive {other}"),
    }
}

pub(super) fn media_rule_admissible(
    role: MediaOperationRole,
    directive: MediaFaultDirective,
) -> bool {
    let media =
        FilesystemMediaAdmission::certification(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = media.fault_schedule_authority();
    let act = authority.one_shot_activation();
    let rule = authority
        .rule(role, 1, directive)
        .for_nth_identified_operation_after_activation(act, NonZeroU64::MIN);
    authority.schedule(vec![rule]).is_ok()
}

#[test]
fn host_media_fault_child() {
    let Some(location) = std::env::var_os(ENV_HOME).map(PathBuf::from) else {
        return;
    };
    let markers = PathBuf::from(std::env::var_os(ENV_MARKERS).unwrap());
    let spec = std::env::var(ENV_SPEC).unwrap();
    let mut parts = spec.split(':');
    let role_idx: usize = parts.next().unwrap().parse().unwrap();
    let nth: u64 = parts.next().unwrap().parse().unwrap();
    let dir_tag = parts.next().unwrap();
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let (home, activation) =
        open_with_one_shot_fault(&location, role_idx, nth, dir_tag, &execution);
    activation.arm().unwrap();
    let _outcome = requests::set(home.runtime(), TARGET_KEY, TARGET_VAL);
    assert!(activation.is_consumed(), "media fault {spec} did not fire");
    crate::killed_child::announce(&markers.join("ready"), spec.as_bytes());
    crate::killed_child::park();
}

#[test]
fn unrecoverable_media_damage_refuses_host_open_before_service() {
    use super::oracle_verifier::PREFIX_VAL;
    use worth_store_application::ApplicationHomeOpenRefusal as StoreRefusal;

    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);

    // Prepare one sealed fixture and restore it cleanly for each control/damage case.
    let saved = RestorableHome::prepare(|loc| prepare_prefix(loc, &execution));
    let (healthy, _) = open_dir(
        saved.location(),
        None,
        Arc::new(AtomicUsize::new(0)),
        &execution,
    );
    assert_eq!(requests::balance(healthy.runtime()), PREFIX_VAL);
    healthy.close(&execution).unwrap().reclaim();

    let damage_cases: [(&[u8], usize, &str, fn(&StoreRefusal) -> bool); 3] = [
        (b"WSRHOME1", 16, "descriptor WSRHOME1", |c| {
            matches!(c, StoreRefusal::Descriptor(_))
        }),
        (b"WAPHCKP1", 16, "checkpoint WAPHCKP1", |c| {
            matches!(c, StoreRefusal::Checkpoint(_))
        }),
        (
            crate::open_refusals::ARTIFACT_MAGIC,
            64,
            "artifact WRTHRCCA",
            |c| matches!(c, StoreRefusal::Relational(_)),
        ),
    ];
    for (magic, offset, label, matcher) in damage_cases {
        let (refused, contacts) = corrupt_and_expect_refusal(&saved, magic, offset, &execution);
        assert_eq!(contacts, 0, "refused {label} open never runs handlers");
        assert_frame_damage_refusal(&refused, label, matcher);
    }
}

type OpenRefused = worth_query_host::facade::WorthQueryApplicationHomeOpenRefused<
    worth_query_testkit_host::facade::host::application_home::schema::HomeIdentitySchema,
    worth_query_testkit_host::facade::host::application_home::HomeProgram,
>;

fn assert_frame_damage_refusal(
    refused: &OpenRefused,
    label: &str,
    frame_match: impl FnOnce(&worth_store_application::ApplicationHomeOpenRefusal) -> bool,
) {
    use worth_query_host::facade::WorthQueryApplicationHomeOpenRefusedPhase as RefusedPhase;
    use worth_store_application::{
        ApplicationHomeOpenRefusal as StoreRefusal, ApplicationHomePhysicalRefusalKind as Kind,
        ApplicationHomePhysicalStage as Stage,
    };
    let ok = match refused.phase() {
        RefusedPhase::Physical { cause, .. } if frame_match(cause) => true,
        RefusedPhase::Physical {
            cause: StoreRefusal::Physical(p),
            ..
        } => {
            p.stage() == Stage::Recovery
                && p.kind() == Kind::PhysicalStep
                && p.recovery_effects() == Some(0)
                && p.detail().contains("ChecksumMismatch")
                && p.detail().contains("Damage(")
        }
        _ => false,
    };
    assert!(
        ok,
        "damaged {label} refused with unexpected cause: {refused:?}"
    );
}

fn corrupt_and_expect_refusal(
    saved: &RestorableHome,
    magic: &[u8],
    offset: usize,
    execution: &ExecutionRequest<'_, '_>,
) -> (OpenRefused, usize) {
    saved.restore();
    let home = saved.location().to_path_buf();
    let flipped = crate::open_refusals::flip_after(&home, magic, offset, &|_| true);
    assert!(
        flipped > 0,
        "closed Home must persist {:?} frame",
        String::from_utf8_lossy(magic)
    );
    let contacts = Arc::new(AtomicUsize::new(0));
    let outcome = host_observation::opening::try_open_with_placement(
        PlacementOpenConfig {
            location: Location::Directory(home),
            build: build_id(BUILD_PRIMARY),
            initialized: Arc::new(AtomicUsize::new(0)),
            contacts: contacts.clone(),
            placement: None,
            limits: Default::default(),
        },
        execution,
    );
    let refused = match outcome {
        Ok(_) => panic!(
            "damaged {:?} frame must refuse open before service",
            String::from_utf8_lossy(magic)
        ),
        Err(refused) => refused,
    };
    (refused, contacts.load(Ordering::SeqCst))
}
