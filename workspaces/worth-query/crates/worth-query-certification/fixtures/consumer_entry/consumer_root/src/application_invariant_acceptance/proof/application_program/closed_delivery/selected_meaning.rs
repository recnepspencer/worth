//! Closed delivery uses the authentic selected action; adoption cannot strand cleanup.
use super::*;
use crate::application_program::fixed_successors::{ChangedFixed, RemovedFixed, UnchangedFixed};
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_host::facade::{
    application_installation::WorthQueryApplicationProgramRoster,
    domain::WorthQueryActionResolutionDenial,
    primary_graph::{WorthQueryBranchAdoptionPublicationOutcome, WorthQueryExecutionObservation},
};

#[test]
fn closed_delivery_uses_actual_selected_action_and_root_after_adoption() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let original = ApplicationProgramAuthoring::<ConsumerSchema, ConsumerProgram>::begin()
        .validated_program()
        .unwrap();
    let original_revision = *original.revision();
    let changed = ApplicationProgramAuthoring::<ConsumerSchema, ChangedFixed>::begin()
        .validated_program()
        .unwrap();
    let changed_revision = *changed.revision();
    let removed = ApplicationProgramAuthoring::<ConsumerSchema, RemovedFixed>::begin()
        .validated_program()
        .unwrap();
    let removed_revision = *removed.revision();
    let unchanged = ApplicationProgramAuthoring::<ConsumerSchema, UnchangedFixed>::begin()
        .validated_program()
        .unwrap();
    let unchanged_revision = *unchanged.revision();
    let grants = [
        crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<ConsumerProgram>(
            "model-owner",
            "model-owner",
        ),
        crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<ChangedFixed>(
            "model-owner",
            "model-owner",
        ),
        crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<RemovedFixed>(
            "model-owner",
            "model-owner",
        ),
        crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<UnchangedFixed>(
            "model-owner",
            "model-owner",
        ),
    ];
    let mut world = installation::install_with_roster(
        &crate::installed_schema(),
        8192,
        4096,
        WorthQueryApplicationProgramRoster::new()
            .support(changed)
            .support(removed)
            .support(unchanged),
        &grants,
    )
    .unwrap();
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
        // Matching action meaning does not maintain arbitrary native index bases.
        // Select the compatible program before the original source/effect creates
        // its current basis; final delivery still compares the original declared
        // action with this authentic, different selected program.
        adopt(unchanged_revision);
        let fixed = settle_fixed(&world, &request, input.clone());
        let tree = settle_tree(&world, &request);
        let contacts = world
            .application
            .producer_contacts_on_this_thread_for_test();
        let closed_fixed = fixed
            .close_for_delivery(&request)
            .unwrap_or_else(|(_, stop)| panic!("compatible Fixed cleanup closes: {stop:?}"));
        let fresh_scope = authentication::request_scope();
        let fresh_principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &fresh_scope,
        ))
        .unwrap();
        let fresh = world.application.request(&fresh_principal, &fresh_scope);
        assert_eq!(
            closed_fixed.read_current(&fresh).unwrap().rows()[0].value,
            length(3)
        );
        assert_eq!(
            tree.read_current(
                &fresh,
                PlanarOutputRead {
                    body_key: "anchor-a".into()
                }
            )
            .unwrap()
            .rows()[0]
                .value,
            length(3)
        );
        let policy = super::exact_observation::installed_policy(&world);
        let compatible = fresh.retain_current_read().unwrap();
        closed_fixed
            .deliver_current_at(&fresh.at(&compatible), policy)
            .expect("compatible selected meaning is accepted at the caller observation");
        drop(compatible);
        // Current delivery and native disposal have distinct preconditions.
        // Changed selected meaning cannot strand these original read interests.
        adopt(changed_revision);
        let closed_tree = tree.close_for_delivery(&fresh).unwrap_or_else(|stop| {
            panic!(
                "changed selected meaning cannot strand read cleanup: {:?}",
                stop.denial()
            )
        });
        assert!(matches!(
            closed_fixed.read_current(&fresh),
            Err(WorthQueryOperationDeliveryDenial::Action(
                WorthQueryActionResolutionDenial::MeaningMismatch
            ))
        ));
        // The caller's observation, not the latest one, selects the compared action.
        let changed_at = fresh.retain_current_read().unwrap();
        let mismatch = |denied: Result<(), WorthQueryOperationDeliveryDenial>| {
            matches!(
                denied,
                Err(WorthQueryOperationDeliveryDenial::Action(
                    WorthQueryActionResolutionDenial::MeaningMismatch
                ))
            )
        };
        assert!(mismatch(closed_fixed.deliver_current_at(&fresh.at(&changed_at), policy)));
        adopt(removed_revision);
        assert!(matches!(
            closed_fixed.read_current(&fresh),
            Err(WorthQueryOperationDeliveryDenial::Action(
                WorthQueryActionResolutionDenial::Missing
            ))
        ));
        let removed_at = fresh.retain_current_read().unwrap();
        assert!(matches!(
            closed_fixed.deliver_current_at(&fresh.at(&removed_at), policy),
            Err(WorthQueryOperationDeliveryDenial::Action(
                WorthQueryActionResolutionDenial::Missing
            ))
        ));
        // The older caller observation still compares its own changed action.
        assert!(mismatch(closed_fixed.deliver_current_at(&fresh.at(&changed_at), policy)));
        drop((removed_at, changed_at));
        adopt(original_revision);
        // Restoring the program restores action meaning, not the missing exact
        // native indexed generation. Neither carrier may reconstruct or replay.
        assert!(matches!(
            closed_fixed.read_current(&fresh),
            Err(WorthQueryOperationDeliveryDenial::Currentness(
                WorthQueryOutputCurrentnessDenial::Output(cause)
            )) if cause.kind() == WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable
        ));
        assert!(matches!(
            closed_tree.read_current(
                &fresh,
                PlanarOutputRead {
                    body_key: "anchor-a".into()
                }
            ),
            Err(WorthQueryOperationDeliveryDenial::Currentness(
                WorthQueryOutputCurrentnessDenial::Output(cause)
            )) if cause.kind() == WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable
        ));
        let restored = fresh.retain_current_read().unwrap();
        let unavailable = |denied: Result<(), WorthQueryOperationDeliveryDenial>| {
            matches!(
                denied,
                Err(WorthQueryOperationDeliveryDenial::Currentness(
                    WorthQueryOutputCurrentnessDenial::Output(cause)
                )) if cause.kind() == WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable
            )
        };
        assert!(unavailable(closed_fixed.deliver_current_at(&fresh.at(&restored), policy)));
        assert!(unavailable(closed_tree.deliver_current_at(&fresh.at(&restored), policy)));
        drop(restored);
        assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
        assert_eq!(
            world
                .application
                .producer_contacts_on_this_thread_for_test(),
            contacts,
            "selection, close and final read never re-execute a producer"
        );
        (closed_fixed, closed_tree)
    };
    drop((closed_fixed, closed_tree));
    world
        .application
        .close_conditional_runtime()
        .expect("original native custody closed; readonly keepalives released");
}
