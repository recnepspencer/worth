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
    // Another caller settles the root. The predecessor first retains that
    // exact settlement, before any independent dependent discovery.
    let mut root = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(controls())
        .start_in_program::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(&world.application)
        .expect("another caller demands the root output");
    let root_commit = match root.advance(&request).unwrap() {
        WorthQueryApplicationOutputDemandProgress::Settled(settlement) => {
            settlement.observation().selected_commit().clone()
        }
        _ => panic!("the independent caller settles the original root"),
    };
    let branch = predecessor.receipt().product_branch();
    {
        let fresh_scope = authentication::request_scope();
        let fresh_principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &fresh_scope,
        ))
        .unwrap();
        let fresh = world
            .application
            .request(&fresh_principal, &fresh_scope)
            .on_branch(branch);
        assert!(matches!(
            predecessor
                .required_output_mut()
                .advance(&world.application, &fresh)
                .unwrap(),
            WorthQueryApplicationProgramOutputProgress::Progressed
        ));
        assert_eq!(
            predecessor
                .required_output_mut()
                .settled_root_observation()
                .expect("the handoff retains the actual joined root settlement")
                .selected_commit(),
            &root_commit,
        );
    }
    // A sibling can make legitimate progress while another delivery waits.
    // Keep the native delivery pause armed across those handoffs; only actual
    // Pending establishes the interruption point needed by this proof.
    loop {
        let fresh_scope = authentication::request_scope();
        let fresh_principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &fresh_scope,
        ))
        .unwrap();
        let fresh = world
            .application
            .request(&fresh_principal, &fresh_scope)
            .on_branch(branch);
        world
            .application
            .delay_next_output_readiness_delivery_for_test();
        match predecessor
            .required_output_mut()
            .advance(&world.application, &fresh)
            .expect("the original dependent advances to its deferred native delivery")
        {
            WorthQueryApplicationProgramOutputProgress::Progressed => {}
            WorthQueryApplicationProgramOutputProgress::Pending => break,
            WorthQueryApplicationProgramOutputProgress::Settled(_) => {
                panic!("deferred native delivery cannot settle the predecessor tree");
            }
        }
    }
    assert_eq!(
        request
            .query(PlanarOutputRead {
                body_key: "final:anchor-a".to_owned()
            })
            .execute()
            .expect("the paused dependent has genuinely published")
            .rows()[0]
            .value,
        length(4),
        "supersession must interrupt dependent delivery after the original root settled",
    );

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

    let previous_basis = world
        .application
        .with_host_advancement(|host_phase| request.retain_read(&host_phase).unwrap())
        .expect("installed host call opens");
    request
        .at(&previous_basis)
        .require_current_program_output(&settled)
        .expect("settled output is current at its retained product basis");
    let _changed_source = perform(&request, &world.application, "anchor-a", 6, 10_111);
    let current_basis = world
        .application
        .with_host_advancement(|host_phase| request.retain_read(&host_phase).unwrap())
        .expect("installed host call opens");
    assert!(matches!(
        request.at(&current_basis).require_current_program_output(
            &settled),
        Err(WorthQueryOutputCurrentnessDenial::Output(denial))
            if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded
    ));
    request
        .at(&previous_basis)
        .require_current_program_output(&settled)
        .expect("a later source edit cannot rewrite a retained prior basis");
}
