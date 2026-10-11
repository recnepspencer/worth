//! The real caller loop refreshes a Ready whose producer changed a gathered fact.
use super::super::region_output::arm_own_write;
use super::*;
use worth_query_host::facade::primary_graph::{
    advancement_requests_on_this_thread_for_test as reports,
    bound_advancement_requests_on_this_thread_for_test as bound,
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryExecutionPlacementForTest as Placement,
};
struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
        arm_own_write(None);
    }
}
#[test]
fn producer_caller_pass_refuses_before_its_source_or_handler_reads() {
    use worth_query_host::facade::{
        application_contribution::{
            WorthQueryAdvancementDenial as Denial,
            WorthQueryManagedComputationResourceDenial as Resource,
            WorthQueryMemoryLimitLevel as Level,
        },
        application_entry::WorthQueryApplicationOutputDemandDenial as OutputDenial,
        primary_graph::installed_source_reads_on_this_thread_for_test as reads,
    };
    let _guard = checkpoint_recovery_test_guard();
    let application = install(|graph| {
        facts::seed_set(graph, "even", -0.0);
        seed_entry(
            graph,
            &["even"],
            0,
            RegionEntry {
                id: 0,
                region: 0,
                value: 1.0,
                work: 1,
                fault: None,
            },
        );
    });
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut interest = request
        .demand(RegionOutputDemand(SCOPE.to_owned()))
        .start_dependent_in_program::<OracleProgram, RegionConnection>(&application)
        .unwrap();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        for zero_memory in [false, true] {
            let policy = support::CHECKPOINT_EXECUTION_POLICY.budget();
            let _restore = Restore(
                place(placement),
                bound(Some(worth_foundational::ExecutionBudget::new(
                    NonZeroUsize::MIN,
                    if zero_memory {
                        0
                    } else {
                        policy.charged_memory_bytes()
                    },
                    if zero_memory {
                        policy.work_ceiling()
                    } else {
                        0
                    },
                ))),
            );
            let before = reads();
            reports();
            take_calls();
            let OutputDenial::Demand(denial) = interest.advance(&request).err().unwrap() else {
                panic!("the caller must retain its opening cause");
            };
            let WorthQueryOutputDemandDenialKind::ExecutionRequest(cause) = denial.kind() else {
                panic!("the caller opens before its source: {denial:?}");
            };
            if !zero_memory {
                assert_eq!(cause, Denial::Resource(Resource::WorkExhausted));
            } else {
                match placement {
                    Placement::Serial => {
                        assert_eq!(cause, Denial::Resource(Resource::PolicyMemoryLimit))
                    }
                    Placement::Leased(_) => {
                        let Denial::Resource(Resource::MemoryLimit {
                            level,
                            requested,
                            admitted,
                        }) = cause
                        else {
                            panic!("lease bytes must be retained");
                        };
                        assert_eq!(level, Level::Policy);
                        assert!(requested > 0);
                        assert_eq!(admitted, 0);
                    }
                    Placement::World | Placement::Certified { .. } => {
                        unreachable!("this probe names its placements")
                    }
                }
            }
            assert_eq!(reads(), before);
            assert_eq!(take_calls(), OwnerCalls::default());
            assert_eq!(reports(), vec![Err(cause)]);
        }
        bound(None);
        let before = reads();
        interest.advance(&request).unwrap();
        assert!(
            reads() > before,
            "admitted caller advance enters its counted source reader"
        );
    }
}
