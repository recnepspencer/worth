use worth_query::facade::{domain, foundation, read};

use super::installed_operation_fixture::{
    configured_runtime, required_domain_runtime, AuxiliaryDomain, GeometryDomain,
    ReadExecutionInput, ReadFamily, ReadVertex,
};

type SettledProjection = domain::WorthQuerySettledDomainProjection<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
>;

#[test]
fn cancellation_terminates_while_disposal_detaches_and_stale_cleanup_still_closes() {
    let mut workspace = configured_runtime()
        .controlled_workspace("projection-lifecycle-terminal-causes")
        .unwrap();
    let cancelled = settle(&mut workspace);
    let cancelled_live = promote(cancelled, &mut workspace);
    let cancelled_resource = cancelled_live.resource_name().to_string();
    let cancelled = match cancelled_live.cancel(&mut workspace) {
        domain::WorthQueryProjectionCancellationOutcome::Cancelled(cancelled) => cancelled,
        domain::WorthQueryProjectionCancellationOutcome::Stopped(_) => {
            panic!("cancellation unexpectedly stopped")
        }
    };
    assert_eq!(
        cancelled.close_receipt().cause(),
        domain::WorthQueryProjectionLifecycleCloseCause::Cancellation
    );
    assert_eq!(
        cancelled
            .close_receipt()
            .ordinary()
            .closeout_kind()
            .as_str(),
        "consumer_terminated"
    );
    assert!(workspace
        .resolve_live_artifact_target(&cancelled_resource)
        .is_err());
    let disposed_after_cancel = cancelled.dispose();
    assert_eq!(
        disposed_after_cancel
            .close_receipt()
            .ordinary()
            .closeout_kind()
            .as_str(),
        "consumer_terminated"
    );

    let disposed = settle(&mut workspace);
    let disposed_live = promote(disposed, &mut workspace);
    let disposed_resource = disposed_live.resource_name().to_string();
    workspace.advance_domain_installation_generation().unwrap();
    let disposed = match disposed_live.dispose(&mut workspace) {
        domain::WorthQueryProjectionDisposalOutcome::Disposed(disposed) => disposed,
        domain::WorthQueryProjectionDisposalOutcome::Stopped(_) => {
            panic!("stale owner cleanup must remain available")
        }
    };
    assert_eq!(
        disposed.close_receipt().cause(),
        domain::WorthQueryProjectionLifecycleCloseCause::Disposal
    );
    assert_eq!(
        disposed.close_receipt().ordinary().closeout_kind().as_str(),
        "consumer_detached"
    );
    assert_eq!(disposed.close_receipt().counters().close_attempts, 1);
    assert_eq!(disposed.close_receipt().counters().close_completions, 1);
    assert!(workspace
        .resolve_live_artifact_target(&disposed_resource)
        .is_err());
}

#[test]
fn replacement_readmits_the_exact_pair_and_preserves_the_old_owner_on_wrong_pair() {
    let mut workspace = configured_runtime()
        .workspace("projection-lifecycle-replacement")
        .unwrap();
    let settled = settle(&mut workspace);
    let live = promote(settled, &mut workspace);
    let old_resource = live.resource_name().to_string();
    let candidate = settle(&mut workspace).into_lifecycle();
    let witness = live.replacement_witness_for(&candidate).unwrap();
    let replaced = match live.replace_with(candidate, witness, &mut workspace) {
        domain::WorthQueryProjectionReplacementOutcome::Replaced(replaced) => replaced,
        _ => panic!("exact replacement pair did not converge"),
    };
    assert_eq!(replaced.transition_work().compatibility_readmissions(), 1);
    assert!(
        replaced
            .replacement_witness()
            .counters()
            .canonical_comparisons
            > 0
    );
    assert_eq!(replaced.transition_work().candidate().lifecycle_attempts, 1);
    assert_eq!(
        replaced.predecessor_close_receipt().cause(),
        domain::WorthQueryProjectionLifecycleCloseCause::Replacement
    );
    assert!(workspace
        .resolve_live_artifact_target(&old_resource)
        .is_err());
    replaced.refresh(&mut workspace).unwrap();
    let disposed = match replaced.dispose(&mut workspace) {
        domain::WorthQueryTransitionedProjectionDisposalOutcome::Disposed(disposed) => disposed,
        domain::WorthQueryTransitionedProjectionDisposalOutcome::Stopped(_) => {
            panic!("replaced projection disposal stopped")
        }
    };
    assert!(matches!(
        disposed.prior_transition(),
        Some(domain::WorthQueryProjectionPriorTransitionEvidence::Replacement { .. })
    ));

    let settled = settle(&mut workspace);
    let live = promote(settled, &mut workspace);
    let candidate = settle(&mut workspace).into_lifecycle();
    let wrong_candidate = settle(&mut workspace).into_lifecycle();
    let wrong_witness = live.replacement_witness_for(&wrong_candidate).unwrap();
    let stop = match live.replace_with(candidate, wrong_witness, &mut workspace) {
        domain::WorthQueryProjectionReplacementOutcome::Stopped(stop) => stop,
        _ => panic!("wrong-pair replacement witness was accepted"),
    };
    assert_eq!(
        stop.kind(),
        domain::WorthQueryProjectionTransitionDenialKind::WrongCompatibilityPair
    );
    assert_eq!(stop.work().compatibility_readmissions(), 1);
    assert_eq!(stop.work().candidate().planning_attempts, 0);
    assert_eq!(stop.work().candidate().lower_runtime_contacts, 0);
    let (live, _) = stop.into_retry_parts();
    live.refresh(&mut workspace).unwrap();
    drop(live);
    drop(wrong_candidate);
}

