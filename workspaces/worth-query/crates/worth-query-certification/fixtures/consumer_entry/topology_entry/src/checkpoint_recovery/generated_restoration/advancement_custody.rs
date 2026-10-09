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
    let policy = support::CHECKPOINT_EXECUTION_POLICY.budget();
    bound(Some(worth_foundational::ExecutionBudget::new(
        NonZeroUsize::MIN,
        if memory {
            0
        } else {
            policy.charged_memory_bytes()
        },
        if memory { policy.work_ceiling() } else { 0 },
    )));
}
fn assert_cause(cause: Denial, memory: bool, placement: Placement) {
    if !memory {
        assert_eq!(cause, Denial::Resource(Resource::WorkExhausted));
    } else if placement == Placement::Serial {
        assert_eq!(cause, Denial::Resource(Resource::PolicyMemoryLimit));
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
            let selected = application
                .on_branch(application.current_world())
                .select()
                .unwrap();
            limit(memory);
            let before = reads();
            let Err(SuspensionFailure::ExecutionDenied(cause)) =
                selected.suspend_current_generated_output::<Final>(&scope, source.clone())
            else {
                panic!("suspension refuses at open")
            };
            assert_cause(cause, memory, placement);
            assert_eq!(reads(), before);
            bound(None);
            let selected = application
                .on_branch(application.current_world())
                .select()
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
            assert_cause(Denial::Resource(resource), memory, placement);
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
