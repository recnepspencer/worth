//! Child process entrypoints for C4b.4 C2.8 Durable World Branch Creation,
//! Retirement, History Disposal, and Barrier-Kill lane.
use super::super::super::super::graceful::signal::derived_discard::checkpoint_and_discard_world_home;
use super::super::super::{barrier_marker, build_id};
use super::ops::{
    continue_wb_pending_custody, create_branch, find_branch, observe_wb_opt, open_wb, publish_on,
};
use super::{WbCmd, WbReply, ENV_BARRIER_SPEC, ENV_BUILD, ENV_CH, ENV_HOME};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::*;
use worth_store::physical_runtime::{
    certification::{MediaFaultDirective, MediaOperationRole},
    FilesystemAccessPosture, FilesystemMediaAdmission,
};
use worth_store_application::{ApplicationHomeCheckpointDenial, ApplicationHomeCloseOutcome};
use worth_store_testkit_application_oracle::{home_opening, world_home};

#[test]
fn c4b4_world_branch_child() {
    let Some(home_path) = std::env::var_os(ENV_HOME).map(PathBuf::from) else {
        return;
    };
    let channel = PathBuf::from(std::env::var_os(ENV_CH).unwrap());
    let build = std::env::var(ENV_BUILD).unwrap();
    let req = crate::workload::serial_request();
    let exec = ExecutionRequest::serial(&req);
    let mut fixture = Some(open_wb(&home_path, &build, &exec));
    let send = |f: Option<&world_home::WorldHomeFixture>,
                retired_commit,
                reclaimed_ordinals,
                custody_continued,
                discard_gen,
                close_gen,
                closed| {
        let reply = WbReply {
            snap: f.and_then(|fx| observe_wb_opt(fx, &exec)),
            retired_commit,
            reclaimed_ordinals,
            custody_continued,
            discard_gen,
            close_gen,
            closed,
        };
        crate::killed_child::announce(
            &channel.join("response"),
            &serde_json::to_vec(&reply).unwrap(),
        );
    };
    send(fixture.as_ref(), None, Vec::new(), None, None, None, false);
    loop {
        let req_path = channel.join("request");
        if !req_path.is_file() {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        }
        let cmd: WbCmd = serde_json::from_slice(&std::fs::read(&req_path).unwrap()).unwrap();
        std::fs::remove_file(req_path).unwrap();
        let (mut ret_ord, mut reclaimed, mut custody_cont, mut dgen) =
            (None, Vec::new(), None, None);
        match cmd {
            WbCmd::BootstrapRoot => {
                drop(fixture.as_ref().unwrap().bootstrap_root(&exec));
            }
            WbCmd::CreateBranch {
                parent,
                name,
                fork_prefix,
                native_fork,
                signal_fork,
            } => {
                create_branch(
                    fixture.as_ref().unwrap(),
                    &exec,
                    &parent,
                    &name,
                    &fork_prefix,
                    native_fork,
                    signal_fork,
                );
            }
            WbCmd::Publish { branch, round } => {
                publish_on(fixture.as_ref().unwrap(), &exec, &branch, round);
            }
            WbCmd::ContinuePendingCustody => {
                custody_cont = Some(continue_wb_pending_custody(
                    fixture.as_ref().unwrap(),
                    &exec,
                ));
            }
            WbCmd::RetireBranch { name } => {
                let fx = fixture.as_ref().unwrap();
                let target = find_branch(fx, &exec, &name);
                let bid = target.branch_identity().clone();
                let report = fx
                    .home
                    .world()
                    .branch_port()
                    .retire_product_branch(&target)
                    .unwrap();
                ret_ord = Some(report.retired_head().ordinal());
                assert!(matches!(
                    fx.home.world().branch_port().retire_product_branch(&target),
                    Err(RuntimeWorldServiceDenial::Denied(
                        RuntimeWorldBranchRetirementDenial::AlreadyRetired
                    ))
                ));
                assert!(matches!(
                    fx.home
                        .world()
                        .observation_port()
                        .observe_product_branch(&bid),
                    Err(RuntimeWorldServiceDenial::Denied(
                        RuntimeWorldBranchAdmissionDenial::RetiredBranch
                    ))
                ));
            }
            WbCmd::DisposeRetiredHistory { retired_commit } => {
                let fx = fixture.as_ref().unwrap();
                let owner = fx.home.world().owner_identity().durable().unwrap();
                let durable_id =
                    worth_foundational::facade::DurableCompositeCommitIdentity::from_recorded(
                        owner,
                        retired_commit,
                    );
                let head = fx
                    .home
                    .world()
                    .observation_port()
                    .readmitted_publication(&durable_id)
                    .unwrap()
                    .after()
                    .selected_commit()
                    .clone();
                let r1 = fx
                    .home
                    .world()
                    .recovery_port()
                    .release_retired_branch_history(&head)
                    .unwrap();
                reclaimed = r1.reclaimed_commits().iter().map(|c| c.ordinal()).collect();
                let r2 = fx
                    .home
                    .world()
                    .recovery_port()
                    .release_retired_branch_history(&head)
                    .unwrap();
                assert!(r2.reclaimed_commits().is_empty(), "zero double disposal");
            }
            WbCmd::RetireChangedHistory => {
                let fx = fixture.as_ref().unwrap();
                let r1 = fx
                    .home
                    .world()
                    .recovery_port()
                    .retire_changed_history()
                    .unwrap();
                reclaimed = r1.iter().map(|c| c.ordinal()).collect();
                let r2 = fx
                    .home
                    .world()
                    .recovery_port()
                    .retire_changed_history()
                    .unwrap();
                assert!(r2.is_empty(), "zero double changed-history disposal");
            }
            WbCmd::AssertDisposed { ordinals } => {
                let fx = fixture.as_ref().unwrap();
                let owner = fx.home.world().owner_identity().durable().unwrap();
                for ord in ordinals {
                    let id =
                        worth_foundational::facade::DurableCompositeCommitIdentity::from_recorded(
                            owner, ord,
                        );
                    assert!(
                        matches!(
                            fx.home
                                .world()
                                .observation_port()
                                .readmitted_publication(&id),
                            Err(RuntimeWorldServiceDenial::Denied(
                                WorldReadmittedPublicationDenial::NotReadmitted
                            ))
                        ),
                        "disposed commit #{ord} must not resurrect"
                    );
                }
            }
            WbCmd::AssertReadable { ordinals } => {
                let fx = fixture.as_ref().unwrap();
                let owner = fx.home.world().owner_identity().durable().unwrap();
                for ord in ordinals {
                    let id =
                        worth_foundational::facade::DurableCompositeCommitIdentity::from_recorded(
                            owner, ord,
                        );
                    let r = fx
                        .home
                        .world()
                        .observation_port()
                        .readmitted_publication(&id)
                        .unwrap_or_else(|e| panic!("AssertReadable #{ord} failed: {e:?}"));
                    assert_eq!(r.after().selected_commit().ordinal(), ord);
                }
            }
            WbCmd::AssertProtectedInAncestry { ordinals } => {
                let fx = fixture.as_ref().unwrap();
                let branches = fx
                    .home
                    .world()
                    .observation_port()
                    .observe_installed_product_branches(&exec)
                    .unwrap();
                let mut reachable = std::collections::BTreeSet::new();
                for b in branches.branches() {
                    let tr = fx
                        .home
                        .world()
                        .inspection_port()
                        .trace_ancestry(
                            b.selected_commit().clone(),
                            std::num::NonZeroUsize::new(256).unwrap(),
                        )
                        .unwrap();
                    for c in tr.commits() {
                        reachable.insert(c.identity().ordinal());
                    }
                }
                for ord in ordinals {
                    assert!(
                        reachable.contains(&ord),
                        "protected ancestor commit #{ord} must remain installed in live ancestry (reachable={reachable:?})"
                    );
                }
            }
            WbCmd::CheckpointAndDiscard => {
                dgen = Some(checkpoint_and_discard_world_home(fixture.as_ref().unwrap(), &exec).1);
            }
            WbCmd::Close { expect_discarded } => {
                let out = fixture
                    .take()
                    .unwrap()
                    .home
                    .close(&exec)
                    .unwrap_or_else(|e| panic!("wb close refused: {:?}", e.reason()));
                let cgen = match out {
                    ApplicationHomeCloseOutcome::Closed {
                        checkpoint: Err(ApplicationHomeCheckpointDenial::DerivedStateDiscarded),
                        ..
                    } if expect_discarded => None,
                    ApplicationHomeCloseOutcome::Closed {
                        checkpoint: Ok(cp), ..
                    } if !expect_discarded => Some(cp.generation()),
                    _ => {
                        panic!("unexpected wb close (discarded={expect_discarded})")
                    }
                };
                send(None, None, Vec::new(), None, None, cgen, true);
                return;
            }
        }
        send(
            fixture.as_ref(),
            ret_ord,
            reclaimed,
            custody_cont,
            dgen,
            None,
            false,
        );
    }
}

