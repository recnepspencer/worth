//! Stable native completion must wake Pending without a World business write.
use super::*;

#[test]
fn equal_input_completion_without_business_commit_renews_same_consumer_record() {
    let _guard = checkpoint_recovery_test_guard();
    let provider_observation = crate::producer::ProviderContacts::default();
    let mut app = installation::install_observed(&provider_observation);
    let (scope, principal) = authenticate(&app);
    let request = app.request(&principal, &scope);
    races::complete_geometry(&app, &request, false);
    let initial = quantity(&app, &request);
    let initial_read = initial
        .read_current(&request, read::Read(publication::output_key("population")))
        .unwrap();
    assert_eq!(initial_read.rows()[0].count, 1);
    assert_eq!(initial_read.rows()[0].measure, Measure::Complete(11));
    let identity = initial
        .settlement()
        .root_settlement()
        .outputs_of::<binding::InitialOutputs>()
        .unwrap()
        .entity::<binding::Created>()
        .unwrap()
        .entity_id();
    // A live required interest asks later advances to refresh this consumer.
    // Close original native work custody but retain full readonly delivery.
    let initial = initial
        .close_for_delivery(&request)
        .unwrap_or_else(|stop| panic!("original Initial delivery close: {:?}", stop.denial()));

    // This is part of measure-a's actual Query footprint but not its projected y.
    let source = request
        .query(PlanarRead {
            body_key: "measure-b".into(),
        })
        .execute()
        .unwrap();
    request
        .advanced()
        .mutate(PlanarSourceAdjustment {
            scope_key: "measure-b".into(),
            replacement_y: length(11),
        })
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x91742704u64)
        .execute_performed::<Program, GeometryRoot>(
            &app,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(initial
        .read_current(&request, read::Read(publication::output_key("population")))
        .is_err());
    provider_observation.reset();
    let pending = quantity(&app, &request)
        .close_for_delivery(&request)
        .unwrap_or_else(|stop| panic!("original Pending delivery close: {:?}", stop.denial()));
    // This remains a real current original delivery after every work interest
    // is discharged, not a copied observation or an unvalidated historical row.
    let held = pending
        .read_current(&request, read::Read(publication::output_key("population")))
        .unwrap();
    assert_eq!(held.rows()[0].count, 1);
    assert_eq!(held.rows()[0].measure, Measure::Pending);
    assert_eq!(provider_observation.count(), 0);
    let before = request.retain_current_read().unwrap();
    // The actual native owner must accept stable reuse, not force an execution.
    races::complete_geometry(&app, &request, true);
    assert_eq!(
        before.selected_commit(),
        request.retain_current_read().unwrap().selected_commit()
    );
    // complete_geometry requires StableReused, zero native execution contacts
    // and no producer receipt. Canonical-input/resource/key admission callbacks
    // precede that cutoff and remain separately visible provider work.
    assert!(
        pending
            .read_current(&request, read::Read(publication::output_key("population")))
            .is_err(),
        "genuine stable completion invalidates Pending despite no business commit"
    );
    let renewed = quantity(&app, &request);
    let current = renewed
        .read_current(&request, read::Read(publication::output_key("population")))
        .unwrap();
    assert_eq!(current.rows()[0].count, 1);
    assert_eq!(current.rows()[0].measure, Measure::Complete(11));
    assert_eq!(
        current.rows()[0].contributors,
        initial_read.rows()[0].contributors
    );
    assert_eq!(held.rows()[0].measure, Measure::Pending);
    for actual in [
        pending
            .settlement()
            .root_settlement()
            .outputs_of::<binding::PreserveOutputs>()
            .unwrap()
            .entity::<binding::Preserved>()
            .unwrap()
            .entity_id(),
        renewed
            .settlement()
            .root_settlement()
            .outputs_of::<binding::PreserveOutputs>()
            .unwrap()
            .entity::<binding::Preserved>()
            .unwrap()
            .entity_id(),
    ] {
        assert_eq!(identity, actual);
    }
    eprintln!("native availability: real footprint edit -> Count1/Pending/0contacts -> stable completion/no World commit -> same-record Complete11");
    drop((initial_read, held, current, before, source));
    renewed
        .close(&request)
        .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
    drop((initial, pending));
    drop((principal, scope));
    app.close_conditional_runtime().unwrap();
}
