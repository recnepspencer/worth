//! A saved source survives close before its optional assessment is demanded.
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationOutputDemandDenial,
    WorthQueryApplicationReadObservationDenial, WorthQueryRequiredOutputPreparationDenial,
};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    runtime::ExecutionAllocationPolicy,
};
mod conditional_binding;
mod program;
use program::{DemandProgram, DemandRoot, RequiredProgram, UngovernedProgram};

#[test]
#[allow(
    clippy::result_large_err,
    reason = "The checkpoint boundary preserves its native capture refusal without allocation"
)]
fn demand_only_source_saves_and_reopens_before_explicit_assessment() {
    let _guard = checkpoint_recovery_test_guard();
    super::super::producer::reset_provider_contacts();
    let application = support::install_program::<DemandProgram>(None, Default::default());
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let observed = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    let saved = request
        .advanced()
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(4),
        })
        .expect_source(observed.observed_sources()[0].clone())
        .idempotency(&30_001)
        .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation)
        .expect("a demand-only graph allows its authored source to save without assessment");
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = saved else {
        panic!("the actual source edit must commit freshly")
    };
    assert_eq!(super::super::producer::provider_contacts(), 0);
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(4)
    );
    assert_eq!(
        request
            .query(PlanarOutputRead {
                body_key: "anchor-a".to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        length(2),
        "saving does not update the seeded output value"
    );
    let checkpoint = application
        .with_application_advancement(&scope, |phase| {
            application.capture_application_checkpoint(&phase, CapturePolicy::SystemAllocation)
        })
        .expect("the original request admits native capture")
        .expect("the unassessed source is checkpointable");
    let checkpoint =
        application_installation::WorthQueryApplicationCheckpoint::from_untrusted_bytes(
            checkpoint.bytes().to_vec(),
        );
    drop(receipt);
    drop(observed);
    drop(principal);
    drop(scope);
    drop(application);

    let restored = support::install_program::<DemandProgram>(Some(checkpoint), Default::default());
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(4)
    );
    assert_eq!(
        request
            .query(PlanarOutputRead {
                body_key: "anchor-a".to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        length(2)
    );
    assert_eq!(super::super::producer::provider_contacts(), 0);

    let mut outputs = request
        .advanced()
        .start_program_outputs::<DemandProgram, DemandRoot>(
            &restored,
            PlanarOutputDemand::new("anchor-a"),
            Default::default(),
        )
        .unwrap();
    let cancellation = authentication::WorthQueryCancellationSource::new();
    let cancelled_scope = authentication::WorthQueryRequestScope::new(
        Some(std::time::Instant::now() + std::time::Duration::from_secs(120)),
        cancellation.token(),
    );
    cancellation.cancel();
    let cancelled_request = restored.request(&principal, &cancelled_scope);
    let before = request.retain_current_read().unwrap();
    assert!(matches!(
        cancelled_request.retain_current_read(),
        Err(
            WorthQueryApplicationReadObservationDenial::ExecutionRequest(
                WorthQueryAdvancementDenial::Interrupted(
                    WorthQueryManagedComputationInterruption::Cancelled
                )
            )
        )
    ));
    assert!(matches!(outputs.advance(&restored, &cancelled_request),
        Err(WorthQueryRequiredOutputPreparationDenial::Demand(
            WorthQueryApplicationOutputDemandDenial::Demand(ref denial)))
        if denial.kind() == WorthQueryOutputDemandDenialKind::ExecutionRequest(
            WorthQueryAdvancementDenial::Interrupted(WorthQueryManagedComputationInterruption::Cancelled))));
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        before.selected_commit()
    );
    let settled = (0..256)
        .find_map(|_| match outputs.advance(&restored, &request).unwrap() {
            WorthQueryApplicationProgramOutputProgress::Pending => None,
            WorthQueryApplicationProgramOutputProgress::Settled(settled) => Some(settled),
        })
        .expect("the same owner settles after fresh live request authority");
    assert!(settled.work().producer_contact_count() > 0);
    assert_eq!(
        request
            .at(settled.observation())
            .query(PlanarOutputRead {
                body_key: "anchor-a".to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        length(5)
    );
    assert_eq!(
        request
            .at(settled.observation())
            .query(PlanarOutputRead {
                body_key: "final:anchor-a".to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        length(6)
    );
    let current = request.retain_current_read().unwrap();
    request
        .at(&current)
        .require_current_program_output(&settled, support::currentness_work_limit())
        .unwrap();
    let mut repeated = request
        .advanced()
        .start_program_outputs::<DemandProgram, DemandRoot>(
            &restored,
            PlanarOutputDemand::new("anchor-a"),
            Default::default(),
        )
        .unwrap();
    let repeated = (0..256)
        .find_map(|_| match repeated.advance(&restored, &request).unwrap() {
            WorthQueryApplicationProgramOutputProgress::Pending => None,
            WorthQueryApplicationProgramOutputProgress::Settled(settled) => Some(settled),
        })
        .unwrap();
    // Carried readiness and receipts retain their original work history.
    // Per-handle contacts and the native head distinguish actual fresh effects.
    assert_eq!(repeated.root_producer_contacts_in_this_demand(), 0);
    let finals = repeated
        .outputs_for::<CheckpointSchema, PlanarOutputToFinalConnection>()
        .collect::<Vec<_>>();
    let alternates = repeated
        .outputs_for::<CheckpointSchema, PlanarOutputToAlternateFinalConnection>()
        .collect::<Vec<_>>();
    assert_eq!(finals.len(), 1);
    assert_eq!(alternates.len(), 1);
    assert_eq!(finals[0].1.producer_contacts_in_this_demand(), 0);
    assert_eq!(alternates[0].1.producer_contacts_in_this_demand(), 0);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        current.selected_commit()
    );
}

#[test]
fn required_graph_still_refuses_source_only_save() {
    let _guard = checkpoint_recovery_test_guard();
    let application = support::install_program::<RequiredProgram>(None, Default::default());
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let before = request.retain_current_read().unwrap();
    let observed = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    let denial = request
        .advanced()
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(4),
        })
        .expect_source(observed.observed_sources()[0].clone())
        .idempotency(&30_002)
        .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation)
        .expect("native proposal validation returns its actual commit refusal");
    assert!(matches!(
        denial,
        WorthQueryApplicationMutationOutcome::Commit(
            primary_graph::WorthQueryApplicationUncommitted::Denied(ref cause)
        ) if cause.kind() == primary_graph::WorthQueryApplicationCommitDenialKind::ApplicationProgramRequired
    ));
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        before.selected_commit()
    );
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(1)
    );
}

