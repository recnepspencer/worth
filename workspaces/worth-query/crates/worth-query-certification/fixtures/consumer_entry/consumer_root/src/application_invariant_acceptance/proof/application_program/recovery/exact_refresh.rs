//! Recovery enters progression with one exact publication and cannot refresh.
use super::super::lifecycle::{controls, perform};
use super::*;
use worth_query_consumer_values::PlanarDerivedOutput;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryRequiredOutputPreparationDenial,
};
use worth_query_host::facade::primary_graph::WorthQueryProductBranchAdmissionRefusalKind as AdmissionRefusal;

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
        .find_map(|_| {
            match output
                .required_output_mut()
                .advance(&world.application, &request)
            {
                Err(stop) => Some(stop),
                Ok(
                    WorthQueryApplicationProgramOutputProgress::Pending
                    | WorthQueryApplicationProgramOutputProgress::Progressed,
                ) => None,
                Ok(WorthQueryApplicationProgramOutputProgress::Settled(_)) => {
                    panic!("Ready read pressure must preserve source custody")
                }
            }
        })
        .expect("the root becomes Ready before its settlement read is stopped");
    assert!(
        matches!(&interrupted,
        WorthQueryRequiredOutputPreparationDenial::Demand(WorthQueryApplicationOutputDemandDenial::Demand(cause))
        if cause.kind() == worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::ProductSelection(
            AdmissionRefusal::ObservationCapacityExhausted)),
        "{interrupted:?}"
    );
    drop(output);
    let mut recovered = request
        .advanced()
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
        .advanced()
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
        .find_map(|_| match recovered.advance(&world.application, &request) {
            Err(stop) => Some(stop),
            Ok(
                WorthQueryApplicationProgramOutputProgress::Pending
                | WorthQueryApplicationProgramOutputProgress::Progressed,
            ) => None,
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
