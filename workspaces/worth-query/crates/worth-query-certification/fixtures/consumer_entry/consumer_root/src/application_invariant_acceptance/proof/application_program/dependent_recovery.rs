use std::num::NonZeroUsize;

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationPerformedMutationOutcome, WorthQueryApplicationProgramOutputProgress,
    WorthQueryApplicationRequestExt, WorthQueryOutputDemandControls,
};
use worth_query_topology_entry::{
    PlanarFinalBodyOutput, PlanarFinalOutputDemand, PlanarFinalOutputFeature, PlanarOutputRead,
    PlanarRead, PlanarSourceAdjustment,
};

use super::super::super::{authentication, installation, seed::length};
use crate::ConsumerSchema;

pub(super) fn caller_disposal_after_root_recovers_dependent(
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
    let source = request
        .query(PlanarRead {
            body_key: "anchor-c".to_owned(),
        })
        .execute()
        .expect("the source occurrence is readable")
        .observed_sources()[0]
        .clone();
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-c".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source)
        .idempotency(&10_018)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramInventory, crate::ConsumerProgramRoot>(&world.application)
        .expect("the source edit reaches its installed program");
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
        panic!("the source edit must be fresh")
    };
    let controls = WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(8_192).unwrap(),
    );
    let mut started = performed
        .start_required_outputs(&request, controls)
        .unwrap_or_else(|failure| panic!("required outputs start: {:?}", failure.denial()));
    let source_receipt = started.receipt().clone();

    for _ in 0..2 {
        assert!(matches!(
            started.required_output_mut().advance(&request).unwrap(),
            WorthQueryApplicationProgramOutputProgress::Pending
        ));
    }
    assert_eq!(
        request
            .query(PlanarOutputRead {
                body_key: "anchor-c".to_owned(),
            })
            .execute()
            .expect("the root output is already published")
            .rows()[0]
            .value,
        length(3)
    );
    assert!(
        request
            .query(PlanarOutputRead {
                body_key: "final:anchor-c".to_owned(),
            })
            .execute()
            .is_err(),
        "the dependent has not yet published at the interruption point"
    );
    drop(started);

    let mut recovered = request
        .recover_required_outputs::<crate::ConsumerProgram, crate::ConsumerProgramInventory, crate::ConsumerProgramRoot>(
            &world.application,
            &source_receipt,
            controls,
        )
        .expect("fresh caller authority recovers the installed obligation");
    let settled = loop {
        match recovered.advance(&request).unwrap() {
            WorthQueryApplicationProgramOutputProgress::Pending => {}
            WorthQueryApplicationProgramOutputProgress::Settled(settled) => break settled,
        }
    };
    assert_eq!(
        settled
            .output_occurrences::<
                PlanarFinalOutputFeature,
                PlanarFinalBodyOutput,
                PlanarFinalOutputDemand,
            >()
            .map(|(demand, _)| demand.body_key())
            .collect::<Vec<_>>(),
        ["anchor-c", "anchor-a"]
    );
    assert_eq!(
        request
            .at(settled.latest_observation())
            .query(PlanarOutputRead {
                body_key: "final:anchor-c".to_owned(),
            })
            .execute()
            .expect("the recovered dependent output is immediately readable")
            .rows()[0]
            .value,
        length(4)
    );
}
