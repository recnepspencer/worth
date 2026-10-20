//! checkpoint reopens ready output without producer contact and recomputes after source change.
use super::*;

#[test]
fn checkpoint_reopens_ready_output_without_producer_contact_and_recomputes_after_source_change() {
    let _guard = checkpoint_recovery_test_guard();
    super::super::producer::reset_provider_contacts();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let settled = settle(&request, &application);
    assert!(settled.work().producer_contact_count() > 0);
    assert!(super::super::producer::provider_contacts() > 0);
    drop(settled);
    drop(principal);
    drop(scope);

    let checkpoint = application
        .with_host_advancement(|host_phase| {
            application
                .capture_application_checkpoint(&host_phase, CapturePolicy::SystemAllocation)
                .expect("the ready output is checkpointable")
        })
        .expect("installed host call opens");
    drop(application);

    super::super::producer::reset_provider_contacts();
    let restored = install(Some(checkpoint));
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    let restored_settlement = settle(&request, &restored);
    assert_eq!(restored_settlement.work().producer_contact_count(), 0);
    assert_eq!(restored_settlement.work().delivery_contact_count(), 0);
    assert_eq!(restored_settlement.work().derived_publication_count(), 0);
    assert_eq!(super::super::producer::provider_contacts(), 0);
    let retained = crate::checkpoint_recovery::support::retain_read(&restored, &request)
        .expect("the recovered root is observable");
    request
        .at(&retained)
        .require_current_program_output(
            &restored_settlement,
            Some((support::currentness_work_limit()).get()),
        )
        .expect("the restored settlement is current at its recovered source");

    let repeated_restored_settlement = settle(&request, &restored);
    assert_eq!(
        repeated_restored_settlement.work().producer_contact_count(),
        0
    );
    assert_eq!(super::super::producer::provider_contacts(), 0);
    drop(repeated_restored_settlement);

    let observed = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the recovered source is readable");
    request
        .advanced()
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(3),
        })
        .expect_source(observed.observed_sources()[0].clone())
        .idempotency(&77_u64)
        .execute_performed::<CheckpointProgram, CheckpointRoot>(
            &restored,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the recovered source accepts a fresh edit");
    let current = crate::checkpoint_recovery::support::retain_read(&restored, &request)
        .expect("the edited root is observable");
    assert!(matches!(
        request.at(&current).require_current_program_output(
            &restored_settlement,
            Some((support::currentness_work_limit()).get()),
        ),
        Err(WorthQueryOutputCurrentnessDenial::Output(denial))
            if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded
    ));
    drop(restored_settlement);

    super::super::producer::reset_provider_contacts();
    let recomputed = settle(&request, &restored);
    assert!(recomputed.work().producer_contact_count() > 0);
    assert!(super::super::producer::provider_contacts() > 0);
}
