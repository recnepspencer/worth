//! These controls contact the same materialization reader their refused call cannot reach.
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Denial,
        WorthQueryManagedComputationResourceDenial as Resource,
    },
    primary_graph::{
        bound_advancement_requests_on_this_thread_for_test as bound,
        installed_source_reads_on_this_thread_for_test as reads,
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryExecutionPlacementForTest as Placement,
        WorthQueryGeneratedOutputRestorationFailure as RestorationFailure,
        WorthQueryGeneratedOutputRestorationFailureCause as Cause,
        WorthQueryGeneratedOutputSuspensionFailure as SuspensionFailure,
        WorthQueryProviderSessionDenialKind as ProviderKind,
    },
};
struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}
fn limit(memory: bool) {
    let policy = support::observation::CHECKPOINT_EXECUTION_POLICY
        .accounting()
        .expect("this fixture declares accounting");
    bound(Some(worth_foundational::ExecutionBudget::new(
        NonZeroUsize::MIN,
        if memory {
            Some(0)
        } else {
            policy.charged_memory_bytes()
        },
        if memory {
            policy.work_ceiling()
        } else {
            Some(0)
        },
    )));
}
fn assert_cause(cause: Denial, memory: bool) {
    if !memory {
        assert_eq!(cause, Denial::Resource(Resource::WorkExhausted));
    } else {
        let Denial::Resource(Resource::MemoryLimit {
            level,
            requested,
            admitted,
        }) = cause
        else {
            panic!("exact lease cause: {cause:?}")
        };
        assert_eq!(
            level,
            worth_query_host::facade::application_contribution::WorthQueryMemoryLimitLevel::Policy
        );
        assert!(requested > 0);
        assert_eq!(admitted, 0);
    }
}
#[test]
fn suspension_and_restoration_refuse_before_their_real_reader_at_both_placements() {
    let _guard = checkpoint_recovery_test_guard();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        for memory in [false, true] {
            bound(None);
            let application = install(None);
            let (scope, principal) = authenticate(&application);
            let request = application.request(&principal, &scope);
            drop(settle(&request, &application));
            let ring = ring(&request);
            let source = request
                .query(PlanarRead {
                    body_key: ROOT.to_owned(),
                })
                .execute()
                .unwrap()
                .observed_sources()[0]
                .clone();
            let selected = crate::checkpoint_recovery::support::host_call(&application, |phase| {
                application
                    .on_branch(application.current_world())
                    .select(phase)
            })
            .unwrap();
            limit(memory);
            let before = reads();
            let Err(SuspensionFailure::ExecutionDenied(cause)) =
                selected.suspend_current_generated_output::<Final>(&scope, source.clone())
            else {
                panic!("suspension refuses at open")
            };
            assert_cause(cause, memory);
            assert_eq!(reads(), before);
            bound(None);
            let selected = crate::checkpoint_recovery::support::host_call(&application, |phase| {
                application
                    .on_branch(application.current_world())
                    .select(phase)
            })
            .unwrap();
            let before = reads();
            let suspended = selected
                .suspend_current_generated_output::<Final>(&scope, source)
                .unwrap_or_else(|_| panic!("admitted suspension"));
            assert!(
                reads() > before,
                "admitted suspension contacts its materialization reader"
            );
            let completed = reconstruct_ring(&application, suspended, &ring);
            limit(memory);
            let before = reads();
            let Err(RestorationFailure::Rejected {
                suspended,
                cause:
                    Cause::ExecutionDenied(ProviderKind::ExecutionResource {
                        denial: resource, ..
                    }),
            }) = application.restore_generated_output(completed, &scope)
            else {
                panic!("restoration carries its opening refusal")
            };
            assert_cause(Denial::Resource(resource), memory);
            assert_eq!(reads(), before);
            bound(None);
            let completed = reconstruct_ring(&application, suspended, &ring);
            let before = reads();
            application
                .restore_generated_output(completed, &scope)
                .unwrap_or_else(|_| panic!("admitted restoration"));
            assert!(
                reads() > before,
                "admitted restoration contacts its materialization reader"
            );
        }
    }
}