#[test]
fn close_failure_owns_both_resources_until_retry_or_rollback() {
    let mut workspace = configured_runtime()
        .fail_next_live_closes(1)
        .workspace("projection-lifecycle-cleanup-retry")
        .unwrap();
    let settled = settle(&mut workspace);
    let live = promote(settled, &mut workspace);
    let candidate = settle(&mut workspace).into_lifecycle();
    let witness = live.replacement_witness_for(&candidate).unwrap();
    let pending = match live.replace_with(candidate, witness, &mut workspace) {
        domain::WorthQueryProjectionReplacementOutcome::CleanupPending(pending) => pending,
        _ => panic!("injected predecessor close failure did not retain cleanup ownership"),
    };
    workspace
        .resolve_live_artifact_target(pending.predecessor_resource_name())
        .unwrap();
    workspace
        .resolve_live_artifact_target(pending.successor_resource_name())
        .unwrap();
    assert_eq!(pending.cleanup_work().predecessor_close_attempts(), 1);
    assert_eq!(pending.cleanup_work().predecessor_close_completions(), 0);
    let replaced = match pending.retry_cleanup(&mut workspace) {
        domain::WorthQueryReplacementCleanupRetryOutcome::Replaced(replaced) => replaced,
        domain::WorthQueryReplacementCleanupRetryOutcome::Pending(_) => {
            panic!("single injected failure did not recover on retry")
        }
    };
    assert_eq!(replaced.cleanup_work().predecessor_close_attempts(), 2);
    assert_eq!(replaced.cleanup_work().predecessor_close_completions(), 1);
    replaced.refresh(&mut workspace).unwrap();
    drop(replaced);

    let mut workspace = configured_runtime()
        .fail_next_live_closes(1)
        .workspace("projection-lifecycle-cleanup-rollback")
        .unwrap();
    let settled = settle(&mut workspace);
    let live = promote(settled, &mut workspace);
    let candidate = settle(&mut workspace).into_lifecycle();
    let witness = live.replacement_witness_for(&candidate).unwrap();
    let pending = match live.replace_with(candidate, witness, &mut workspace) {
        domain::WorthQueryProjectionReplacementOutcome::CleanupPending(pending) => pending,
        _ => panic!("injected close failure did not enter cleanup pending"),
    };
    let successor_resource = pending.successor_resource_name().to_string();
    let restored = match pending.rollback(&mut workspace) {
        domain::WorthQueryReplacementRollbackOutcome::Restored {
            live,
            receipt,
            work,
        } => {
            assert_eq!(
                receipt.cause(),
                domain::WorthQueryProjectionLifecycleCloseCause::ReplacementRollback
            );
            assert_eq!(work.predecessor_close_attempts(), 1);
            assert_eq!(work.rollback_close_attempts(), 1);
            assert_eq!(work.rollback_close_completions(), 1);
            live
        }
        domain::WorthQueryReplacementRollbackOutcome::Pending(_) => {
            panic!("rollback did not close the successor")
        }
    };
    assert!(workspace
        .resolve_live_artifact_target(&successor_resource)
        .is_err());
    restored.refresh(&mut workspace).unwrap();
    drop(restored);
}

