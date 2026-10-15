//! Descriptor branch and current installed delivery stay exact through a real fork and adoption.
use super::*;
use crate::application_program::discovered_delivery::NoProjectionDelivery;
use worth_query_host::facade::application_entry::{
    WorthQueryOperationCloseDenial, WorthQueryOperationDeliveryDenial,
};
#[test]
fn discovered_ordinary_fork_refuses_foreign_requests_and_current_changed_delivery_preserves_history(
) {
    let original = ApplicationProgramAuthoring::<ConsumerSchema, Program<QueryDelivery>>::begin()
        .validated_program()
        .unwrap();
    let successor =
        ApplicationProgramAuthoring::<ConsumerSchema, Program<NoProjectionDelivery>>::begin()
            .validated_program()
            .unwrap();
    let roster=worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new().support(successor);
    let grants = [
        crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<Program<QueryDelivery>>(
            "model-owner",
            "model-owner",
        ),
        crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<
            Program<NoProjectionDelivery>,
        >("model-owner", "model-owner"),
    ];
    let world = installation::install_selected_program(
        &crate::installed_schema(),
        8192,
        4096,
        roster,
        &grants,
        original,
    )
    .unwrap();
    let runtime = world.application.runtime();
    let root = runtime.current_world();
    let branch = runtime
        .with_host_advancement(|phase| {
            runtime
                .branches()
                .fork(root)
                .components(|c| c.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let adapter = authentication::admit(world.application.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let wrong = world.application.request(&principal, &scope);
    let request = world
        .application
        .request(&principal, &scope)
        .on_branch(branch);
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&input);
    let source = request
        .query(PlanarRead {
            body_key: input.scope_key.clone(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let operation = request
        .mutate(Original(input.clone()))
        .expect_source(source)
        .idempotency_owned(110_051)
        .retain_operation::<Program<QueryDelivery>, Action<QueryDelivery>>(&world.application)
        .unwrap_or_else(|(_, d)| panic!("construction: {d:?}"));
    assert_eq!(operation.branch(), branch);
    let operation = match operation.advance(&wrong) {
        Advance::Refused {
            operation,
            denial: Denial::FreshRequestMismatch,
        } => operation,
        _ => panic!("different default cannot replace original branch"),
    };
    assert_eq!(planar_source_adjustment_contacts(&input), (0, 0));
    let mut operation = Some(operation);
    let settled = (0..32)
        .find_map(|_| match operation.take().unwrap().advance(&request) {
            Advance::Progress {
                operation: next, ..
            } => {
                operation = Some(next);
                None
            }
            Advance::Settled(settled) => Some(settled),
            _ => panic!("original branch progresses"),
        })
        .unwrap();
    assert_eq!(settled.branch(), branch);
    assert!(matches!(
        settled.read_current(&wrong),
        Err(WorthQueryOperationDeliveryDenial::FreshRequestMismatch)
    ));
    let (settled, denial) = settled
        .close(&wrong)
        .err()
        .expect("foreign close retains original settlement");
    assert!(matches!(
        denial,
        WorthQueryOperationCloseDenial::FreshRequestMismatch
    ));
    let before = settled.read_current(&request).unwrap().rows().to_vec();
    let revision = *world
        .application
        .supported_program::<Program<NoProjectionDelivery>>()
        .unwrap()
        .owned_revision();
    let programs = request.programs();
    let requirements = programs.compare(&revision).unwrap();
    let prepared = programs
        .adopt(&requirements)
        .unwrap()
        .prepare(4096)
        .unwrap();
    assert!(matches!(request.programs().publish(prepared),worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPublicationOutcome::Performed(_)));
    assert!(matches!(
        settled.read_current(&request),
        Err(WorthQueryOperationDeliveryDenial::Action(_))
    ));
    assert_eq!(
        settled.read_historical(&request).unwrap().rows(),
        before.as_slice()
    );
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    assert_eq!(
        wrong
            .query(PlanarRead {
                body_key: input.scope_key.clone()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(1)
    );
    settled
        .close(&request)
        .unwrap_or_else(|(_, d)| panic!("historical cleanup remains lawful: {d:?}"));
}
