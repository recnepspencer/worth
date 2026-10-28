//! Saved public reads select both real heads; the fresh phase owns lifecycle.
use super::*;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    primary_graph::{
        RuntimeWorldBranchAdmissionDenial, RuntimeWorldBranchReplacementDenial,
        RuntimeWorldServiceDenial,
    },
    product::{
        WorthQueryProductBranch, WorthQueryProductBranchCreateError,
        WorthQueryProductBranchCreationDenial,
    },
};

fn pin(host: &host::Host, branch: WorthQueryProductBranch) -> WorthQueryApplicationReadObservation {
    host.with_host_advancement(|phase| {
        WorthQueryApplicationReadObservation::retain_on_branch(&phase, host.runtime(), branch)
    })
    .unwrap()
    .unwrap()
}

fn fork(host: &host::Host, read: &WorthQueryApplicationReadObservation) -> WorthQueryProductBranch {
    host.with_host_advancement(|phase| {
        read.fork_product_branch(host)
            .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
            .create(&phase)
    })
    .unwrap()
    .unwrap()
}

#[test]
fn saved_head_replacement_refuses_moved_images_and_retains_late_stop_cleanup() {
    // Native advancement checks the original controls at final handoff;
    // accounted advancement instead preserves a completed callback's value.
    let resources = crate::document_retention_model::host::world_resources::world_resources(
        256,
        4 * 1024 * 1024,
        256,
    )
    .with_execution_policy(worth_foundational::ExecutionRequestPolicy::for_workers(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        std::num::NonZeroUsize::MIN,
    ));
    let host = host::install_with_resources(resources);
    let root = pin(&host, host.current_world());
    let old = fork(&host, &root);
    host::edit_on(&host, old, Value::Primary(11), "saved-old-eleven");
    let saved_old = pin(&host, old);
    let candidate = fork(&host, &saved_old);
    host::edit_on(&host, candidate, Value::Secondary(19), "candidate-nineteen");
    let saved_candidate = pin(&host, candidate);
    host::edit_on(&host, old, Value::Primary(13), "old-moved-thirteen");
    let current_old = pin(&host, old);

    let stale_fork = host
        .with_host_advancement(|phase| {
            saved_old
                .fork_product_branch(&host)
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
        })
        .unwrap();
    assert!(matches!(
        stale_fork,
        Err(WorthQueryProductBranchCreateError::Creation(
            WorthQueryProductBranchCreationDenial::World(RuntimeWorldServiceDenial::Denied(
                RuntimeWorldBranchAdmissionDenial::StaleSourceHead
            ))
        ))
    ));
    let moved_old = host
        .with_host_advancement(|phase| {
            saved_old.retire_replaced_head(&phase, &host, saved_candidate.retain())
        })
        .unwrap();
    assert!(matches!(
        moved_old,
        Err(WorthQueryApplicationHeadReplacementDenial::World(
            RuntimeWorldServiceDenial::Denied(RuntimeWorldBranchReplacementDenial::OldHeadMoved)
        ))
    ));
    assert_eq!(
        pin(&host, old).selected_commit(),
        current_old.selected_commit()
    );
    assert_eq!(host::read_on(&host, old)[0].primary, 13);
    assert_eq!(host::read_on(&host, candidate)[0].secondary, 19);

    host::edit_on(
        &host,
        candidate,
        Value::Secondary(23),
        "candidate-moved-twenty-three",
    );
    let current_candidate = pin(&host, candidate);
    let moved_candidate = host
        .with_host_advancement(|phase| {
            current_old.retire_replaced_head(&phase, &host, saved_candidate.retain())
        })
        .unwrap();
    assert!(matches!(
        moved_candidate,
        Err(WorthQueryApplicationHeadReplacementDenial::World(
            RuntimeWorldServiceDenial::Denied(
                RuntimeWorldBranchReplacementDenial::CandidateHeadMoved
            )
        ))
    ));
    assert_eq!(
        pin(&host, old).selected_commit(),
        current_old.selected_commit()
    );
    assert_eq!(
        pin(&host, candidate).selected_commit(),
        current_candidate.selected_commit()
    );
    let foreign = host::install();
    let foreign_phase = foreign
        .with_host_advancement(|phase| {
            current_old.retire_replaced_head(&phase, &host, current_candidate.retain())
        })
        .unwrap();
    assert!(matches!(
        foreign_phase,
        Err(WorthQueryApplicationHeadReplacementDenial::ForeignPhase)
    ));
    let foreign_read = pin(&foreign, foreign.current_world());
    let foreign_candidate = host
        .with_host_advancement(|phase| {
            current_old.retire_replaced_head(&phase, &host, foreign_read)
        })
        .unwrap();
    assert!(matches!(foreign_candidate, Err(WorthQueryApplicationHeadReplacementDenial::ReadAdmission(
        worth_query_host::facade::product::WorthQueryProductBranchAdmissionDenial::Refused(
            worth_query_host::facade::product::WorthQueryProductBranchAdmissionRefusalKind::ForeignOwner
        )
    ))));

    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    let mut terminal = None;
    let outer = host.with_application_advancement(&scope, |phase| {
        terminal = Some(
            current_old
                .retire_replaced_head(&phase, &host, current_candidate.retain())
                .unwrap(),
        );
        // The real effect was returned and saved outside this callback before
        // the caller stops its original request at the outer handoff.
        cancellation.cancel();
    });
    assert!(
        matches!(
            outer,
            Err(WorthQueryAdvancementDenial::Interrupted(
                WorthQueryManagedComputationInterruption::Cancelled
            ))
        ),
        "native outer advancement must observe the late stop: {outer:?}"
    );
    let retired = terminal.expect("actual retired custody survives the stopped outer handoff");
    assert_eq!(retired.retired_head(), current_old.selected_commit());
    assert!(
        retired.handoff_stop().is_none(),
        "this stop occurred after the inner lifecycle handoff"
    );
    let retained_candidate = retired.into_candidate();
    assert_eq!(
        retained_candidate.selected_commit(),
        current_candidate.selected_commit()
    );
    let now_retired = host
        .with_host_advancement(|phase| {
            WorthQueryApplicationReadObservation::retain_on_branch(&phase, host.runtime(), old)
        })
        .unwrap();
    assert!(matches!(now_retired, Err(worth_query_host::facade::product::WorthQueryProductBranchAdmissionDenial::Refused(
        worth_query_host::facade::product::WorthQueryProductBranchAdmissionRefusalKind::RetiredBranch
    ))));
    assert_eq!(host::read_on(&host, candidate)[0].primary, 11);
    assert_eq!(host::read_on(&host, candidate)[0].secondary, 23);
    drop((current_old, saved_old, saved_candidate));
    let pending = host.branches().pending_cleanup();
    assert_eq!(
        pending.len(),
        1,
        "actual retirement cleanup remained discoverable after cancellation"
    );
    for cleanup in pending {
        cleanup.retry().unwrap();
    }
    assert!(host.branches().pending_cleanup().is_empty());
    assert_eq!(
        pin(&host, candidate).selected_commit(),
        retained_candidate.selected_commit()
    );
}