#[test]
fn c4b4_world_branch_barrier_child() {
    let Some(home_path) = std::env::var_os(ENV_HOME).map(PathBuf::from) else {
        return;
    };
    let markers = PathBuf::from(std::env::var_os(ENV_CH).unwrap());
    let build = std::env::var(ENV_BUILD).unwrap();
    let spec = std::env::var(ENV_BARRIER_SPEC).unwrap();
    let parts: Vec<&str> = spec.split('|').collect();
    assert_eq!(parts[0], "publish");
    let (branch, round, nth): (&str, usize, u64) = (
        parts[1],
        parts[2].parse().unwrap(),
        parts[3].parse().unwrap(),
    );
    let req = crate::workload::serial_request();
    let exec = ExecutionRequest::serial(&req);
    let media =
        FilesystemMediaAdmission::certification(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = media.fault_schedule_authority();
    let activation = authority.one_shot_activation();
    let gate = authority.pause_gate();
    let role = MediaOperationRole::SynchronizeFileState;
    let rule = authority
        .rule(role, 1, MediaFaultDirective::PauseAfter(gate.clone()))
        .for_nth_identified_operation_after_activation(
            activation.clone(),
            std::num::NonZeroU64::new(nth).unwrap(),
        );
    let schedule = authority.schedule(vec![rule]).unwrap();
    let done = Arc::new(AtomicBool::new(false));
    let ready = markers.join("ready");
    let marker = barrier_marker(role, nth);
    let done_bg = done.clone();
    let watcher = std::thread::spawn(move || {
        while !done_bg.load(Ordering::SeqCst) {
            if gate.reached_context().is_some() {
                crate::killed_child::announce(&ready, marker.as_bytes());
                crate::killed_child::park();
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    });
    let home_req = home_opening::with_media(
        home_opening::request_with_build(
            &home_path,
            crate::workload::runtime_builder(crate::schema_transition::SOURCE),
            build_id(&build),
        ),
        media.with_fault_schedule(schedule),
    );
    let fixture = world_home::open(home_req, &exec).unwrap();
    activation.arm().unwrap();
    publish_on(&fixture, &exec, branch, round);
    done.store(true, Ordering::SeqCst);
    let _ = watcher.join();
    let actual = activation.matching_operation_count();
    crate::killed_child::announce(
        &markers.join("finished"),
        format!("missed:actual={actual}").as_bytes(),
    );
}
