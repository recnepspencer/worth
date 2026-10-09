//! A refused exact-recovery advance cannot stop another handle's dependency.
use super::*;
use crate::checkpoint_recovery::support::install_program;
use worth_query_consumer_values::{PlanarDerivedOutput, PlanarOperation};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryApplicationProgramOutputProgress,
    WorthQueryOutputDemandControls, WorthQueryRequiredOutputPreparationDenial,
};

#[test]
fn recovery_selected_refusal_leaves_an_ordinary_dependent_live() {
    scenario(false);
}
#[test]
fn recovery_direct_refresh_door_refuses_its_exact_publication() {
    scenario(true);
}
fn scenario(direct: bool) {
    let _guard = checkpoint_recovery_test_guard();
    let application = install_program::<program::ChainProgram>(None, Default::default());
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    // Preserve the execution audience's owner-paired query result for the
    // direct-door probe. This uses the same authenticated query boundary as
    // request.query(), without the publication wrapper's private projection.
    macro_rules! read_source {
        () => {{
            use worth_query_decl::facade::application_query::{
                ApplicationQueryIntent, ApplicationQueryScopeResolution,
            };
            let intent = PlanarRead {
                body_key: "anchor-a".into(),
            };
            let parameters =
                <PlanarRead as ApplicationQueryIntent<CheckpointSchema>>::parameters(&intent);
            let scope_binding =
                <PlanarRead as ApplicationQueryIntent<CheckpointSchema>>::into_scope(intent);
            let runtime = application.runtime();
            let binding = runtime
                .installed_schema()
                .installed_query_binding::<crate::PlanarReadBinding<CheckpointSchema>>()
                .unwrap();
            let limits = runtime.resolve_application_query_limits(binding.limits());
            let selected = runtime
                .on_branch(application.current_world())
                .select()
                .unwrap();
            let actor = selected
                .resolve_authenticated_principal(
                    binding.principal_binding(),
                    &principal,
                    &scope,
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let (field, value) = scope_binding.into_field_parts(actor.principal_identity());
            let entity = selected
                .resolve_entity(
                    field,
                    value,
                    &scope,
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let access =
                primary_graph::WorthQueryApplicationQueryAccessContext::new(&actor, &entity);
            let controls = primary_graph::WorthQueryProductQueryControls::new(
                limits.maximum_results(),
                limits.maximum_work(),
                &scope,
            );
            let plan = selected
                .admit_application_query(binding.query(), &access, parameters, controls)
                .unwrap();
            runtime
                .execute_application_query_one_shot(plan)
                .unwrap()
                .into_admitted_disclosed()
                .into_output_demand_source()
        }};
    }
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    let changed = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".into(),
            replacement_y: length(2),
        })
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x69_c100_u64)
        .execute_performed::<program::ChainProgram, program::ChainRoot>(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = changed else {
        panic!("source write commits")
    };
    let controls = WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4096).unwrap(),
        NonZeroUsize::new(8192).unwrap(),
    );
    let mut output = performed
        .start_required_outputs(&application, &request, controls)
        .unwrap_or_else(|failure| panic!("{:?}", failure.denial()));
    let receipt = output.receipt().clone();
    let retained_source = read_source!();
    application.press_next_ready_read_with_world_snapshots_for_test();
    let stop = (0..64)
        .find_map(
            |_| match output.required_output_mut().advance(&application, &request) {
                Err(stop) => Some(stop),
                Ok(WorthQueryApplicationProgramOutputProgress::Pending) => None,
                Ok(WorthQueryApplicationProgramOutputProgress::Settled(_)) => {
                    panic!("pressure interrupts Ready")
                }
            },
        )
        .expect("the original root reaches Ready");
    assert!(
        matches!(&stop, WorthQueryRequiredOutputPreparationDenial::Demand(
        WorthQueryApplicationOutputDemandDenial::Demand(cause)) if cause.kind() == WorthQueryOutputDemandDenialKind::ProductSelection(
            primary_graph::WorthQueryProductBranchAdmissionDenial::ObservationCapacityExhausted)),
        "{stop:?}"
    );
    drop(output);
    let mut recovered = request
        .recover_required_outputs::<program::ChainProgram, program::ChainRoot>(
            &application,
            &receipt,
            PlanarOutputDemand::new("anchor-a"),
            controls,
        )
        .unwrap();
    let mut dependent = request
        .demand(ChainDemand("anchor-b".into()))
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    let before = settle!(dependent, request).producer_contacts_in_this_demand();
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    let changed = request
        .mutate(PlanarEdit(PlanarMutation {
            scope_key: "anchor-a".into(),
            operation: PlanarOperation::PublishDerivedOutput(PlanarDerivedOutput {
                body_key: "anchor-a".into(),
                value: length(99),
            }),
        }))
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x69_c101_u64)
        .execute_in_program(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(changed.receipt().is_some());
    if direct {
        let current = read_source!();
        let redisclosed = read_source!();
        let stop = application
            .runtime()
            .attempt_recovery_refresh_for_test::<crate::PlanarOutputFamily>(
                retained_source,
                current,
                redisclosed,
                application
                    .runtime()
                    .output_demand_resource_profile()
                    .limits(),
                &receipt,
            )
            .expect_err("the direct refresh door refuses a genuinely admitted Recovery");
        assert_eq!(stop.kind(), WorthQueryOutputDemandDenialKind::Superseded);
    } else {
        let stop = (0..64)
            .find_map(|_| match recovered.advance(&application, &request) {
                Err(stop) => Some(stop),
                Ok(WorthQueryApplicationProgramOutputProgress::Pending) => None,
                Ok(WorthQueryApplicationProgramOutputProgress::Settled(_)) => {
                    panic!("Recovery cannot refresh")
                }
            })
            .expect("selected Ready refuses recovery refresh");
        assert!(
            matches!(
                &stop,
                WorthQueryRequiredOutputPreparationDenial::Demand(
                    WorthQueryApplicationOutputDemandDenial::Superseded
                )
            ),
            "{stop:?}"
        );
    }
    assert_eq!(
        request
            .query(PlanarOutputRead {
                body_key: "anchor-a".into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        length(99),
        "a recovery refusal cannot publish a replacement"
    );
    // The refused Recovery stays live while the independent handle progresses.
    // Exact Ready wins before head_stop (pending_readmission.rs); its Current
    // certification clears the row's stop before the dependent proceeds.
    let settled = settle!(dependent, request);
    assert!(settled.producer_contacts_in_this_demand() > before);
}
