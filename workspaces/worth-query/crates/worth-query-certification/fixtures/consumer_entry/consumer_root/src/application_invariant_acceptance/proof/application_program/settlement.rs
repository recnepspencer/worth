use std::num::NonZeroUsize;

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandProgress, WorthQueryApplicationPerformedMutationOutcome,
    WorthQueryApplicationProgramOutputProgress, WorthQueryApplicationRequestExt,
    WorthQueryOutputDemandControls,
};
use worth_query_topology_entry::{
    PlanarAlternateFinalToSummaryConnection, PlanarFinalToSummaryConnection, PlanarOutputDemand,
    PlanarOutputRead, PlanarOutputToAlternateFinalConnection, PlanarOutputToFinalConnection,
    PlanarRead, PlanarSourceAdjustment,
};

use super::super::super::{authentication, installation, seed::length};
use super::{program_contract, recovery};
use crate::ConsumerSchema;
pub(super) fn performed_source_settles_required_output(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    assert_eq!(
        crate::application_program::missing_required_input_is_denied(),
        worth_query_decl::facade::application_program::ApplicationProgramValidationDenialKind::MissingRequiredInput,
    );
    assert_eq!(
        crate::application_program::duplicate_feature_is_denied(),
        worth_query_decl::facade::application_program::ApplicationProgramValidationDenialKind::DuplicateFeature,
    );
    assert_eq!(
        crate::application_program::undeclared_input_is_denied(),
        worth_query_decl::facade::application_program::ApplicationProgramValidationDenialKind::UndeclaredInput,
    );
    assert_eq!(
        crate::application_program::unexported_cross_instance_is_denied(),
        worth_query_decl::facade::application_program::ApplicationProgramValidationDenialKind::UnexportedCrossInstanceConnection,
    );
    installation::assert_program_cannot_omit_an_installed_rule();
    installation::assert_required_output_source_cannot_be_an_action();
    let world = installation::install(foreign);
    let installed_program = world.application.installed_program();
    program_contract::assert_installed(installed_program.connections(), installed_program.rules());
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the program application authenticates its principal");
    let request = world.application.request(&principal, &scope);
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the source occurrence is readable")
        .observed_sources()[0]
        .clone();
    let controls = WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(8_192).unwrap(),
    );
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source)
        .idempotency(&10_001)
        .execute_performed(&world.application)
        .expect("the source edit reaches publication");
    let performed = match outcome {
        WorthQueryApplicationPerformedMutationOutcome::Performed(performed) => performed,
        WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied { denial, .. } => {
            panic!("the fresh source publication denied required output: {denial:?}")
        }
        WorthQueryApplicationPerformedMutationOutcome::NotPerformed(outcome) => {
            panic!("the fresh source publication was not performed: {outcome:?}")
        }
    };
    let mut performed = performed
        .start_required_outputs(&request, controls)
        .unwrap_or_else(|failure| panic!("required outputs start: {:?}", failure.denial()));
    assert!(matches!(
        performed
            .required_output_mut()
            .advance(&request)
            .expect("Signal schedules the connected producer"),
        WorthQueryApplicationProgramOutputProgress::Pending
    ));
    world
        .application
        .delay_next_output_readiness_delivery_for_test();
    assert!(matches!(
        performed
            .required_output_mut()
            .advance(&request)
            .expect("the producer publishes before readiness delivery is interrupted"),
        WorthQueryApplicationProgramOutputProgress::Pending
    ));
    let mut recovered = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(controls)
        .start()
        .expect("a concurrent interest joins the exact pending delivery");
    let settled = loop {
        match recovered
            .advance(&request)
            .expect("the required output advances through the production entry")
        {
            WorthQueryApplicationOutputDemandProgress::Pending => {}
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => break settled,
        }
    };
    let retained = request.at(settled.observation());
    let row = retained
        .query(PlanarOutputRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the exact settled observation remains readable");
    assert_eq!(row.rows()[0].value, length(3));
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .expect("derived publication does not rewrite authored source")
            .rows()[0]
            .y,
        length(2)
    );
    let mut repeated = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(controls)
        .start()
        .expect("unchanged source reuses its exact output demand");
    let WorthQueryApplicationOutputDemandProgress::Settled(repeated_settlement) = repeated
        .advance(&request)
        .expect("unchanged source is already settled")
    else {
        panic!("unchanged source must not schedule another producer effect")
    };
    assert_eq!(
        repeated_settlement.observation().selected_commit(),
        settled.observation().selected_commit()
    );
    let original_settlement = loop {
        match performed
            .required_output_mut()
            .advance(&request)
            .expect("the installed transitive outputs advance")
        {
            WorthQueryApplicationProgramOutputProgress::Pending => {}
            WorthQueryApplicationProgramOutputProgress::Settled(settled) => break settled,
        }
    };
    assert_eq!(
        original_settlement.root_observation().selected_commit(),
        settled.observation().selected_commit()
    );
    assert_eq!(
        original_settlement
            .outputs_for::<ConsumerSchema, PlanarOutputToFinalConnection>()
            .count(),
        1
    );
    assert_eq!(
        original_settlement
            .outputs_for::<ConsumerSchema, PlanarOutputToFinalConnection>()
            .map(|(demand, _)| demand.body_key())
            .collect::<Vec<_>>(),
        ["anchor-a"]
    );
    assert_eq!(
        original_settlement
            .outputs_for::<ConsumerSchema, PlanarOutputToAlternateFinalConnection>()
            .map(|(demand, _)| demand.body_key())
            .collect::<Vec<_>>(),
        ["anchor-b"],
        "the sibling graph branch executes through its typed connection"
    );
    assert_eq!(
        original_settlement
            .outputs_for::<ConsumerSchema, PlanarFinalToSummaryConnection>()
            .count(),
        1,
        "the main branch recursively executes its child edge"
    );
    assert_eq!(
        original_settlement
            .outputs_for::<ConsumerSchema, PlanarAlternateFinalToSummaryConnection>()
            .count(),
        1,
        "the sibling branch recursively executes its child edge"
    );
    let latest_commit = original_settlement
        .outputs_for::<ConsumerSchema, PlanarOutputToFinalConnection>()
        .map(|(_, settlement)| settlement.observation().selected_commit())
        .chain(
            original_settlement
                .outputs_for::<ConsumerSchema, PlanarOutputToAlternateFinalConnection>()
                .map(|(_, settlement)| settlement.observation().selected_commit()),
        )
        .chain(
            original_settlement
                .outputs_for::<ConsumerSchema, PlanarFinalToSummaryConnection>()
                .map(|(_, settlement)| settlement.observation().selected_commit()),
        )
        .chain(
            original_settlement
                .outputs_for::<ConsumerSchema, PlanarAlternateFinalToSummaryConnection>()
                .map(|(_, settlement)| settlement.observation().selected_commit()),
        )
        .chain(std::iter::once(
            original_settlement.root_observation().selected_commit(),
        ))
        .max_by_key(|commit| commit.ordinal())
        .expect("the independently enumerated program outputs are non-empty");
    assert_eq!(
        original_settlement.observation().selected_commit(),
        latest_commit,
        "the aggregate selects the exact newest commit across every settled graph output"
    );
    assert_eq!(
        request
            .at(original_settlement.observation())
            .query(PlanarOutputRead {
                body_key: "final:anchor-a".to_owned(),
            })
            .execute()
            .expect("the final transitive output remains readable")
            .rows()[0]
            .value,
        length(4)
    );
    assert_eq!(
        request
            .at(original_settlement.observation())
            .query(PlanarOutputRead {
                body_key: "final:anchor-b".to_owned(),
            })
            .execute()
            .expect("the second distinct dependent output remains readable")
            .rows()[0]
            .value,
        length(3)
    );
    assert_eq!(
        request
            .at(original_settlement.observation())
            .query(PlanarRead {
                body_key: "final:anchor-a".to_owned(),
            })
            .execute()
            .expect("the main dependent geometry remains readable")
            .rows()[0]
            .y,
        length(4)
    );
    assert_eq!(
        request
            .at(original_settlement.observation())
            .query(PlanarRead {
                body_key: "final:anchor-b".to_owned(),
            })
            .execute()
            .expect("the alternate dependent geometry remains readable")
            .rows()[0]
            .y,
        length(3)
    );
    assert_eq!(
        request
            .at(original_settlement.observation())
            .query(PlanarOutputRead {
                body_key: "final:final:anchor-a".to_owned(),
            })
            .execute()
            .expect("the nested summary output remains readable")
            .rows()[0]
            .value,
        length(6)
    );
    assert_eq!(
        request
            .at(original_settlement.observation())
            .query(PlanarOutputRead {
                body_key: "final:final:anchor-b".to_owned(),
            })
            .execute()
            .expect("the second nested summary output remains readable")
            .rows()[0]
            .value,
        length(5)
    );
    let exact_output = settled.observation().selected_commit().clone();
    drop(original_settlement);
    drop(repeated_settlement);
    drop(repeated);
    drop(settled);
    drop(recovered);
    drop(performed);
    let retired_settlement = recovery::settle_recovered(&request, "anchor-a", controls);
    assert_eq!(
        retired_settlement
            .receipt()
            .committed_product_publication()
            .composite_commit(),
        &exact_output
    );
    assert!(retired_settlement.observation().selected_commit().ordinal() > exact_output.ordinal());
    assert_eq!(
        request
            .at(retired_settlement.observation())
            .query(PlanarOutputRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .expect("the recovered semantic output is present in the current descendant")
            .rows()[0]
            .value,
        length(3)
    );
}
