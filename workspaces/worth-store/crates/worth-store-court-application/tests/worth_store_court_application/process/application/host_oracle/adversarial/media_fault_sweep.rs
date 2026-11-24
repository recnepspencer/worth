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
    Arc, Mutex,
};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::WorthQueryApplicationHomeLocation as Location;
use worth_store::physical_runtime::{
    certification::{MediaFaultDirective, MediaOperationRole},
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
    let counts = measure_media_roles(saved.location(), &execution);
    assert!(!counts.is_empty(), "Host mutation must perform identified media ops");

    let cases = enumerate_admissible_cases(&counts);
    assert!(!cases.is_empty());
    let verdicts = run_parallel_media_cases(&cases, &reference);
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
    cases: &[MediaCase],
    reference: &ReferenceSnapshots,
) -> Vec<RecoveredPrefix> {
    let workers = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(4)
        .clamp(1, 4)
        .min(cases.len().max(1));
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<Option<Result<RecoveredPrefix, String>>>> =
        Mutex::new((0..cases.len()).map(|_| None).collect());

    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                let req = opening::serial();
                let exec = ExecutionRequest::serial(&req);
                let home = RestorableHome::prepare(|loc| prepare_prefix(loc, &exec));
                let markers = tempfile::tempdir().unwrap();
                loop {
                    let idx = next.fetch_add(1, Ordering::Relaxed);
                    let Some(&(role_idx, nth, dir_tag)) = cases.get(idx) else {
                        break;
                    };
                    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        run_one_media_case((&home, markers.path()), (role_idx, nth, dir_tag), reference, &exec)
                    }))
                    .map_err(|payload| panic_message(&*payload));
                    results.lock().unwrap()[idx] = Some(outcome);
                }
            });
        }
    });

    let mut verdicts = Vec::with_capacity(cases.len());
    for (idx, slot) in results.into_inner().unwrap().into_iter().enumerate() {
        match slot.expect("every media case ran") {
            Ok(v) => verdicts.push(v),
            Err(msg) => panic!("media case #{idx} ({:?}) failed: {msg}", cases[idx]),
        }
    }
    verdicts
}

