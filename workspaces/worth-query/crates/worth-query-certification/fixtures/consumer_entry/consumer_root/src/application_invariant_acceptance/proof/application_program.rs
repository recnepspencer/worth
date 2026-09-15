use std::num::NonZeroUsize;

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandProgress, WorthQueryApplicationPerformedMutationOutcome,
    WorthQueryApplicationProgramOutputProgress, WorthQueryApplicationRequestExt,
    WorthQueryOutputDemandControls,
};
use worth_query_topology_entry::{
    PlanarFinalBodyOutput, PlanarFinalOutputDemand, PlanarFinalOutputFeature, PlanarOutputDemand,
    PlanarOutputRead, PlanarRead, PlanarSourceAdjustment,
};

use super::super::{authentication, installation, seed::length};
use crate::ConsumerSchema;

mod authority_identity;
mod custody;
mod dependent_recovery;
pub(super) mod lifecycle;
mod owner_demand_boundary;
mod readiness_recovery;
mod receipt_evidence;
mod recovery;
mod recovery_authority;

pub(super) fn caller_disposal_before_progress_recovers(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    recovery::caller_disposal_before_progress_recovers(foreign);
    recovery::changed_root_cannot_adopt_stale_prepared_source(foreign);
    dependent_recovery::caller_disposal_after_root_recovers_dependent(foreign);
}

pub(super) fn resource_denial_preserves_source_and_delivery(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    recovery::resource_denial_preserves_source_and_delivery(foreign);
}

pub(super) fn lifecycle_proofs(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    authority_identity::forged_inventory_is_denied_before_publication(foreign);
    authority_identity::unavailable_inventory_is_denied_before_publication();
    custody::abandoned_and_superseded_preparations_are_bounded(foreign);
    custody::close_before_required_output_start_is_typed(foreign);
    lifecycle::supersession_retires_pending_predecessor(foreign);
    lifecycle::duplicate_retry_does_not_schedule_again(foreign);
    lifecycle::two_forks_preserve_predecessor_output(foreign);
    lifecycle::branch_close_wakes_live_required_output(foreign);
    readiness_recovery::readiness_failure_recovers_exact_pending_output(foreign);
    recovery_authority::foreign_retired_and_superseded_custody_is_denied(foreign);
    recovery_authority::zero_discovery_settles_an_empty_inventory(foreign);
    owner_demand_boundary::raw_owner_guards(foreign);
}

pub(super) fn performed_source_settles_required_output(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    installation::assert_plain_installation_requires_program();
    let world = installation::install(foreign);
    let installed_program = world.application.installed_program();
    assert_eq!(installed_program.connections().len(), 2);
    assert_eq!(installed_program.rules().len(), 2);
    assert!(installed_program.rules().iter().any(|rule| {
        rule.identity() == "PositiveParameterCount"
            && rule.local_owner() == Some("worth.query.certification.parameter-feature.v1")
    }));
    assert!(installed_program
        .rules()
        .iter()
        .any(|rule| { rule.identity() == "PositivePlanarTurn" && rule.local_owner().is_none() }));
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
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramInventory, crate::ConsumerProgramRoot>(&world.application)
        .expect("the source edit reaches publication");
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
        panic!("the fresh source publication must retain performed delivery")
    };
    let mut performed = performed
        .start_required_outputs(&request, controls)
        .unwrap_or_else(|failure| panic!("required outputs start: {:?}", failure.denial()));
    assert!(performed
        .required_output()
        .settled_root_observation()
        .is_none());
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
    let output_observations = original_settlement
        .output_observations::<PlanarFinalOutputFeature, PlanarFinalBodyOutput>()
        .collect::<Vec<_>>();
    assert_eq!(output_observations.len(), 2);
    assert_eq!(
        original_settlement.root_observation().selected_commit(),
        settled.observation().selected_commit()
    );
    assert_eq!(
        original_settlement
            .output_occurrences::<
                PlanarFinalOutputFeature,
                PlanarFinalBodyOutput,
                PlanarFinalOutputDemand,
            >()
            .map(|occurrence| occurrence.demand().body_key())
            .collect::<Vec<_>>(),
        ["anchor-a", "anchor-b"]
    );
    receipt_evidence::assert_exact_outputs(
        &original_settlement,
        performed
            .required_output()
            .settled_root_observation()
            .expect("the completed handle retains the exact root basis"),
    );

    let latest_commit = original_settlement
        .output_observations::<PlanarFinalOutputFeature, PlanarFinalBodyOutput>()
        .map(|observation| observation.selected_commit())
        .max_by_key(|commit| commit.ordinal())
        .expect("the independently enumerated program outputs are non-empty");
    assert_eq!(
        original_settlement.latest_observation().selected_commit(),
        latest_commit,
        "the aggregate retains the latest actual dependent publication"
    );
    assert_eq!(
        request
            .at(original_settlement.latest_observation())
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
            .at(original_settlement.latest_observation())
            .query(PlanarOutputRead {
                body_key: "final:anchor-b".to_owned(),
            })
            .execute()
            .expect("the second distinct dependent output remains readable")
            .rows()[0]
            .value,
        length(3)
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

pub(super) fn foreign_program_is_denied_before_publication(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let source_world = installation::install(foreign);
    let other_world = installation::install(foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(source_world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the source application authenticates its principal");
    let request = source_world.application.request(&principal, &scope);
    let before = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the source occurrence is readable");
    let source = before.observed_sources()[0].clone();
    let denial = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source)
        .idempotency(&10_004)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramInventory, crate::ConsumerProgramRoot>(&other_world.application);
    let Err(denial) = denial else {
        panic!("foreign program meaning must be denied before source publication")
    };
    assert!(format!("{denial:?}").contains("ForeignProgram"));
    let after = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the source remains readable after denial");
    assert_eq!(after.rows()[0].y, length(1));
}

pub(super) fn ordinary_source_publication_cannot_bypass_program(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the program application authenticates its principal");
    let request = world.application.request(&principal, &scope);
    let before = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the source occurrence is readable");
    let source = before.observed_sources()[0].clone();
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source)
        .idempotency(&10_005)
        .execute();
    let Err(denial) = outcome else {
        panic!("the migrated source operation must require its installed program")
    };
    assert_eq!(
        denial.kind(),
        worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenialKind::ApplicationProgramRequired,
    );
    let after = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the denied source remains readable");
    assert_eq!(after.rows()[0].y, length(1));
}
