//! The caller's observation, not the latest one, selects the installed action
//! compared with the closed no-projection owner's original declaration.
use super::*;
use crate::application_program::fixed_successors::{
    ChangedNoProjection, RemovedFixed, UnchangedNoProjection,
};
use worth_query_host::facade::{
    domain::WorthQueryActionResolutionDenial,
    primary_graph::WorthQueryBranchAdoptionPublicationOutcome,
};

#[test]
fn closed_no_projection_compares_the_action_selected_at_the_caller_observation() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let original = ApplicationProgramAuthoring::<ConsumerSchema, NoProjectionProgram>::begin()
        .validated_program()
        .unwrap();
    let original_revision = *original.revision();
    let changed = ApplicationProgramAuthoring::<ConsumerSchema, ChangedNoProjection>::begin()
        .validated_program()
        .unwrap();
    let changed_revision = *changed.revision();
    let removed = ApplicationProgramAuthoring::<ConsumerSchema, RemovedFixed>::begin()
        .validated_program()
        .unwrap();
    let removed_revision = *removed.revision();
    let unchanged = ApplicationProgramAuthoring::<ConsumerSchema, UnchangedNoProjection>::begin()
        .validated_program()
        .unwrap();
    let unchanged_revision = *unchanged.revision();
    let grants = [
        model_owner_grant::<NoProjectionProgram>(),
        model_owner_grant::<ChangedNoProjection>(),
        model_owner_grant::<RemovedFixed>(),
        model_owner_grant::<UnchangedNoProjection>(),
    ];
    let mut world = install_no_projection(
        WorthQueryApplicationProgramRoster::new()
            .support(changed)
            .support(removed)
            .support(unchanged),
        &grants,
    );
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-a".into(),
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&input);
    let (closed_fixed, closed_tree) = {
        let scope = authentication::request_scope();
        let adapter = authentication::admit(world.application.installed_schema());
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        let adopt = |revision| {
            let programs = request.programs();
            let requirements = programs.compare(&revision).unwrap();
            let prepared = programs
                .adopt(&requirements)
                .unwrap()
                .prepare(4096)
                .unwrap();
            assert!(matches!(
                request.programs().publish(prepared),
                WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
            ));
        };
        // Select the identical-meaning program before the original source/effect
        // creates its current basis; delivery still compares the original declared
        // action with this authentic, different selected program.
        adopt(unchanged_revision);
        let fixed = settle_fixed(&world, &request, input.clone(), 0x9176_6282);
        let tree = settle_tree(&world, &request);
        let contacts = world
            .application
            .producer_contacts_on_this_thread_for_test();
        let closed_fixed = fixed
            .close_for_delivery(&request)
            .unwrap_or_else(|(_, stop)| panic!("compatible no-projection close: {stop:?}"));
        let fresh_scope = authentication::request_scope();
        let fresh_principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &fresh_scope,
        ))
        .unwrap();
        let fresh = world.application.request(&fresh_principal, &fresh_scope);
        let compatible = fresh.retain_current_read().unwrap();
        closed_fixed
            .deliver_current_at(&fresh.at(&compatible))
            .expect("compatible selected meaning is accepted at the caller observation");
        drop(compatible);
        // Changed selected meaning cannot strand the tree's original read cleanup.
        adopt(changed_revision);
        let closed_tree = tree.close_for_delivery(&fresh).unwrap_or_else(|stop| {
            panic!(
                "changed selected meaning cannot strand read cleanup: {:?}",
                stop.denial()
            )
        });
        let mismatch = |denied: Result<(), WorthQueryOperationDeliveryDenial>| {
            matches!(
                denied,
                Err(WorthQueryOperationDeliveryDenial::Action(
                    WorthQueryActionResolutionDenial::MeaningMismatch
                ))
            )
        };
        let missing = |denied: Result<(), WorthQueryOperationDeliveryDenial>| {
            matches!(
                denied,
                Err(WorthQueryOperationDeliveryDenial::Action(
                    WorthQueryActionResolutionDenial::Missing
                ))
            )
        };
        // The owner's own observation and a caller-held one agree.
        assert!(mismatch(closed_fixed.deliver_current(&fresh).map(drop)));
        let changed_at = fresh.retain_current_read().unwrap();
        assert!(mismatch(
            closed_fixed.deliver_current_at(&fresh.at(&changed_at))
        ));
        adopt(removed_revision);
        assert!(missing(closed_fixed.deliver_current(&fresh).map(drop)));
        let removed_at = fresh.retain_current_read().unwrap();
        assert!(missing(
            closed_fixed.deliver_current_at(&fresh.at(&removed_at))
        ));
        // The older caller observation still compares its own changed action.
        assert!(mismatch(
            closed_fixed.deliver_current_at(&fresh.at(&changed_at))
        ));
        drop((removed_at, changed_at));
        adopt(original_revision);
        // Restoring the program restores action meaning, not the missing exact
        // native indexed generation. Neither carrier may reconstruct or replay.
        let unavailable = |denied: Result<(), WorthQueryOperationDeliveryDenial>| {
            matches!(
                denied,
                Err(WorthQueryOperationDeliveryDenial::Currentness(
                    WorthQueryOutputCurrentnessDenial::Output(cause)
                )) if cause.kind() == WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable
            )
        };
        assert!(unavailable(closed_fixed.deliver_current(&fresh).map(drop)));
        assert!(unavailable(closed_tree.deliver_current(&fresh).map(drop)));
        let restored = fresh.retain_current_read().unwrap();
        assert!(unavailable(
            closed_fixed.deliver_current_at(&fresh.at(&restored))
        ));
        assert!(unavailable(
            closed_tree.deliver_current_at(&fresh.at(&restored))
        ));
        drop(restored);
        assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
        assert_eq!(
            world
                .application
                .producer_contacts_on_this_thread_for_test(),
            contacts,
            "selection, close and exact-observation gates never re-execute a producer"
        );
        (closed_fixed, closed_tree)
    };
    drop((closed_fixed, closed_tree));
    world
        .application
        .close_conditional_runtime()
        .expect("original native custody closed; readonly keepalives released");
}

