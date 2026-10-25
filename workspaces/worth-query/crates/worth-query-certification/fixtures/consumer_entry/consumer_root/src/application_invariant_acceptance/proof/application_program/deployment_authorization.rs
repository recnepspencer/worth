//! No grant, or a target-only grant, cannot authorize the governing source.
use super::*;
use crate::application_program::fixed_successors::UnchangedFixed;
use crate::program_deployment::ConsumerDeploymentGrantSeed;
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationProgramAdoptionPreparationDenial, WorthQueryApplicationRequestExt,
    },
    application_installation::{WorthQueryApplicationProgramRoster, WorthQueryProgramOwner},
    primary_graph::{
        WorthQueryOperationAuthorizationDenialKind, WorthQueryOperationAuthorizationRefusalKind,
        WorthQueryProgramAdoptionAuthorizationDenial,
    },
};
#[test]
fn fixed_adoption_requires_an_explicit_source_revision_grant() {
    for grants in [
        vec![],
        vec![ConsumerDeploymentGrantSeed::for_program::<UnchangedFixed>(
            "model-owner",
            "model-owner",
        )],
    ] {
        let world = installation::install_with_roster(
            &crate::installed_schema(),
            WorthQueryApplicationProgramRoster::new().support(
                ApplicationProgramAuthoring::<ConsumerSchema, UnchangedFixed>::begin()
                    .with_schema(
                        &ConsumerSchema::declaration().expect("the completed schema is valid"),
                    )
                    .validated_program()
                    .unwrap(),
            ),
            &grants,
        )
        .unwrap();
        let scope = authentication::request_scope();
        let adapter = authentication::admit(world.application.installed_schema());
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let programs = world.application.request(&principal, &scope).programs();
        let head = || {
            world
                .application
                .with_host_advancement(|phase| {
                    world
                        .application
                        .runtime()
                        .on_branch(world.application.current_world())
                        .select(&phase)
                        .unwrap()
                        .product()
                        .selected_commit()
                        .clone()
                })
                .unwrap()
        };
        let before_head = head();
        let before = programs.inspect().unwrap();
        let target = *world
            .application
            .supported_program::<UnchangedFixed>()
            .unwrap()
            .owned_revision();
        match programs
            .compare(&target)
            .expect_err("absent source deployment grant must refuse compare")
        {
            WorthQueryApplicationProgramAdoptionPreparationDenial::Authorization(
                WorthQueryProgramAdoptionAuthorizationDenial::Authorization(denial),
            ) => assert!(
                matches!(
                    denial.kind(),
                    WorthQueryOperationAuthorizationDenialKind::Refused(
                        WorthQueryOperationAuthorizationRefusalKind::CapabilityGrantMissing
                    )
                ),
                "{denial:?}"
            ),
            other => panic!("the native source capability must decide: {other:?}"),
        }
        assert_eq!(programs.inspect().unwrap().revision(), before.revision());
        assert_eq!(head(), before_head);
        let observed = world
            .application
            .request(&principal, &scope)
            .query(worth_query_topology_entry::PlanarRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .unwrap();
        assert_eq!(
            observed.rows()[0].y,
            crate::application_invariant_acceptance::seed::length(1)
        );
        assert!(world
            .application
            .product_publication_recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
            .unwrap()
            .rows()
            .is_empty());
    }
}
