//! supersession retires pending predecessor.
use super::*;

pub(in super::super) fn supersession_retires_pending_predecessor(
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

    let mut predecessor = perform(&request, &world.application, "anchor-a", 2, 10_006);
    // Another caller settles the root, so the predecessor's advance joins it
    // and the delivery it is interrupted at is its dependent's.
    let mut root = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(controls())
        .start_in_program::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(&world.application)
        .expect("another caller demands the root output");
    assert!(matches!(
        root.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
    world
        .application
        .delay_next_output_readiness_delivery_for_test();
    assert!(matches!(
        predecessor
            .required_output_mut()
            .advance(&world.application, &request)
            .unwrap(),
        WorthQueryApplicationProgramOutputProgress::Pending
    ));

    let mut successor = perform(&request, &world.application, "anchor-a", 5, 10_007);
    let stopped = predecessor
        .required_output_mut()
        .advance(&world.application, &request)
        .map(|progress| {
            matches!(
                progress,
                WorthQueryApplicationProgramOutputProgress::Pending
            )
        });
    assert!(
        matches!(
            stopped,
            Err(worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
                WorthQueryApplicationOutputDemandDenial::Superseded,
            ))
        ),
        "the replaced publication's pending output stops superseded: {stopped:?}"
    );
    let settled = settle(&world.application, &mut successor, &request);
    let exact = request
        .at(settled.observation())
        .query(PlanarOutputRead {
            body_key: "final:anchor-a".to_owned(),
        })
        .execute()
        .expect("the successor output remains exactly readable");
    assert_eq!(exact.rows()[0].value, length(7));
    let comparison_work = NonZeroUsize::new(
        world
            .application
            .runtime()
            .output_demand_resource_profile()
            .limits()
            .source_currentness_work(),
    )
    .expect("the installed profile admits positive source currentness work");
    let previous_basis = world
        .application
        .with_host_advancement(|host_phase| request.retain_read(&host_phase).unwrap())
        .expect("installed host call opens");
    request
        .at(&previous_basis)
        .require_current_program_output(&settled, Some((comparison_work).get()))
        .expect("settled output is current at its retained product basis");
    let _changed_source = perform(&request, &world.application, "anchor-a", 6, 10_111);
    let current_basis = world
        .application
        .with_host_advancement(|host_phase| request.retain_read(&host_phase).unwrap())
        .expect("installed host call opens");
    assert!(matches!(
        request.at(&current_basis).require_current_program_output(
            &settled,
            Some((comparison_work).get()),
        ),
        Err(WorthQueryOutputCurrentnessDenial::Output(denial))
            if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded
    ));
    request
        .at(&previous_basis)
        .require_current_program_output(&settled, Some((comparison_work).get()))
        .expect("a later source edit cannot rewrite a retained prior basis");
}
