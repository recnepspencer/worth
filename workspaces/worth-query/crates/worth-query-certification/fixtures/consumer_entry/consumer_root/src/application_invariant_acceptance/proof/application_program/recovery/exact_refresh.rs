//! Recovery enters progression with one exact publication and cannot refresh.
use super::super::lifecycle::{controls, perform};
use super::*;
use worth_query_consumer_values::PlanarDerivedOutput;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryRequiredOutputPreparationDenial,
};

#[test]
fn recovery_ready_with_changed_output_refuses_refresh_of_its_exact_publication() {
    let world = installation::install(&crate::installed_schema());
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let mut output = perform(&request, &world.application, "anchor-c", 2, 0x69_3100);
    let receipt = output.receipt().clone();
    // Preserve source custody after this program's root reaches Ready, before
    // its settlement read completes. Its issued commit mode matches recovery.
    world
        .application
        .press_next_ready_read_with_world_snapshots_for_test();
    let interrupted = (0..64)
        .find_map(|_| match output.required_output_mut().advance(&request) {
            Err(stop) => Some(stop),
            Ok(WorthQueryApplicationProgramOutputProgress::Pending) => None,
            Ok(WorthQueryApplicationProgramOutputProgress::Settled(_)) => {
                panic!("Ready read pressure must preserve source custody")
            }
        })
        .expect("the root becomes Ready before its settlement read is stopped");
    assert!(
        matches!(&interrupted,
        WorthQueryRequiredOutputPreparationDenial::Demand(WorthQueryApplicationOutputDemandDenial::Demand(cause))
        if cause.kind() == worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::ProductSelection(
            worth_query_host::facade::primary_graph::WorthQueryProductBranchAdmissionDenial::ObservationCapacityExhausted)),
        "{interrupted:?}"
    );
    drop(output);
    let mut recovered = request
        .recover_required_outputs::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            &world.application,
            &receipt,
            PlanarOutputDemand::new("anchor-c"),
            controls(),
        )
        .expect("Recovery admits the exact, still-current publication");
    let source = request
        .query(PlanarRead {
            body_key: "anchor-c".to_owned(),
        })
        .execute()
        .unwrap();
    let changed = request
        .mutate(PlanarEdit(PlanarMutation {
            scope_key: "anchor-c".to_owned(),
            operation: PlanarOperation::PublishDerivedOutput(PlanarDerivedOutput {
                body_key: "anchor-c".to_owned(),
                value: length(99),
            }),
        }))
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x69_3101_u64)
        .execute_in_program(
            &world.application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("an independent writer changes only the accepted output");
    assert!(
        changed.receipt().is_some(),
        "the output-only mutation commits"
    );
    let stop = (0..64)
        .find_map(|_| match recovered.advance(&request) {
            Err(stop) => Some(stop),
            Ok(WorthQueryApplicationProgramOutputProgress::Pending) => None,
            Ok(WorthQueryApplicationProgramOutputProgress::Settled(_)) => {
                panic!("Recovery cannot move to a refreshed publication")
            }
        })
        .expect("the unavailable Ready reaches progression's refresh refusal");
    assert!(
        matches!(
            &stop,
            WorthQueryRequiredOutputPreparationDenial::Demand(
                WorthQueryApplicationOutputDemandDenial::Superseded
            )
        ),
        "{stop:?}"
    );
    assert_eq!(
        request
            .query(PlanarOutputRead {
                body_key: "anchor-c".to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        length(99),
        "refusal cannot publish a replacement output"
    );
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: "anchor-c".to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(2),
        "only the output changed after exact recovery admission"
    );
}

#[test]
fn consumer_root_recovery_preserves_exact_custody_and_readiness() {
    let installed = crate::installed_schema();
    caller_disposal_before_progress_recovers(&installed);
    snapshot_pressure_preserves_recoverable_source(&installed);
    superseded_completion_is_terminal(&installed);
    changed_root_cannot_adopt_stale_prepared_source(&installed);
    resource_denial_preserves_source_and_delivery(&installed);
    super::super::dependent_recovery::caller_disposal_after_root_recovers_dependent(&installed);
    use super::super::readiness_recovery as ready;
    ready::readiness_failure_recovers_exact_pending_output(&installed);
    ready::ready_read_capacity_preserves_completion(&installed);
    ready::already_committed_replace_reuses_readiness(&installed);
    ready::complete_dependency_aba_advances_the_live_demand(&installed);
    ready::readiness_snapshot_pressure_keeps_published_output_recoverable(&installed);
    ready::published_outputs_hold_no_hidden_read_lease(&installed);
    ready::preserved_noop_output_completes_readiness_without_a_signal_successor(&installed);
    use super::super::discovered::recovery as discovered;
    discovered::newer_discovered_source_retires_recovery(&installed);
    discovered::foreign_runtime_cannot_recover_discovered_source(&installed);
    discovered::interrupted_discovery_recovers_both_consumed_roots(&installed);
    discovered::required_recovery_cannot_claim_discovered_custody(&installed);
}
