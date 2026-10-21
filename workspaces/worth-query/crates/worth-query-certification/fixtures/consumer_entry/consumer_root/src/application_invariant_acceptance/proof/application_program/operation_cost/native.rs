//! Correctly coordinated current native counterpart; no owner or outcome is fabricated.
use super::*;
use journey::Sample;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
pub(super) fn run(fanout: usize, unrelated: usize) -> Sample {
    let runtime = installation::install(fanout, unrelated);
    let adapter = authentication::admit(runtime.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = runtime.request(&principal, &scope);
    let population = population::retain_unrelated(&runtime, &request, unrelated);
    let before = measurement::retained(&runtime);
    let allocation = measurement::native_allocation();
    reset_planar_source_adjustment_contacts(&input("focus-a"));
    let measuring = measurement::Measure::begin(&runtime);
    let source = request
        .query(PlanarRead {
            body_key: "focus-a".to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let outcome = request
        .advanced()
        .mutate(Original(input("focus-a")))
        .expect_source(source)
        .preconditions(TypedMutationPreconditions::new().expect_fact(
            BodyKey::reference::<Schema>(),
            ApplicationEncodedScalarValue::try_new("focus-a".to_owned()).unwrap(),
        ))
        .idempotency(&221_001)
        .execute_performed_discovered::<Program, FanoutRoot>(
            &runtime,
            ExecutionAllocationPolicy::Execution(&allocation),
        )
        .unwrap();
    let WorthQueryApplicationDiscoveredMutationOutcome::Performed(performed) = outcome else {
        panic!("unarmed native counterpart must perform genuine source")
    };
    let mut started = performed
        .start_required_outputs(
            &runtime,
            &request,
            WorthQueryOutputDemandControls::host_policy(),
        )
        .unwrap_or_else(|failure| panic!("native start: {:?}", failure.denial()));
    let settlement = crate::application_invariant_acceptance::proof::settle(|| {
        match started
            .required_output_mut()
            .advance(&runtime, &request)
            .unwrap()
        {
            WorthQueryDiscoveredProgramOutputProgress::Pending
            | WorthQueryDiscoveredProgramOutputProgress::Progressed => None,
            WorthQueryDiscoveredProgramOutputProgress::Settled(settlement) => Some(settlement),
        }
    });
    assert_eq!(settlement.root_outputs().count(), fanout);
    assert_eq!(settlement.output_count(), fanout * 2);
    let native = settlement.work();
    drop(started); // The native continuation is finished; retain actual settlement evidence.
    let traversal = measuring.finish(&runtime);
    let held = measurement::retained(&runtime);
    let delivering = measurement::Measure::begin(&runtime);
    let current = request.retain_current_read().unwrap();
    request
        .at(&current)
        .require_current_discovered_program_outputs(&settlement)
        .unwrap();
    let delivered = request
        .at(&current)
        .query(PlanarOutputRead {
            body_key: population::target("focus-a", 0),
        })
        .execute()
        .unwrap();
    let query_work = delivered.receipt().inspect().read_work().total_work_units();
    let delivery = delivering.finish(&runtime);
    assert_eq!(delivered.rows()[0].value, length(2));
    let observed = journey::values(&request, fanout);
    assert_eq!(planar_source_adjustment_contacts(&input("focus-a")), (1, 1));
    let before_close = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    drop((current, delivered));
    // Fresh unarmed source/producer execution finalized its managed runs already.
    // The same existing cleanup collection as ordinary close therefore has no
    // failed run/recovery sidecar. Actually delegate its native release, never
    // relabel dropping settlement as a successful close receipt.
    let mut collection = WorthQueryOperationCleanup::default();
    let closing = measurement::Measure::begin(&runtime);
    let released = runtime
        .runtime()
        .with_application_advancement(&scope, |phase| {
            collection.close_in_advancement(&phase, runtime.runtime())
        })
        .unwrap()
        .unwrap();
    drop(settlement); // Local settled references release after actual collection discharge.
    let cleanup = closing.finish(&runtime);
    assert!(released.managed_cleanup_receipt().is_none());
    assert_eq!(released.retired_component_count(), 0);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &before_close
    );
    let work = measurement::Stages {
        traversal,
        delivery,
        cleanup,
    };
    for owner in population {
        owner
            .close(&request)
            .unwrap_or_else(|(_, d)| panic!("population close: {d:?}"));
    }
    Sample {
        native,
        work,
        population: before,
        retained: held,
        values: observed,
        query_work,
    }
}
