//! The retained currentness public call owns its validation request.
use super::*;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Denial,
        WorthQueryManagedComputationResourceDenial as Resource,
        WorthQueryMemoryLimitLevel as Level,
    },
    primary_graph::{
        bound_advancement_requests_on_this_thread_for_test as bound,
        installed_source_reads_on_this_thread_for_test as reads,
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryExecutionPlacementForTest as Placement,
    },
};
struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}

#[test]
fn retained_currentness_refuses_before_its_admitted_validation_reader() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let settlement = settle(&request, &application);
    let retained =
        crate::checkpoint_recovery::support::retain_read(&application, &request).unwrap();
    let current = request.at(&retained);
    let policy = support::observation::CHECKPOINT_EXECUTION_POLICY.accounting().expect("this fixture declares accounting");
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        for zero_memory in [false, true] {
            bound(None);
            let before = reads();
            current.require_current_program_output(&settlement).unwrap();
            assert!(
                reads() > before,
                "the same admitted entry reaches its source reader"
            );
            bound(Some(worth_foundational::ExecutionBudget::new(
                NonZeroUsize::MIN,
                if zero_memory {
                    Some(0)
                } else {
                    policy.charged_memory_bytes()
                },
                if zero_memory {
                    policy.work_ceiling()
                } else {
                    Some(0)
                },
            )));
            let before = reads();
            let WorthQueryOutputCurrentnessDenial::ExecutionRequest(cause) = current
                .require_current_program_output(&settlement)
                .unwrap_err()
            else {
                panic!("the opening refusal retains its own spelling")
            };
            match (placement, zero_memory) {
                (_, false) => assert_eq!(cause, Denial::Resource(Resource::WorkExhausted)),
                (Placement::Serial | Placement::Leased(_), true) => {
                    let Denial::Resource(Resource::MemoryLimit {
                        level,
                        requested,
                        admitted,
                    }) = cause
                    else {
                        panic!("the opening refusal names its memory limit: {cause:?}");
                    };
                    assert_eq!(level, Level::Policy);
                    assert_eq!(admitted, 0);
                    assert!(requested > 0);
                }
                (Placement::World | Placement::Certified { .. }, true) => {
                    panic!("the custody matrix selects serial and leased placement explicitly")
                }
            }
            assert_eq!(
                reads(),
                before,
                "no retained validation read precedes the opener"
            );
        }
    }
}

// Custody probes share this module's existing observer boundary.