fn run_one_media_case(
    paths: (&RestorableHome, &Path),
    case: MediaCase,
    reference: &ReferenceSnapshots,
    execution: &ExecutionRequest<'_, '_>,
) -> RecoveredPrefix {
    let (home, markers_root) = paths;
    let (role_idx, nth, dir_tag) = case;
    home.restore();
    let ready = markers_root.join("ready");
    let finished = markers_root.join("finished");
    let _ = std::fs::remove_file(&ready);
    let _ = std::fs::remove_file(&finished);
    let spec = PathBuf::from(format!("{role_idx}:{nth}:{dir_tag}"));
    let end = crate::killed_child::run_and_kill(
        MEDIA_CHILD,
        &[
            (ENV_HOME, home.location()),
            (ENV_MARKERS, markers_root),
            (ENV_SPEC, &spec),
        ],
        &ready,
        &finished,
    );
    assert_eq!(end, crate::killed_child::ChildEnd::KilledAtPark);
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

fn measure_media_roles(location: &Path, execution: &ExecutionRequest<'_, '_>) -> Vec<(usize, u64)> {
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
    let (home, _) = open_dir(location, Some(placement), Arc::new(AtomicUsize::new(0)), execution);
    for act in &activations {
        act.arm().unwrap();
    }
    assert!(requests::set(home.runtime(), TARGET_KEY, TARGET_VAL)
        .receipt()
        .is_some());
    let counts: Vec<_> = activations
        .iter()
        .enumerate()
        .map(|(idx, act)| (idx, act.matching_operation_count()))
        .filter(|(_, c)| *c > 0)
        .collect();
    home.close(execution).unwrap().reclaim();
    counts
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

pub(super) fn media_rule_admissible(role: MediaOperationRole, directive: MediaFaultDirective) -> bool {
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
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let (home, _) = open_dir(
        &location,
        Some(AdversarialPlacement::new().with_media(media.with_fault_schedule(schedule))),
        Arc::new(AtomicUsize::new(0)),
        &execution,
    );
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

    // Control: undamaged prepared fixture opens cleanly and serves PREFIX_VAL.
    let healthy_dir = tempfile::tempdir().unwrap();
    let healthy_home = healthy_dir.path().join("home");
    prepare_prefix(&healthy_home, &execution);
    let (healthy, _) = open_dir(&healthy_home, None, Arc::new(AtomicUsize::new(0)), &execution);
    assert_eq!(requests::balance(healthy.runtime()), PREFIX_VAL);
    healthy.close(&execution).unwrap().reclaim();

    // Case 1: unrecoverable damage inside the Home descriptor frame (WSRHOME1).
    let (desc_refused, desc_contacts) = corrupt_and_expect_refusal(b"WSRHOME1", 16, &execution);
    assert_eq!(desc_contacts, 0, "refused descriptor open never runs handlers");
    assert_frame_damage_refusal(&desc_refused, "descriptor WSRHOME1", |cause| {
        matches!(cause, StoreRefusal::Descriptor(_))
    });

    // Case 2: unrecoverable damage inside the Application checkpoint manifest (WAPHCKP1).
    let (ckpt_refused, ckpt_contacts) = corrupt_and_expect_refusal(b"WAPHCKP1", 16, &execution);
    assert_eq!(ckpt_contacts, 0, "refused checkpoint open never runs handlers");
    assert_frame_damage_refusal(&ckpt_refused, "checkpoint WAPHCKP1", |cause| {
        matches!(cause, StoreRefusal::Checkpoint(_))
    });

    // Case 3: unrecoverable damage inside an authoritative relational journal artifact (WRTHRCCA).
    let (art_refused, art_contacts) =
        corrupt_and_expect_refusal(crate::open_refusals::ARTIFACT_MAGIC, 64, &execution);
    assert_eq!(art_contacts, 0, "refused journal open never runs handlers");
    assert_frame_damage_refusal(&art_refused, "artifact WRTHRCCA", |cause| {
        matches!(cause, StoreRefusal::Relational(_))
    });
}

fn assert_frame_damage_refusal(
    refused: &worth_query_host::facade::WorthQueryApplicationHomeOpenRefused<
        worth_query_testkit_host::facade::host::application_home::schema::HomeIdentitySchema,
        worth_query_testkit_host::facade::host::application_home::HomeProgram,
    >,
    label: &str,
    frame_match: impl FnOnce(&worth_store_application::ApplicationHomeOpenRefusal) -> bool,
) {
    use worth_query_host::facade::WorthQueryApplicationHomeOpenRefused as OpenRefused;
    use worth_store_application::{
        ApplicationHomeOpenRefusal as StoreRefusal, ApplicationHomePhysicalRefusalKind as Kind,
        ApplicationHomePhysicalStage as Stage,
    };
    let ok = match refused {
        OpenRefused::Physical { cause, .. } if frame_match(cause) => true,
        OpenRefused::Physical { cause: StoreRefusal::Physical(p), .. } => {
            p.stage() == Stage::Recovery
                && p.kind() == Kind::PhysicalStep
                && p.recovery_effects() == Some(0)
                && p.detail().contains("ChecksumMismatch")
                && p.detail().contains("Damage(")
        }
        _ => false,
    };
    assert!(ok, "damaged {label} refused with unexpected cause: {refused:?}");
}

fn corrupt_and_expect_refusal(
    magic: &[u8],
    offset: usize,
    execution: &ExecutionRequest<'_, '_>,
) -> (
    worth_query_host::facade::WorthQueryApplicationHomeOpenRefused<
        worth_query_testkit_host::facade::host::application_home::schema::HomeIdentitySchema,
        worth_query_testkit_host::facade::host::application_home::HomeProgram,
    >,
    usize,
) {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    prepare_prefix(&home, execution);
    let flipped = crate::open_refusals::flip_after(&home, magic, offset, &|_| true);
    assert!(flipped > 0, "closed Home must persist {:?} frame", String::from_utf8_lossy(magic));
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
        Ok(_) => panic!("damaged {:?} frame must refuse open before service", String::from_utf8_lossy(magic)),
        Err(refused) => refused,
    };
    (refused, contacts.load(Ordering::SeqCst))
}
