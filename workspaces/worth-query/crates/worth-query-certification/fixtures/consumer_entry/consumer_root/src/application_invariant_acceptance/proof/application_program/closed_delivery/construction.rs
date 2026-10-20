//! Invalid governed Fixed metadata is refused before minting any effect owner.
use super::*;
use crate::application_program::fixed_artifact_source::MissingLocalityProgram;
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_host::facade::{
    application_entry::WorthQueryOperationConstructionDenial,
    application_installation::WorthQueryApplicationProgramRoster,
    primary_graph::{WorthQueryExecutionObservation, WorthQueryOutputDemandDenialKind},
};

#[test]
fn fixed_constructor_rejects_missing_governed_locality_without_effects() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let program = ApplicationProgramAuthoring::<ConsumerSchema, MissingLocalityProgram>::begin()
        .validated_program()
        .unwrap();
    let mut world = installation::install_selected_program(
        &crate::installed_schema(),
        4096,
        WorthQueryApplicationProgramRoster::new(),
        &[],
        program,
    )
    .unwrap();
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-a".into(),
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&input);
    {
        let scope = authentication::request_scope();
        let adapter = authentication::admit(world.application.installed_schema());
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        let before = request.retain_current_read().unwrap();
        let selected = request
            .query(PlanarRead {
                body_key: input.scope_key.clone(),
            })
            .execute()
            .unwrap();
        let initial = selected.rows().to_vec();
        let description = request
            .mutate(input.clone())
            .expect_source(selected.observed_sources()[0].clone())
            .idempotency_owned(0x9176_6276)
            .unwrap_or_else(|(_, _, stop)| panic!("input capture: {stop:?}"));
        drop(selected);
        let original = match description
            .retain_operation::<MissingLocalityProgram, ConsumerFixedAction>(&world.application)
        {
            Err((original, WorthQueryOperationConstructionDenial::OutputSource(cause))) => {
                assert_eq!(
                    cause.kind(),
                    WorthQueryOutputDemandDenialKind::ForeignDemand
                );
                assert_eq!(
                    cause.subject(),
                    "the performed source action has no locality for its governed artifact",
                    "intended installed-metadata refusal, not a different ForeignDemand"
                );
                original
            }
            _ => panic!("invalid governed source must not mint an effect-capable Fixed owner"),
        };
        assert_eq!(planar_source_adjustment_contacts(&input), (0, 0));
        assert_eq!(
            world
                .application
                .producer_contacts_on_this_thread_for_test(),
            0
        );
        assert_eq!(
            request
                .query(PlanarRead {
                    body_key: input.scope_key.clone()
                })
                .execute()
                .unwrap()
                .rows(),
            initial.as_slice()
        );
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            before.selected_commit()
        );
        drop(original);
    }
    world
        .application
        .close_conditional_runtime()
        .expect("original description and authentic runtime remain releasable");
}
