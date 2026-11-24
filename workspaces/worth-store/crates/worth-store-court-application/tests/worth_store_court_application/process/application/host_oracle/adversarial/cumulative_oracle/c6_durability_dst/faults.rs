//! Certification media fault schedules, custody settlement, and durable branch
//! creation helpers for the C6 Store Durability DST.
use super::super::super::build_id;
use super::observe::observe_root_branch;
use super::tla_oracle::BuildTag;
use std::num::{NonZeroU64, NonZeroUsize};
use std::path::Path;
use std::time::{Duration, Instant};
use worth_execution::ExecutionRequest;
use worth_relational::facade::history::BranchId;
use worth_runtime_world::facade::*;
use worth_store::physical_runtime::{
    certification::{CertificationMediaFaultActivation, MediaFaultDirective, MediaOperationRole},
    FilesystemAccessPosture, FilesystemMediaAdmission,
};
use worth_store_application::{ApplicationHomeOpenRefusal, ApplicationHomePhysicalRefusalKind};
use worth_store_testkit_application_oracle::{home_opening, world_home};

pub(super) struct DstFaultHandles {
    pub fail_before_write: CertificationMediaFaultActivation,
    pub indeterminate_sync: CertificationMediaFaultActivation,
    pub fail_before_sync: CertificationMediaFaultActivation,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct CustodySettlementReport {
    pub saw_fork_busy: bool,
}

fn fail_before() -> MediaFaultDirective {
    MediaFaultDirective::FailBefore {
        kind: std::io::ErrorKind::Other,
        raw_os_error: None,
    }
}

fn is_transient_contended(err: &world_home::WorldHomeFixtureRefusal) -> bool {
    matches!(
        err,
        world_home::WorldHomeFixtureRefusal::Home(box_err)
            if matches!(
                box_err.as_ref(),
                ApplicationHomeOpenRefusal::Physical(p)
                    if p.kind() == ApplicationHomePhysicalRefusalKind::MutationOwnerContended
            )
    )
}

pub(super) fn open_world_home_with_faults(
    path: &Path,
    build: BuildTag,
    sync_nth: u64,
    exec: &ExecutionRequest<'_, '_>,
) -> (world_home::WorldHomeFixture, DstFaultHandles) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let media = FilesystemMediaAdmission::certification(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        );
        let auth = media.fault_schedule_authority();
        let fail_before_write = auth.one_shot_activation();
        let indeterminate_sync = auth.one_shot_activation();
        let fail_before_sync = auth.one_shot_activation();

        let rules = vec![
            auth.rule(MediaOperationRole::PositionedWrite, 1, fail_before())
                .for_nth_identified_operation_after_activation(
                    fail_before_write.clone(),
                    NonZeroU64::new(1).unwrap(),
                ),
            auth.rule(
                MediaOperationRole::SynchronizeFileState,
                1,
                MediaFaultDirective::IndeterminateAfterEffect,
            )
            .for_nth_identified_operation_after_activation(
                indeterminate_sync.clone(),
                NonZeroU64::new(sync_nth.clamp(1, 2)).unwrap(),
            ),
            auth.rule(MediaOperationRole::SynchronizeFileState, 1, fail_before())
                .for_nth_identified_operation_after_activation(
                    fail_before_sync.clone(),
                    NonZeroU64::new(1).unwrap(),
                ),
        ];
        let schedule = auth.schedule(rules).unwrap();
        let req = home_opening::with_media(
            home_opening::request_with_build(
                path,
                crate::workload::runtime_builder(crate::schema_transition::SOURCE),
                build_id(build.as_uuid()),
            ),
            media.with_fault_schedule(schedule),
        );
        match world_home::open_with_retained_history_limit(req, exec, 64) {
            Ok(fixture) => {
                return (
                    fixture,
                    DstFaultHandles {
                        fail_before_write,
                        indeterminate_sync,
                        fail_before_sync,
                    },
                );
            }
            Err(err) if is_transient_contended(&err) && Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(2));
            }
            Err(err) => panic!("clean world home open must succeed: {err:?}"),
        }
    }
}

