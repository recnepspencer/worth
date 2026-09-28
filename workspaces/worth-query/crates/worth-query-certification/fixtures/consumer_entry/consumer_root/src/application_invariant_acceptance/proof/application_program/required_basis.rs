use std::num::NonZeroUsize;

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationPerformedMutationOutcome, WorthQueryApplicationProgramOutputProgress,
    WorthQueryApplicationRequestExt, WorthQueryOutputDemandControls,
};
use worth_query_topology_entry::{
    PlanarOutputDemand, PlanarOutputToLateFinalConnection, PlanarRead, PlanarSourceAdjustment,
};

use super::super::super::{authentication, installation, seed::length};
use crate::application_invariant_acceptance::proof::settle;
use crate::application_program::ConsumerRequiredSharedRoot;
use crate::{ConsumerProgram, ConsumerSchema};

pub(super) fn joined_required_root_discovers_at_its_own_publication(
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
    .expect("the source owner authenticates");
    let request = world.application.request(&principal, &scope);
    let controls = WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(8_192).unwrap(),
    );
    let initial = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let first = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(initial)
        .idempotency(&10_049)
        .execute_performed::<ConsumerProgram, ConsumerRequiredSharedRoot>(&world.application)
        .expect("the first source publication succeeds");
    let WorthQueryApplicationPerformedMutationOutcome::Performed(first) = first else {
        panic!("the first source publication is fresh")
    };
    let mut first = first
        .start_required_outputs(&request, controls)
        .unwrap_or_else(|failure| panic!("the first root starts: {:?}", failure.denial()));
    let first_settled = settle(|| {
        match first
            .required_output_mut()
            .advance(&request)
            .expect("first program advances")
        {
            WorthQueryApplicationProgramOutputProgress::Pending => None,
            WorthQueryApplicationProgramOutputProgress::Settled(settled) => Some(settled),
        }
    });
    assert_eq!(
        first_settled
            .outputs_for::<ConsumerSchema, PlanarOutputToLateFinalConnection>()
            .map(|(demand, _)| demand.body_key())
            .collect::<Vec<_>>(),
        ["remote-b"]
    );
    let current = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let second = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(3),
        })
        .expect_source(current)
        .idempotency(&10_050)
        .execute_performed::<ConsumerProgram, ConsumerRequiredSharedRoot>(&world.application)
        .expect("the later source publication succeeds");
    let WorthQueryApplicationPerformedMutationOutcome::Performed(second) = second else {
        panic!("the later source publication is fresh")
    };
    let second_commit = second
        .receipt()
        .committed_product_publication()
        .composite_commit()
        .ordinal();
    let mut second = second
        .start_required_outputs(&request, controls)
        .unwrap_or_else(|failure| panic!("unchanged root joins: {:?}", failure.denial()));
    let second_settled = settle(|| {
        match second
            .required_output_mut()
            .advance(&request)
            .expect("later program advances")
        {
            WorthQueryApplicationProgramOutputProgress::Pending => None,
            WorthQueryApplicationProgramOutputProgress::Settled(settled) => Some(settled),
        }
    });
    assert_eq!(
        first_settled
            .root_receipt()
            .expect("the first root retains its commit")
            .committed_product_publication()
            .composite_commit(),
        second_settled
            .root_receipt()
            .expect("the joined root retains its commit")
            .committed_product_publication()
            .composite_commit(),
        "the later publication joins the older completed root output"
    );
    let children = second_settled
        .outputs_for::<ConsumerSchema, PlanarOutputToLateFinalConnection>()
        .collect::<Vec<_>>();
    assert_eq!(children.len(), 1);
    assert_eq!(
        children[0].0.body_key(),
        "sibling-b",
        "dependent discovery reads the later anchor edit"
    );
    assert!(children[0].1.observation().selected_commit().ordinal() >= second_commit);

    let latest = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let third = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(4),
        })
        .expect_source(latest)
        .idempotency(&10_051)
        .execute_performed::<ConsumerProgram, ConsumerRequiredSharedRoot>(&world.application)
        .expect("the recovery source publication succeeds");
    let WorthQueryApplicationPerformedMutationOutcome::Performed(third) = third else {
        panic!("the recovery source publication is fresh")
    };
    let receipt = third.receipt().clone();
    let third_commit = receipt
        .committed_product_publication()
        .composite_commit()
        .ordinal();
    drop(third);
    let mut recovered = request
        .recover_required_outputs::<ConsumerProgram, ConsumerRequiredSharedRoot>(
            &world.application,
            &receipt,
            PlanarOutputDemand::new("remote-b"),
            controls,
        )
        .expect("the exact later source publication recovers");
    let third_settled = settle(|| {
        match recovered
            .advance(&request)
            .expect("recovered program advances")
        {
            WorthQueryApplicationProgramOutputProgress::Pending => None,
            WorthQueryApplicationProgramOutputProgress::Settled(settled) => Some(settled),
        }
    });
    assert_eq!(
        first_settled
            .root_receipt()
            .expect("the first root retains its commit")
            .committed_product_publication()
            .composite_commit(),
        third_settled
            .root_receipt()
            .expect("the recovered root retains its commit")
            .committed_product_publication()
            .composite_commit()
    );
    let recovered_children = third_settled
        .outputs_for::<ConsumerSchema, PlanarOutputToLateFinalConnection>()
        .collect::<Vec<_>>();
    assert_eq!(recovered_children.len(), 1);
    assert_eq!(recovered_children[0].0.body_key(), "sibling-c");
    assert!(
        recovered_children[0]
            .1
            .observation()
            .selected_commit()
            .ordinal()
            >= third_commit
    );
}