#[test]
fn demand_only_root_keeps_governed_source_locality_fence() {
    let _guard = checkpoint_recovery_test_guard();
    let application = support::install_program::<UngovernedProgram>(None, Default::default());
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let before = request.retain_current_read().unwrap();
    super::super::producer::reset_provider_contacts();
    let refusal = match request
        .advanced()
        .start_program_outputs::<UngovernedProgram, DemandRoot>(
            &application,
            PlanarOutputDemand::new("anchor-a"),
            Default::default(),
        ) {
        Err(refusal) => refusal,
        Ok(_) => panic!("explicit demand cannot bypass its governed source action"),
    };
    assert!(
        matches!(refusal, WorthQueryRequiredOutputPreparationDenial::Demand(
        WorthQueryApplicationOutputDemandDenial::Demand(ref denial))
        if denial.kind() == WorthQueryOutputDemandDenialKind::ForeignDemand
            && denial.subject() == "the performed source action has no locality for its governed artifact")
    );
    assert_eq!(super::super::producer::provider_contacts(), 0);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        before.selected_commit()
    );
    let direct = match request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .start_in_program::<UngovernedProgram, DemandRoot>(&application)
    {
        Err(refusal) => refusal,
        Ok(_) => panic!("direct selected demand must keep the same governance fence"),
    };
    assert!(
        matches!(direct, WorthQueryApplicationOutputDemandDenial::Demand(ref denial)
        if denial.kind() == WorthQueryOutputDemandDenialKind::ForeignDemand)
    );
    assert_eq!(super::super::producer::provider_contacts(), 0);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        before.selected_commit()
    );
}
