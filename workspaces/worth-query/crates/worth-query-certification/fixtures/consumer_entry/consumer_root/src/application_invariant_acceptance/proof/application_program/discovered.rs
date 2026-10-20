pub(super) mod continuation;
#[cfg(test)]
mod currentness;
pub(super) mod initial_unpublished;
pub(super) mod publication_basis;
pub(super) mod publication_lifecycle;
pub(super) mod recovery;
pub(super) mod unpublished;

use crate::application_program::discovered_delivery::{
    Action, NoProjectionDelivery, Program, QueryDelivery,
};
use std::num::NonZeroUsize;
use worth_query_consumer_values::{PlanarAdjustment, PlanarOperation};
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_host::facade::application_entry::{
    WorthQueryOperationAdvance, WorthQueryOperationDeliveryPolicy,
};

use worth_query_host::facade::application_entry::WorthQueryApplicationRequestExt;
use worth_query_topology_entry::{
    PlanarEdit, PlanarMutation, PlanarOutputDemand, PlanarOutputRead, PlanarRead,
    PlanarSourceAdjustment,
};

use super::super::super::{authentication, installation, seed::length};
use crate::application_invariant_acceptance::proof::settle;
use crate::{ConsumerDiscoveredProgramRoot, ConsumerProgram, ConsumerSchema};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationDiscoveredMutationOutcome, WorthQueryDiscoveredProgramOutputProgress,
    WorthQueryOutputDemandControls,
};

pub(super) fn performed_source_discovers_required_root(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let program = ApplicationProgramAuthoring::<ConsumerSchema, Program<QueryDelivery>>::begin()
        .validated_program()
        .unwrap();
    let grants = [
        crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<Program<QueryDelivery>>(
            "model-owner",
            "model-owner",
        ),
    ];
    let world = installation::install_selected_program(
        foreign,
        8192,
        4096,
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new(
        ),
        &grants,
        program,
    )
    .unwrap();
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
    let prior = world
        .application
        .with_host_advancement(|host_phase| {
            request
                .retain_read(&host_phase)
                .expect("the prior source remains readable")
        })
        .expect("installed host call opens");
    // compiled-example: begin
    let operation = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source)
        .idempotency_owned(10_030)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<Program<QueryDelivery>, Action<QueryDelivery>>(&world.application)
        .unwrap_or_else(|(_, denial)| panic!("declared discovered construction: {denial:?}"));
    let mut operation = Some(operation);
    let settled = settle(|| match operation.take().unwrap().advance(&request) {
        WorthQueryOperationAdvance::Progress {
            operation: next, ..
        } => {
            operation = Some(next);
            None
        }
        WorthQueryOperationAdvance::Settled(settled) => Some(settled),
        WorthQueryOperationAdvance::Refused { denial, .. }
        | WorthQueryOperationAdvance::Terminal { denial, .. } => {
            panic!("ordinary discovery: {denial:?}")
        }
    });
    assert_eq!(
        settled.read_current(&request).unwrap().rows()[0].value,
        length(42)
    );
    let roots = settled.settlement().root_outputs().collect::<Vec<_>>();
    assert_eq!(roots.len(), 3);
    assert_eq!(
        roots
            .iter()
            .map(|(demand, _)| demand.body_key())
            .collect::<Vec<_>>(),
        ["remote-b", "sibling-b", "sibling-c"]
    );
    assert_eq!(settled.settlement().output_count(), 6);
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
                .at(settled.settlement().observation())
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
    settled
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("lawful discovered close: {denial:?}"));
    // compiled-example: end
}

pub(super) fn isolated_source_settles_without_roots(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let program =
        ApplicationProgramAuthoring::<ConsumerSchema, Program<NoProjectionDelivery>>::begin()
            .validated_program()
            .unwrap();
    let grants = [
        crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<
            Program<NoProjectionDelivery>,
        >("model-owner", "model-owner"),
    ];
    let world = installation::install_selected_program(
        foreign,
        8192,
        4096,
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new(
        ),
        &grants,
        program,
    )
    .unwrap();
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
    let operation = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "sibling-a".to_owned(),
            replacement_y: length(22),
        })
        .expect_source(source)
        .idempotency_owned(10_031)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<Program<NoProjectionDelivery>, Action<NoProjectionDelivery>>(
            &world.application,
        )
        .unwrap_or_else(|(_, denial)| panic!("declared no-projection construction: {denial:?}"));
    let mut operation = Some(operation);
    let settled = settle(|| match operation.take().unwrap().advance(&request) {
        WorthQueryOperationAdvance::Progress {
            operation: next, ..
        } => {
            operation = Some(next);
            None
        }
        WorthQueryOperationAdvance::Settled(settled) => Some(settled),
        WorthQueryOperationAdvance::Refused { denial, .. }
        | WorthQueryOperationAdvance::Terminal { denial, .. } => {
            panic!("ordinary empty discovery: {denial:?}")
        }
    });
    let current = settled
        .deliver_current(&request, WorthQueryOperationDeliveryPolicy::default())
        .unwrap();
    assert_eq!(
        current.selected_commit(),
        settled.settlement().observation().selected_commit()
    );
    assert_eq!(settled.settlement().root_outputs().count(), 0);
    assert_eq!(settled.settlement().output_count(), 0);
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0
    );
    assert_eq!(
        request
            .at(settled.settlement().observation())
            .query(PlanarRead {
                body_key: "sibling-a".to_owned(),
            })
            .execute()
            .expect("the isolated source remains readable")
            .rows()[0]
            .y,
        length(22)
    );
    settled
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("lawful empty close: {denial:?}"));
}

#[cfg(test)]
mod retention;
