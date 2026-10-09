//! Output settlement funds delivery from the same request that ran its producer.
use super::*;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Denial,
        WorthQueryManagedComputationResourceDenial as Resource,
    },
    application_entry::WorthQueryApplicationOutputDemandDenial as OutputDenial,
    primary_graph::signal_request_charges_on_this_thread_for_test as signal_charges,
};

#[test]
fn producer_completes_but_delivery_cannot_reset_its_remaining_budget() {
    let _guard = checkpoint_recovery_test_guard();
    let _restore = Restore(place(Placement::Serial), bound(None));
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        place(placement);
        bound(None);
        let application = installation::install_configured::<false, PRODUCER_WORK, RUNS>(
            None,
            Default::default(),
            |graph| {
                facts::seed_set(graph, "even", -0.0);
                seed_entry(
                    graph,
                    &["even"],
                    0,
                    RegionEntry {
                        id: 0,
                        region: 0,
                        value: 1.0,
                        work: ENTRY_WORK,
                        fault: None,
                    },
                );
            },
        );
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let mut interest = request.demand(RegionOutputDemand(SCOPE.to_owned()))
            .start_dependent_in_program::<OracleProgram<false, PRODUCER_WORK, RUNS>, RegionConnection>(&application)
            .unwrap();
        // The producer's real own-write invalidates the gathered readiness facts,
        // so settlement must evaluate Signal again after that producer completed.
        arm_own_write(Some(OwnWrite {
            number: 0,
            bits: 2.5_f64.to_bits(),
            read: OwnWriteRead::Blind,
        }));
        let policy = support::CHECKPOINT_EXECUTION_POLICY.budget();
        bound(Some(worth_foundational::ExecutionBudget::new(
            NonZeroUsize::MIN,
            policy.charged_memory_bytes(),
            CEILING,
        )));
        reports();
        signal_charges();
        kernel_charges();
        room().clear();
        let stopped = interest.settle(&request);
        let OutputDenial::Demand(denial) = stopped.err().expect("delivery must refuse") else {
            panic!("the output consumer retains the delivery cause");
        };
        assert_eq!(
            denial.kind(),
            WorthQueryOutputDemandDenialKind::ExecutionRequest(Denial::Resource(
                Resource::WorkExhausted
            ),)
        );
        assert_eq!(
            room().len(),
            REQUIRED_PRODUCER_PASSES as usize,
            "both real producers completed before delivery refused"
        );
        assert!(
            room().iter().all(|run| run.outcome.is_ok()),
            "the declared producer allowances cover both handlers"
        );
        let producer_charges = kernel_charges();
        assert_eq!(
            producer_charges,
            vec![ENTRY_WORK as u64 + EVEN_WEIGHT_WORK; REQUIRED_PRODUCER_PASSES as usize]
        );
        let requests = reports();
        assert_eq!(requests.len(), 1);
        let charged = requests[0].as_ref().unwrap().charged_work();
        let signal_work = signal_charges().iter().sum::<u64>();
        assert!(
            signal_work > 0,
            "real delivery reserves Signal work after producing"
        );
        // Entry checkpoints give a lower bound, not the whole computation charge:
        // map preparation and completion retain their execution-owner accounting.
        assert!(
            charged >= producer_charges.iter().sum::<u64>() + signal_work,
            "the parent retains both producer checkpoints and accepted delivery work"
        );
        assert!(
            charged <= CEILING,
            "the refused reservation cannot overdraw its request"
        );
        // Err delivers no settlement. A delivery refusal retains its owner's
        // terminal posture; this probe does not promise a fresh retry of it.
    }
}