/// Attempt a real recovery reopen with an ARMED media fault to simulate a
/// chained crash mid-recovery (`BeginRecover -> CrashKill`), returning both
/// the typed open outcome and whether the injected fault actually fired.
pub(super) fn try_open_with_recovery_fault(
    path: &Path,
    build: BuildTag,
    role: MediaOperationRole,
    nth: u64,
    exec: &ExecutionRequest<'_, '_>,
) -> (
    Result<world_home::WorldHomeFixture, world_home::WorldHomeFixtureRefusal>,
    bool,
) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let media = FilesystemMediaAdmission::certification(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        );
        let auth = media.fault_schedule_authority();
        let activation = auth.one_shot_activation();
        let rule = auth
            .rule(role, 1, fail_before())
            .for_nth_identified_operation_after_activation(
                activation.clone(),
                NonZeroU64::new(nth.max(1)).unwrap(),
            );
        let schedule = auth.schedule(vec![rule]).unwrap();
        activation.arm().unwrap();
        let req = home_opening::with_media(
            home_opening::request_with_build(
                path,
                crate::workload::runtime_builder(crate::schema_transition::SOURCE),
                build_id(build.as_uuid()),
            ),
            media.with_fault_schedule(schedule),
        );
        match world_home::open_with_retained_history_limit(req, exec, 64) {
            Err(err)
                if !activation.is_consumed()
                    && is_transient_contended(&err)
                    && Instant::now() < deadline =>
            {
                std::thread::sleep(Duration::from_millis(2));
            }
            outcome => return (outcome, activation.is_consumed()),
        }
    }
}

pub(super) fn pending_custody_handles(
    fixture: &world_home::WorldHomeFixture,
    ctx: &str,
) -> Vec<ProductUnpublishedRecoveryHandle> {
    let page = fixture
        .home
        .world()
        .inspection_port()
        .recovery_page(None, NonZeroUsize::new(16).unwrap())
        .unwrap_or_else(|e| panic!("{ctx}: recovery_page failed: {e:?}"));
    page.rows().iter().map(|r| r.handle().clone()).collect()
}

pub(super) fn settle_pending_custody(
    fixture: &world_home::WorldHomeFixture,
    exec: &ExecutionRequest<'_, '_>,
    ctx: &str,
) -> CustodySettlementReport {
    let handles = pending_custody_handles(fixture, ctx);
    let mut report = CustodySettlementReport::default();
    let cancel = RuntimeWorldCancellationSource::new();
    for initial_handle in handles {
        let recovery = fixture.home.world().recovery_port();
        if let Ok(mut claim) = recovery.inspect_recovered_effects(&initial_handle) {
            loop {
                match recovery.continue_recovered_effects(exec, claim) {
                    WorldRecoveredContinuationOutcome::Pending(next) => {
                        std::thread::sleep(Duration::from_millis(1));
                        claim = next;
                    }
                    WorldRecoveredContinuationOutcome::Published { .. } => break,
                    WorldRecoveredContinuationOutcome::Refused {
                        cause: WorldRecoveredContinuationDenial::NoOwnerEffect,
                        custody,
                    } => {
                        let mut rel = recovery.release_recovered_effects(exec, custody);
                        loop {
                            match rel {
                                WorldRecoveredReleaseOutcome::Pending(next) => {
                                    std::thread::sleep(Duration::from_millis(1));
                                    rel = recovery.release_recovered_effects(exec, next);
                                }
                                WorldRecoveredReleaseOutcome::Released(_) => break,
                                other => panic!(
                                    "{ctx}: release_recovered_effects unexpected: {}",
                                    std::any::type_name_of_val(&other)
                                ),
                            }
                        }
                        break;
                    }
                    WorldRecoveredContinuationOutcome::Refused { cause, .. } => {
                        panic!("{ctx}: continue_recovered_effects refused: {cause:?}");
                    }
                    other => panic!(
                        "{ctx}: continue_recovered_effects unexpected: {}",
                        std::any::type_name_of_val(&other)
                    ),
                }
            }
        } else {
            let branch_port = fixture.home.world().branch_port();
            let mut handle = initial_handle;
            let deadline = Instant::now() + Duration::from_secs(30);
            loop {
                assert!(
                    Instant::now() < deadline,
                    "{ctx}: recovered resume_creation livelocked in Busy/Pending past 30s"
                );
                match branch_port.resume_creation(exec, &handle, &cancel.token()) {
                    Ok(RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects)) => {
                        handle = effects.recovery_handle();
                        drop(effects);
                        std::thread::sleep(Duration::from_millis(1));
                    }
                    Ok(_) => break,
                    Err(RuntimeWorldRecoveryDenial::Busy) => {
                        report.saw_fork_busy = true;
                        std::thread::sleep(Duration::from_millis(1));
                    }
                    Err(e) => panic!("{ctx}: resume_creation failed: {e:?}"),
                }
            }
        }
    }
    report
}