#[test]
fn ordinary_recovery_returns_cancelled_custody_then_resumes_suspension_and_restoration() {
    use worth_query_host::facade::application_contribution::WorthQueryManagedComputationInterruption;
    use worth_query_host::facade::primary_graph::{
        WorthQueryGeneratedOutputRestorationRecoveryStage as RestoreStage,
        WorthQueryGeneratedOutputSuspensionRecoveryStage as SuspendStage,
    };
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    drop(settle(&request, &application));
    let expected_ring = ring(&request);
    let source = request
        .query(PlanarRead {
            body_key: ROOT.to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let selected = support::host_call(&application, |phase| {
        application
            .on_branch(application.current_world())
            .select(phase)
    })
    .unwrap();
    support::host_call(&application, |phase| {
        application.fail_next_durable_append_for_test(
            &phase
                .checked_execution_request_for(application.product_runtime())
                .unwrap(),
        )
    });
    let recovery = match selected.suspend_current_generated_output::<Final>(&scope, source) {
        Err(SuspensionFailure::ProductUnpublished(recovery)) => recovery,
        _ => panic!("actual durable append failure retains suspension recovery"),
    };
    let branch = recovery.product_branch();
    let before = request.retain_current_read().unwrap();
    let cancellation = authentication::WorthQueryCancellationSource::new();
    let cancelled = authentication::WorthQueryRequestScope::new(None, cancellation.token());
    cancellation.cancel();
    let failed = match application
        .continue_generated_output_suspension_recovery::<Final>(recovery, &cancelled)
    {
        Err(failed) => failed,
        Ok(_) => panic!("cancelled admission cannot advance retained suspension"),
    };
    assert_eq!(
        failed.stage(),
        SuspendStage::ExecutionDenied(Denial::Interrupted(
            WorthQueryManagedComputationInterruption::Cancelled
        ),)
    );
    let recovery = failed.into_recovery();
    assert_eq!(recovery.product_branch(), branch);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        before.selected_commit()
    );
    let suspended = application
        .continue_generated_output_suspension_recovery::<Final>(recovery, &scope)
        .unwrap_or_else(|failed| panic!("same suspension resumes: {:?}", failed.stage()));
    assert_eq!(suspended.product_branch(), branch);
    let completed = reconstruct_ring(&application, suspended, &expected_ring);
    support::host_call(&application, |phase| {
        application.fail_next_durable_append_for_test(
            &phase
                .checked_execution_request_for(application.product_runtime())
                .unwrap(),
        )
    });
    let recovery = match application.restore_generated_output(completed, &scope) {
        Err(RestorationFailure::ProductUnpublished(unpublished)) => unpublished.into_recovery(),
        _ => panic!("actual durable append failure retains restoration recovery"),
    };
    let before = request.retain_current_read().unwrap();
    let failed =
        match application.continue_generated_output_restoration_recovery(recovery, &cancelled) {
            Err(failed) => failed,
            Ok(_) => panic!("cancelled admission cannot advance retained restoration"),
        };
    assert_eq!(
        failed.stage(),
        RestoreStage::ExecutionDenied(Denial::Interrupted(
            WorthQueryManagedComputationInterruption::Cancelled
        ),)
    );
    let recovery = failed.into_recovery();
    assert_eq!(recovery.product_branch(), branch);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        before.selected_commit()
    );
    let restored = application
        .continue_generated_output_restoration_recovery(recovery, &scope)
        .unwrap_or_else(|failed| panic!("same restoration resumes: {:?}", failed.stage()));
    assert_eq!(restored.product_branch(), branch);
    assert_eq!(ring(&request), expected_ring);
    assert_eq!(
        redemanded(&request, &application),
        (Posture::StableReused, 0)
    );
}
