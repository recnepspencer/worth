//! All six in-flight Bank external-rail dispatch fault cases under real OS kill
//! (attempt durable, observation not yet recorded).
use super::*;

#[test]
fn application_kill_in_flight_reconciles_all_six_bank_rail_fault_cases_without_second_effect() {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = checkpoint_capture::ExecutionRequest::leased(&lease);

    // 1. Exactly-once completion: killed between dispatch and response recording.
    // Query admits reconciliation and Bank's read-only reconciliation inquiry
    // resolves the rail's completed effect with ZERO extra dispatch.
    {
        let world = World::new();
        let key = 9501_u64;
        let expected_notice = notice(key);
        let info: ChildDispatchInfo =
            serde_json::from_slice(&spawn_and_kill_child(&world, "dispatch:succeed", key, None))
                .unwrap();
        let reopened = world.open(execution, false);
        assert_unresolved_unknown(&reopened, &info.correlation, "succeed");
        let rc = rail_correlation(&info.correlation);
        let handle = reopened.recover(expected_notice, key);
        let disposition = reopened.admitted(expected_notice, key, |phase, admission, _, _| {
            let rt = reopened.home.runtime().runtime();
            let err = rt.settle_recorded_recovery_completion(phase, handle, admission).unwrap_err();
            assert_eq!(err.kind(), WorthQueryRecoveryHandleDenialKind::UnresolvedExternalPosture);
            let handle = err.into_parts().1.expect("claim retained");
            let auth = rt.admit_recovery_effect_authority(&handle, admission).unwrap();
            let reconcile = reconcile_recovery_handle(phase, handle, &auth).unwrap();
            reopened.transport.reconcile_admitted_effect(&reconcile)
        });
        match disposition {
            BankRailReconciliationDisposition::Completed {
                correlation,
                notice: observed,
            } => {
                assert_eq!(correlation, rc);
                assert_eq!(observed.estate(), expected_notice.estate().get());
                assert_eq!(observed.notice(), expected_notice.notice().get());
                assert_eq!(observed.subject(), expected_notice.subject().get());
            }
            other => panic!("expected Completed reconciliation disposition, got {other:?}"),
        }
        assert_eq!(
            reopened.admitted(expected_notice, key, |phase, admission, binding, _| {
                reopened
                    .home
                    .runtime()
                    .runtime()
                    .resolve_admitted_recovery_claim_status(phase, admission, binding)
                    .unwrap()
                    .unwrap()
            }),
            WorthQueryRecoveryClaimStatus::Consumed
        );
        assert!(
            reopened.transport.production_dispatches().is_empty(),
            "reconcile never redispatches completed effect"
        );
        assert_eq!(reopened.transport.admission_count(), 1);
        assert_eq!(reopened.transport.completed_effect_count(), 1);
        reopened.close(&execution);
        world.finish();
    }

    // 2. Lost response (CommitThenLoseResponse): killed before observation recorded;
    // safe-retry deduplicates on the rail with zero second effect.
    {
        let world = World::new();
        let key = 9502_u64;
        let info: ChildDispatchInfo = serde_json::from_slice(&spawn_and_kill_child(
            &world,
            "dispatch:lost-response",
            key,
            None,
        ))
        .unwrap();
        let reopened = world.open(execution, false);
        assert_unresolved_unknown(&reopened, &info.correlation, "lost-response");
        let rc = rail_correlation(&info.correlation);
        assert_eq!(reopened.transport.ledger_status(&rc), LedgerStatus::Completed);
        let handle = reopened.recover(notice(key), key);
        reopened.transport.under(FaultScript::Succeed, Duration::from_secs(5));
        reopened.admitted(notice(key), key, |phase, admission, _, _| {
            let rt = reopened.home.runtime().runtime();
            let auth = rt.admit_recovery_effect_authority(&handle, admission).unwrap();
            let redisp = rt.redispatch_admitted_external_effect(phase, &handle, &auth, admission).unwrap();
            safe_retry_recovery_handle(phase, handle, &auth, redisp).unwrap();
        });
        assert_eq!(reopened.transport.admission_count(), 1);
        assert_eq!(reopened.transport.completed_effect_count(), 1);
        reopened.close(&execution);
        world.finish();
    }

    // 3. Duplicate acknowledgement: killed before observation recorded; rail holds
    // Acknowledged, redispatch observes Acknowledged without a second admission or
    // completion, safe-retry refuses UnresolvedExternalPosture, and reconcile settles
    // the recovery claim under the rail's contract.
    {
        let world = World::new();
        let key = 9503_u64;
        let info: ChildDispatchInfo =
            serde_json::from_slice(&spawn_and_kill_child(&world, "dispatch:dup-ack", key, None))
                .unwrap();
        let reopened = world.open(execution, false);
        assert_unresolved_unknown(&reopened, &info.correlation, "dup-ack");
        let rc = rail_correlation(&info.correlation);
        assert_eq!(reopened.transport.ledger_status(&rc), LedgerStatus::Acknowledged);
        assert_eq!(reopened.transport.admission_count(), 1);
        assert_eq!(reopened.transport.completed_effect_count(), 0);
        let handle = reopened.recover(notice(key), key);
        reopened.transport.under(FaultScript::Succeed, Duration::from_secs(5));
        let disposition = reopened.admitted(notice(key), key, |phase, admission, _, _| {
            let rt = reopened.home.runtime().runtime();
            let auth = rt.admit_recovery_effect_authority(&handle, admission).unwrap();
            let redisp = rt.redispatch_admitted_external_effect(phase, &handle, &auth, admission).unwrap();
            assert_eq!(
                redisp.dispatch().posture().kind(),
                WorthQueryExternalDispatchPostureKind::Acknowledged
            );
            let err = safe_retry_recovery_handle(phase, handle, &auth, redisp).unwrap_err();
            assert_eq!(err.kind(), WorthQueryRecoveryHandleDenialKind::UnresolvedExternalPosture);
            let retained = err.into_parts().1.expect("claim retained");
            let reconcile = reconcile_recovery_handle(phase, retained, &auth).unwrap();
            reopened.transport.reconcile_admitted_effect(&reconcile)
        });
        assert_eq!(
            disposition,
            BankRailReconciliationDisposition::AcknowledgedPending {
                correlation: rc.clone(),
            }
        );
        assert_eq!(
            reopened
                .home
                .runtime()
                .runtime()
                .durable_dispatch_custody(&info.correlation)
                .unwrap()
                .observation(),
            Some(QueryDispatchRecordedObservation::Acknowledged)
        );
        assert_eq!(reopened.transport.ledger_status(&rc), LedgerStatus::Acknowledged);
        assert_eq!(reopened.transport.admission_count(), 1);
        assert_eq!(reopened.transport.completed_effect_count(), 0);
        reopened.close(&execution);
        world.finish();
    }

    // 4. Completion after timeout (CompleteAfterDelay): killed while timed out;
    // authentic callback completes original dispatch with zero redispatch.
    {
        let mut world = World::new();
        let key = 9504_u64;
        let info: ChildDispatchInfo =
            serde_json::from_slice(&spawn_and_kill_child(&world, "dispatch:timeout", key, None))
                .unwrap();
        let reopened = world.open(execution, false);
        assert_unresolved_unknown(&reopened, &info.correlation, "timeout");
        let body = world.callback.receive(&world.capture_loop.blocking);
        let rx = reopened
            .home
            .runtime()
            .runtime()
            .receive_inbound_occurrence(&reopened.verifier, &body, &request_scope())
            .unwrap();
        assert_eq!(rx.posture(), WorthQueryInboundReceiptPosture::Performed);
        drop(rx);
        reopened.maintain();
        assert!(reopened
            .home
            .runtime()
            .runtime()
            .observe_inbound_terminal(&reopened.verifier, info.correlation)
            .is_some());
        let handle = reopened.recover(notice(key), key);
        reopened.admitted(notice(key), key, |phase, admission, _, _| {
            reopened
                .home
                .runtime()
                .runtime()
                .settle_recorded_recovery_completion(phase, handle, admission)
                .unwrap();
        });
        assert!(reopened.transport.production_dispatches().is_empty());
        assert_eq!(reopened.transport.admission_count(), 1);
        assert_eq!(reopened.transport.completed_effect_count(), 1);
        reopened.close(&execution);
        world.finish();
    }

    // 5. Disappearance mid-dispatch: rail has NoRecord; safe-retry dispatches once.
    {
        let world = World::new();
        let key = 9505_u64;
        let info: ChildDispatchInfo = serde_json::from_slice(&spawn_and_kill_child(
            &world,
            "dispatch:disappear",
            key,
            None,
        ))
        .unwrap();
        let reopened = world.open(execution, false);
        assert_unresolved_unknown(&reopened, &info.correlation, "disappear");
        let rc = rail_correlation(&info.correlation);
        assert_eq!(reopened.transport.ledger_status(&rc), LedgerStatus::NoRecord);
        assert_eq!(reopened.transport.admission_count(), 0);
        let handle = reopened.recover(notice(key), key);
        reopened.transport.under(FaultScript::Succeed, Duration::from_secs(5));
        reopened.admitted(notice(key), key, |phase, admission, _, _| {
            let rt = reopened.home.runtime().runtime();
            let auth = rt.admit_recovery_effect_authority(&handle, admission).unwrap();
            let redisp = rt.redispatch_admitted_external_effect(phase, &handle, &auth, admission).unwrap();
            safe_retry_recovery_handle(phase, handle, &auth, redisp).unwrap();
        });
        assert_eq!(reopened.transport.admission_count(), 1);
        assert_eq!(reopened.transport.completed_effect_count(), 1);
        reopened.close(&execution);
        world.finish();
    }

    // 6. Semantic rejection: killed in flight; reopen shows unresolved, then
    // redispatch against the rail reads ProvenNoEffect(PayloadRejected) with zero admission.
    {
        let world = World::new();
        let key = 9506_u64;
        let info: ChildDispatchInfo =
            serde_json::from_slice(&spawn_and_kill_child(&world, "dispatch:reject", key, None))
                .unwrap();
        let reopened = world.open(execution, false);
        assert_unresolved_unknown(&reopened, &info.correlation, "reject");
        assert_eq!(reopened.transport.admission_count(), 0);
        assert_eq!(reopened.transport.completed_effect_count(), 0);
        let handle = reopened.recover_reject(notice(key), key);
        let disposition = reopened.admitted_reject(notice(key), key, |phase, admission, _, _| {
            let rt = reopened.home.runtime().runtime();
            let auth = rt.admit_recovery_effect_authority(&handle, admission).unwrap();
            let performed = rt
                .redispatch_admitted_external_effect(phase, &handle, &auth, admission)
                .unwrap();
            assert_eq!(
                performed.dispatch().posture().kind(),
                WorthQueryExternalDispatchPostureKind::ProvenNoEffect
            );
            let reconcile = reconcile_recovery_handle(phase, handle, &auth).unwrap();
            reopened.transport.reconcile_admitted_effect(&reconcile)
        });
        assert_eq!(
            disposition,
            BankRailReconciliationDisposition::UnresolvedNoRecord {
                correlation: rail_correlation(&info.correlation),
            }
        );
        assert_eq!(
            reopened
                .home
                .runtime()
                .runtime()
                .durable_dispatch_custody(&info.correlation)
                .unwrap()
                .observation(),
            Some(QueryDispatchRecordedObservation::ProvenNoEffect(
                ExternalEffectNoEffectCause::PayloadRejected
            ))
        );
        assert_eq!(reopened.transport.admission_count(), 0);
        assert_eq!(reopened.transport.completed_effect_count(), 0);
        reopened.close(&execution);
        world.finish();
    }
}