#[test]
fn closed_leaf_clean_currentness_uses_native_marks_before_projection() {
    use crate::application_program::fixed_source_receipts::{self as source, Program, Root};
    let _observations = WorthQueryExecutionObservation::install_on_this_thread();
    let program = ApplicationProgramAuthoring::<ConsumerSchema, Program>::begin()
        .validated_program()
        .unwrap();
    let grants = [model_owner_grant::<Program>()];
    let mut world = installation::install_selected_program(
        &crate::installed_schema(),
        WorthQueryApplicationProgramRoster::new(),
        &grants,
        program,
    )
    .unwrap();
    let closed = {
        let scope = authentication::request_scope();
        let adapter = authentication::admit(world.application.installed_schema());
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        source::assert_y(&request, "sibling-a", length(21));
        let mut operation = request
            .demand(PlanarOutputDemand::new("sibling-a"))
            .retain_program_outputs::<Program, Root>(&world.application)
            .unwrap_or_else(|stop| panic!("leaf construction: {:?}", stop.denial()));
        let settled = loop {
            operation = match operation.advance(&request) {
                TreeAdvance::Pending(next) => next,
                TreeAdvance::Settled(settled) => break settled,
                TreeAdvance::Refused { denial, .. } => panic!("leaf: {denial:?}"),
                TreeAdvance::Terminal(terminal) => panic!("leaf terminal: {:?}", terminal.denial()),
            };
        };
        settled
            .close_for_delivery(&request)
            .unwrap_or_else(|stop| panic!("leaf close: {:?}", stop.denial()))
    };
    {
        let scope = authentication::request_scope();
        let adapter = authentication::admit(world.application.installed_schema());
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        let observation = request.retain_current_read().unwrap();

        let _ =
            worth_query_host::facade::primary_graph::fact_verification_on_this_thread_for_test();
        closed
            .deliver_current_at(&request.at(&observation))
            .unwrap();
        let gate =
            worth_query_host::facade::primary_graph::fact_verification_on_this_thread_for_test();
        assert_eq!(
            gate.source_currentness_visits, 0,
            "original closed leaf gate: {gate:?}"
        );
        println!("closed original leaf gate: {gate:?}");

        source::assert_y(&request, "remote-a", length(41));
        for (step, next_value) in [42, 41, 42].into_iter().enumerate() {
            let replacement = length(next_value);
            let before = request
                .query(PlanarRead {
                    body_key: "remote-a".into(),
                })
                .execute()
                .unwrap()
                .rows()[0]
                .y;
            assert_ne!(
                before, replacement,
                "unrelated edit must change Y at step={step}"
            );
            let edit = source::prepare(
                &world.application,
                &request,
                source::input("remote-a", replacement),
                0x9176_6380 + step as u64,
            );
            source::finish(edit, &request)
                .close(&request)
                .unwrap_or_else(|(_, cause)| {
                    panic!("unrelated Fixed close at step={step}: {cause:?}")
                });
            source::assert_y(&request, "remote-a", replacement);
            let selected = request.retain_current_read().unwrap();
            let _ =
                worth_query_host::facade::primary_graph::fact_verification_on_this_thread_for_test(
                );
            for _ in 0..2 {
                closed.deliver_current_at(&request.at(&selected)).unwrap();
            }
            let facts =
                worth_query_host::facade::primary_graph::fact_verification_on_this_thread_for_test(
                );
            assert_eq!(
                facts.source_currentness_visits, 0,
                "closed leaf continuity step={step}: {facts:?}"
            );
        }
        assert_eq!(
            request
                .at(&observation)
                .query(PlanarOutputRead {
                    body_key: "sibling-a".into()
                })
                .execute()
                .unwrap()
                .rows()[0]
                .value,
            length(21 + 1)
        );
    }
    drop(closed);
    world.application.close_conditional_runtime().unwrap();
}
