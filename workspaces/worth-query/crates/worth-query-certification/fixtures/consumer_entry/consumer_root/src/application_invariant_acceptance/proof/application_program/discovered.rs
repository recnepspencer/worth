pub(super) mod publication_basis;
pub(super) mod publication_lifecycle;
pub(super) mod recovery;

use std::num::NonZeroUsize;
use worth_query_consumer_values::{PlanarAdjustment, PlanarOperation};

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationDiscoveredMutationOutcome, WorthQueryApplicationRequestExt,
    WorthQueryDiscoveredProgramOutputProgress, WorthQueryOutputDemandControls,
};
use worth_query_topology_entry::{
    PlanarMutation, PlanarOutputDemand, PlanarOutputRead, PlanarRead, PlanarSourceAdjustment,
};

use super::super::super::{authentication, installation, seed::length};
use crate::{ConsumerDiscoveredProgramRoot, ConsumerProgram, ConsumerSchema};

pub(super) fn performed_source_discovers_required_root(
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
    .expect("the discovered-root application authenticates its principal");
    let request = world.application.request(&principal, &scope);
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the authored source is readable")
        .observed_sources()[0]
        .clone();
    let prior = request
        .retain_read()
        .expect("the prior source remains readable");
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source)
        .idempotency(&10_030)
        .execute_performed_discovered::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            &world.application,
        )
        .expect("the declared discovered root reaches source publication");
    let WorthQueryApplicationDiscoveredMutationOutcome::Performed(performed) = outcome else {
        panic!("the authored edit must retain discovered-root custody")
    };
    let mut started = performed
        .start_required_outputs(
            &request,
            WorthQueryOutputDemandControls::new(
                NonZeroUsize::new(4_096).unwrap(),
                NonZeroUsize::new(8_192).unwrap(),
            ),
        )
        .unwrap_or_else(|failure| {
            panic!("performed discovered roots start: {:?}", failure.denial())
        });
    let settled = loop {
        match started
            .required_output_mut()
            .advance(&request)
            .expect("the discovered roots advance")
        {
            WorthQueryDiscoveredProgramOutputProgress::Pending => {}
            WorthQueryDiscoveredProgramOutputProgress::Settled(settled) => break settled,
        }
    };
    let roots = settled.root_outputs().collect::<Vec<_>>();
    assert_eq!(roots.len(), 3);
    assert_eq!(
        roots
            .iter()
            .map(|(demand, _)| demand.body_key())
            .collect::<Vec<_>>(),
        ["remote-b", "sibling-b", "sibling-c"]
    );
    assert_eq!(settled.output_count(), 6);
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0,
        "settling all discovered roots releases the performed source and observation"
    );
    for (key, expected) in [
        ("sibling-b", length(22)),
        ("sibling-c", length(31)),
        ("remote-b", length(42)),
    ] {
        assert_eq!(
            request
                .at(settled.observation())
                .query(PlanarOutputRead {
                    body_key: key.to_owned()
                })
                .execute()
                .expect("each settled output is readable")
                .rows()[0]
                .value,
            expected
        );
    }
    assert_eq!(
        request
            .at(&prior)
            .query(PlanarRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .expect("the prior authored source remains coherent")
            .rows()[0]
            .y,
        length(1)
    );
}

pub(super) fn isolated_source_settles_without_roots(
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
    .expect("the isolated source has an authenticated caller");
    let request = world.application.request(&principal, &scope);
    let source = request
        .query(PlanarRead {
            body_key: "sibling-a".to_owned(),
        })
        .execute()
        .expect("the isolated source is readable")
        .observed_sources()[0]
        .clone();
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "sibling-a".to_owned(),
            replacement_y: length(22),
        })
        .expect_source(source)
        .idempotency(&10_031)
        .execute_performed_discovered::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            &world.application,
        )
        .expect("the isolated source publication succeeds");
    let WorthQueryApplicationDiscoveredMutationOutcome::Performed(performed) = outcome else {
        panic!("the isolated source edit is fresh")
    };
    let mut started = performed
        .start_required_outputs(
            &request,
            WorthQueryOutputDemandControls::new(
                NonZeroUsize::new(4_096).unwrap(),
                NonZeroUsize::new(8_192).unwrap(),
            ),
        )
        .unwrap_or_else(|failure| panic!("empty discovery starts: {:?}", failure.denial()));
    let WorthQueryDiscoveredProgramOutputProgress::Settled(settled) = started
        .required_output_mut()
        .advance(&request)
        .expect("empty discovery settles immediately")
    else {
        panic!("empty discovery has no output work")
    };
    assert_eq!(settled.root_outputs().count(), 0);
    assert_eq!(settled.output_count(), 0);
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0
    );
    assert_eq!(
        request
            .at(settled.observation())
            .query(PlanarRead {
                body_key: "sibling-a".to_owned(),
            })
            .execute()
            .expect("the isolated source remains readable")
            .rows()[0]
            .y,
        length(22)
    );
}