#[test]
fn rebind_requires_the_stale_current_pair_and_refreshes_the_successor() {
    let mut workspace = configured_runtime()
        .controlled_workspace("projection-lifecycle-rebind")
        .unwrap();
    let prior_domain = workspace.domain(GeometryDomain).unwrap();
    let settled = settle(&mut workspace);
    let live = promote(settled, &mut workspace);
    let old_resource = live.resource_name().to_string();
    workspace.advance_domain_installation_generation().unwrap();
    let (_, rebind_receipt) = workspace
        .rebind_domain(prior_domain.rebind_request())
        .unwrap()
        .into_parts();
    let candidate = settle(&mut workspace).into_lifecycle();
    let witness = live.rebind_witness_for(&candidate, rebind_receipt).unwrap();
    let rebound = match live.rebind_with(candidate, witness, &mut workspace) {
        domain::WorthQueryProjectionRebindOutcome::Rebound(rebound) => rebound,
        _ => panic!("exact stale/current rebind pair did not converge"),
    };
    assert_eq!(rebound.transition_work().compatibility_readmissions(), 1);
    assert!(rebound.rebind_witness().counters().canonical_comparisons > 0);
    assert_eq!(rebound.transition_work().candidate().lifecycle_attempts, 1);
    assert_eq!(
        rebound.predecessor_close_receipt().cause(),
        domain::WorthQueryProjectionLifecycleCloseCause::Rebind
    );
    assert!(workspace
        .resolve_live_artifact_target(&old_resource)
        .is_err());
    rebound.refresh(&mut workspace).unwrap();
    let cancelled = match rebound.cancel(&mut workspace) {
        domain::WorthQueryTransitionedProjectionCancellationOutcome::Cancelled(cancelled) => {
            cancelled
        }
        domain::WorthQueryTransitionedProjectionCancellationOutcome::Stopped(_) => {
            panic!("rebound projection cancellation stopped")
        }
    };
    assert!(matches!(
        cancelled.prior_transition(),
        Some(domain::WorthQueryProjectionPriorTransitionEvidence::Rebind { .. })
    ));
}

#[test]
fn lifecycle_rebind_carries_required_domain_owner_receipts() {
    let mut workspace = required_domain_runtime(true)
        .controlled_workspace("projection-lifecycle-required-domain-rebind")
        .unwrap();
    let prior_geometry = workspace.domain(GeometryDomain).unwrap();
    let prior_auxiliary = workspace.domain(AuxiliaryDomain).unwrap();
    let settled = settle(&mut workspace);
    let live = promote(settled, &mut workspace);
    workspace.advance_domain_installation_generation().unwrap();
    let geometry = workspace
        .rebind_domain(prior_geometry.rebind_request())
        .unwrap();
    let auxiliary = workspace
        .rebind_domain(prior_auxiliary.rebind_request())
        .unwrap();
    let candidate = settle(&mut workspace).into_lifecycle();
    assert!(live
        .rebind_witness_for(&candidate, geometry.receipt().clone())
        .is_err());
    let witness = live
        .rebind_witness_for_with_required_domains(
            &candidate,
            geometry.receipt().clone(),
            vec![auxiliary.receipt().clone()],
        )
        .unwrap();
    let rebound = match live.rebind_with(candidate, witness, &mut workspace) {
        domain::WorthQueryProjectionRebindOutcome::Rebound(rebound) => rebound,
        _ => panic!("required-domain lifecycle rebind did not converge"),
    };
    assert!(
        rebound
            .rebind_witness()
            .counters()
            .required_domain_rebind_receipts_inspected
            > 0
    );
}

fn promote(
    settled: SettledProjection,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) -> domain::WorthQueryLiveBoundDomainProjection<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
> {
    match settled.into_lifecycle().promote(workspace) {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("settled projection did not promote"),
    }
}

fn settle(workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace) -> SettledProjection {
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
        .unwrap();
    let consumer = bound.consumer_projection_contract().unwrap();
    bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &*workspace,
        )
        .unwrap()
        .execute(workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume(consumer, read::project_facts().entity_identities())
        .unwrap()
        .settle()
        .unwrap()
}
