//! Real discovered roots and recovery share a request across discovery and each demand.
use super::*;
use worth_query_decl::facade::application_program::ApplicationDiscoveredOutputGraph;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Denial,
        WorthQueryManagedComputationResourceDenial as Resource,
        WorthQueryMemoryLimitLevel as Level,
    },
    application_entry::{
        WorthQueryApplicationDiscoveredMutationOutcome as Outcome,
        WorthQueryApplicationOutputDemandDenial as OutputDenial,
        WorthQueryRequiredOutputPreparationDenial as PreparationDenial,
    },
    primary_graph::{
        advancement_requests_on_this_thread_for_test as reports,
        bound_advancement_requests_on_this_thread_for_test as bound,
        installed_source_reads_on_this_thread_for_test as reads,
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryExecutionPlacementForTest as Placement,
    },
};
type DiscoveredRoot = ApplicationDiscoveredOutputGraph<
    RootConnection,
    (
        ApplicationOutputEdge<FinalConnection, ApplicationOutputLeaf>,
        ApplicationOutputEdge<AlternateConnection, ApplicationOutputLeaf>,
    ),
>;
struct DiscoveredProgram;
impl ApplicationProgramDefinition<CheckpointSchema> for DiscoveredProgram {
    type Contributions = <CheckpointSchema as ApplicationSchemaComposition>::Contributions;
    type Outputs = ApplicationProgramOutputs<DiscoveredRoot>;
    type Rules = CheckpointRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("checkpoint-discovered-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        CheckpointProgram::feature_specs()
    }
}
struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}
fn assert_cause(denial: &PreparationDenial, memory: bool, placement: Placement) {
    let PreparationDenial::Demand(OutputDenial::Demand(denial)) = denial else {
        panic!("opening keeps its cause: {denial:?}")
    };
    let primary_graph::WorthQueryOutputDemandDenialKind::ExecutionRequest(cause) = denial.kind()
    else {
        panic!("exact opening cause")
    };
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
            panic!("exact lease bytes")
        };
        assert_eq!(level, Level::Policy);
        assert!(requested > 0);
        assert_eq!(admitted, 0);
    }
}
#[test]
fn discovered_start_and_recovery_open_once_before_their_first_reader() {
    let _guard = checkpoint_recovery_test_guard();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        for memory in [false, true] {
            bound(None);
            let application =
                support::install_program::<DiscoveredProgram>(None, Default::default());
            let (scope, principal) = authenticate(&application);
            let request = application.request(&principal, &scope);
            let read = request
                .query(PlanarRead {
                    body_key: "anchor-a".into(),
                })
                .execute()
                .unwrap();
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
            let before_mutation = reads();
            let cause = request
                .mutate(PlanarSourceAdjustment {
                    scope_key: "anchor-a".into(),
                    replacement_y: length(2),
                })
                .expect_source(read.observed_sources()[0].clone())
                .idempotency(&8122_u64)
                .execute_performed_discovered::<DiscoveredProgram, DiscoveredRoot>(&application)
                .err()
                .unwrap();
            let worth_query_host::facade::application_entry::WorthQueryPerformedMutationExecutionDenial::Mutation(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause)) = cause else { panic!("discovered mutation opens before its handler") };
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
                    panic!("exact bytes")
                };
                assert_eq!(level, Level::Policy);
                assert!(requested > 0);
                assert_eq!(admitted, 0);
            }
            assert_eq!(reads(), before_mutation);
            bound(None);
            let before_mutation = reads();
            let Outcome::Performed(performed) = request
                .mutate(PlanarSourceAdjustment {
                    scope_key: "anchor-a".into(),
                    replacement_y: length(2),
                })
                .expect_source(read.observed_sources()[0].clone())
                .idempotency(&8123_u64)
                .execute_performed_discovered::<DiscoveredProgram, DiscoveredRoot>(&application)
                .unwrap()
            else {
                panic!("source must land with discovered custody")
            };
            assert!(
                reads() > before_mutation,
                "admitted discovered mutation contacts its own reader"
            );
            let receipt = performed.receipt().clone();
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
            reports();
            let before = reads();
            let failure = performed
                .start_required_outputs(&request, Default::default())
                .err()
                .expect("zero budget refuses start");
            assert_cause(failure.denial(), memory, placement);
            assert_eq!(reads(), before);
            assert_eq!(reports().len(), 1);
            let performed = failure.performed();
            let before = reads();
            let denial = request
                .recover_discovered_required_outputs::<DiscoveredProgram, DiscoveredRoot>(
                    &application,
                    &receipt,
                    Default::default(),
                )
                .err()
                .expect("zero budget refuses recovery");
            assert_cause(&denial, memory, placement);
            assert_eq!(reads(), before);
            assert_eq!(reports().len(), 1);
            bound(None);
            let before = reads();
            let started = performed
                .start_required_outputs(&request, Default::default())
                .unwrap_or_else(|_| panic!("admitted discovery starts"));
            assert!(reads() > before);
            assert_eq!(
                reports().len(),
                1,
                "all discovery and demand reads borrow the same request"
            );
            drop(started);
            let before = reads();
            drop(
                request
                    .recover_discovered_required_outputs::<DiscoveredProgram, DiscoveredRoot>(
                        &application,
                        &receipt,
                        Default::default(),
                    )
                    .unwrap(),
            );
            assert!(reads() > before);
            assert_eq!(reports().len(), 1);
        }
    }
}
