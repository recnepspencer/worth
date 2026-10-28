//! Actual installed branches, original requests, and paired lifecycle effects.
use super::*;
use crate::branch::RuntimeWorldBranchReplacementDenial as Denial;
use worth_execution::{CancellationSource, ExecutionRequest, SerialRequest};

fn request(owner: &TestOwner, cancellation: &CancellationSource) -> SerialRequest {
    SerialRequest::from_policy(
        &owner.state.execution.request_policy(),
        cancellation.token(),
        None,
    )
}

#[test]
fn replacement_checks_both_saved_heads_and_keeps_both_live_on_refusal() {
    let (mut fixture, owner, old) = fork_creation::setup_with_relational_source(3);
    let candidate = create_reused_branch(&owner, &old, reuse_intent("saved-candidate"));
    let cancellation = CancellationSource::new();
    let host = request(&owner, &cancellation);
    let execution = ExecutionRequest::serial(&host);
    assert!(matches!(
        owner.retire_replaced_head(execution, &old, &old),
        Err(Denial::SameOccurrence)
    ));
    fork_creation::seed_relational_source(&owner, &mut fixture, old.clone());
    assert!(matches!(
        owner.retire_replaced_head(execution, &old, &candidate),
        Err(Denial::OldHeadMoved)
    ));
    assert_eq!(owner.state.branches.branch_count(), 2);
    assert_eq!(
        RuntimeWorldObservationService::observe_product_branch(&owner, candidate.branch_identity())
            .unwrap(),
        candidate
    );

    let current = fork_creation::current_root_observation(&owner);
    // Swap roles: the genuine moved root is now the saved candidate, so its
    // movement cannot be hidden behind a still-current old child.
    assert!(matches!(
        owner.retire_replaced_head(execution, &candidate, &old),
        Err(Denial::CandidateHeadMoved)
    ));
    let (_other_fixture, other, foreign) = super::setup(2);
    assert!(matches!(
        owner.retire_replaced_head(execution, &candidate, &foreign),
        Err(Denial::ForeignOwner)
    ));
    drop(other);
    cancellation.cancel();
    assert!(matches!(
        owner.retire_replaced_head(execution, &candidate, &current),
        Err(Denial::Request(_))
    ));
    assert_eq!(owner.state.branches.branch_count(), 2);
    assert_eq!(
        RuntimeWorldObservationService::observe_product_branch(&owner, candidate.branch_identity())
            .unwrap(),
        candidate
    );
}

#[test]
fn replacement_busy_candidate_is_pre_effect_and_lawful_retry_retires_only_old() {
    let (_fixture, owner, source) = super::setup(3);
    let old = create_reused_branch(&owner, &source, reuse_intent("old-head"));
    let candidate = create_reused_branch(&owner, &old, reuse_intent("new-head"));
    let cell = owner
        .state
        .branches
        .branch_cell(candidate.branch_identity())
        .unwrap();
    let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
    let (release_tx, release_rx) = std::sync::mpsc::sync_channel(1);
    let held = candidate.clone();
    let worker = std::thread::spawn(move || {
        // This callback only signals/waits. The other thread's bounded write
        // attempt must refuse; the callback never reenters its own cell.
        cell.while_current(&held, (), |()| {
            ready_tx.send(()).unwrap();
            release_rx
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap();
        })
        .unwrap();
    });
    ready_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    let cancellation = CancellationSource::new();
    let host = request(&owner, &cancellation);
    let execution = ExecutionRequest::serial(&host);
    assert!(matches!(
        owner.retire_replaced_head(execution, &old, &candidate),
        Err(Denial::CellBusy)
    ));
    assert_eq!(owner.state.branches.branch_count(), 3);
    release_tx.send(()).unwrap();
    worker.join().unwrap();
    let report = owner
        .retire_replaced_head(execution, &old, &candidate)
        .unwrap();
    assert_eq!(report.retired_head(), old.selected_commit());
    assert!(report.handoff_stop().is_none());
    assert!(matches!(
        RuntimeWorldObservationService::observe_product_branch(&owner, old.branch_identity()),
        Err(RuntimeWorldBranchAdmissionDenial::RetiredBranch)
    ));
    assert_eq!(
        RuntimeWorldObservationService::observe_product_branch(&owner, candidate.branch_identity())
            .unwrap(),
        candidate
    );
    assert_eq!(owner.state.branches.branch_count(), 2);
}

#[test]
fn replacement_retains_real_component_work_and_late_original_stop() {
    use worth_execution::{ExecutionResourceLease, LeaseRequest, MapKernelStop, WorkCeilingDenial};
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let (_fixture, owner, source) = fork_creation::setup_with_relational_source(3);
    let old = create_reused_branch(
        &owner,
        &source,
        relational_fork_intent("funded-old", "funded-old-relational"),
    );
    let cancellation = CancellationSource::new();
    let lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(2_000_000),
                Some(1_000_000),
            ),
        ),
        cancellation: cancellation.token(),
        deadline: None,
    })
    .unwrap();
    let before = lease.charged_memory_bytes();
    let stop = cancellation.clone();
    let _rehearsal =
        super::super::replacement::testing::rehearse(owner.owner_identity(), move || stop.cancel());
    let report = owner
        .retire_replaced_head(ExecutionRequest::leased(&lease), &old, &source)
        .unwrap();
    assert!(matches!(
        report.handoff_stop(),
        Some(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled))
    ));
    assert_eq!(report.retired_incarnation(), old.lifecycle_incarnation());
    assert!(lease.charged_memory_bytes() > before);
    let (_, _, work, stop) = report.into_cleanup_parts();
    assert!(stop.is_some());
    assert_eq!(work.as_slice().len(), 1);
    assert_eq!(work.as_slice()[0].target_name(), "funded-old-relational");
    assert!(owner.state.custody.installed_records().is_empty());
    assert_eq!(
        RuntimeWorldObservationService::observe_product_branch(&owner, source.branch_identity())
            .unwrap(),
        source
    );
    drop(work);
    assert_eq!(
        lease.charged_memory_bytes(),
        before,
        "last backing drop releases its original payer"
    );
}