pub(super) fn try_create_branch(
    fixture: &world_home::WorldHomeFixture,
    exec: &ExecutionRequest<'_, '_>,
    name: &str,
    fork_prefix: &str,
    native_fork: bool,
    signal_fork: bool,
    ctx: &str,
) -> Result<(ProductBranchObservation, bool), String> {
    let source = observe_root_branch(fixture, ctx);
    let intent = ProductBranchCreationIntent::from_source(
        name,
        ProductBranchCreationPlans::new(
            if native_fork {
                RelationalBranchCreationPlan::ForkExact {
                    target: BranchId(format!("{fork_prefix}-native")),
                }
            } else {
                RelationalBranchCreationPlan::ReuseExact
            },
            if signal_fork {
                SignalBranchCreationPlan::fork_exact_named(format!("{fork_prefix}-signal")).unwrap()
            } else {
                SignalBranchCreationPlan::ReuseExact
            },
        ),
    )
    .map_err(|e| format!("intent: {e:?}"))?;
    let port = fixture.home.world().branch_port();
    let cancel = RuntimeWorldCancellationSource::new();
    let mut out = port
        .create_home_product_branch(
            exec,
            RuntimeWorldHomeBranchCreationRequest::new(
                source.clone(),
                fixture.admit_initial_program_source(&source),
                intent,
            ),
            &cancel.token(),
        )
        .map_err(|e| format!("create_home_product_branch: {e:?}"))?;
    let mut saw_busy = false;
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if Instant::now() >= deadline {
            return Err("branch creation timed out".into());
        }
        match out {
            RuntimeWorldBranchCreationOutcome::Performed(obs) => return Ok((obs, saw_busy)),
            RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects) => {
                let handle = effects.recovery_handle();
                drop(effects);
                std::thread::sleep(Duration::from_millis(1));
                loop {
                    match port.resume_creation(exec, &handle, &cancel.token()) {
                        Ok(next) => {
                            out = next;
                            break;
                        }
                        Err(RuntimeWorldRecoveryDenial::Busy) if Instant::now() < deadline => {
                            saw_busy = true;
                            std::thread::sleep(Duration::from_millis(1));
                        }
                        Err(e) => return Err(format!("resume_creation: {e:?}")),
                    }
                }
            }
            other => return Err(format!("non-performed branch creation: {other:?}")),
        }
    }
}

pub(super) fn assert_physical_io_open_refusal(
    err: &world_home::WorldHomeFixtureRefusal,
    ctx: &str,
) {
    match err {
        world_home::WorldHomeFixtureRefusal::Home(box_err) => match box_err.as_ref() {
            ApplicationHomeOpenRefusal::Physical(p)
                if p.kind() == ApplicationHomePhysicalRefusalKind::PhysicalStep => {}
            other => panic!("{ctx}: recovery fault produced unexpected open refusal: {other:?}"),
        },
        other => panic!("{ctx}: recovery fault produced unexpected fixture refusal: {other:?}"),
    }
}

pub(super) fn drive_open_outcome(
    mut outcome: WorldOpenAttemptOutcome,
    exec: &ExecutionRequest<'_, '_>,
    ctx: &str,
) -> WorldOpenAttemptOutcome {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        match outcome {
            WorldOpenAttemptOutcome::Pending(pending) => {
                std::thread::sleep(Duration::from_millis(1));
                outcome = pending.advance(exec);
            }
            done => return done,
        }
    }
    panic!("{ctx}: open_publication_without_signal timed out in Pending")
}

pub(super) fn drive_publish_outcome(
    mut outcome: WorldDurablePublicationOutcome,
    exec: &ExecutionRequest<'_, '_>,
    ctx: &str,
) -> WorldDurablePublicationOutcome {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        match outcome {
            WorldDurablePublicationOutcome::Pending(pending) => {
                std::thread::sleep(Duration::from_millis(1));
                outcome = pending.advance(exec);
            }
            done => return done,
        }
    }
    panic!("{ctx}: execute_durable_without_signal timed out in Pending")
}
