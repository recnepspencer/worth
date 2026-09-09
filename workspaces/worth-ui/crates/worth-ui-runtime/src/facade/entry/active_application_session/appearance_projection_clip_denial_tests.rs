use super::support;
use crate::runtime::tests::source_ingress_boundary_test_support::lower_rust_submission;

#[path = "appearance_projection_clip_denial_tests/clip_declaration_test_support.rs"]
mod clip_declaration;
use clip_declaration::clip_declaration;

#[test]
fn current_scroll_topology_is_not_waived_by_a_portal_child_requirement() {
    use worth_ui_dsl::{
        UiDslSemanticArtifactSpec, UiDslSemanticFamily, UiDslSemanticKey, UiDslSourceProvenance,
        UiDslStructuralToken,
    };
    let (_, _, world_profile) = crate::evidence::measurement::projection::fact_test_support::
        display_field_projection_context("appearance-scroll-ancestry");
    let mut session = crate::facade::WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .with_graph_world_profile(world_profile)
        .with_rust_authored_declaration_fixture(
            crate::facade::WorthUiRustAuthoredDeclarationFixture::named(
                "appearance-scroll-ancestry",
            )
            .with_semantic_artifact_spec(
                UiDslSemanticArtifactSpec::new(
                    UiDslSemanticKey::new("appearance.scroll"),
                    UiDslSemanticFamily::Control,
                    UiDslSourceProvenance::rust_authored("appearance/scroll", 0),
                )
                .with_structural_token(UiDslStructuralToken::new("control:primary"))
                .with_structural_token(UiDslStructuralToken::new("operator:scroll")),
            ),
        )
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .unwrap()
        .launch()
        .unwrap();
    let scroll = session
        .graph()
        .snapshot()
        .nodes()
        .iter()
        .find(|node| {
            node.operator_kind() == crate::declaration::UiDeclarationPlanningOperatorKind::Scroll
        })
        .expect("the declaration must really classify as Scroll")
        .graph_node_identity();
    let observed = {
        let turn = session
            .execute_framework_turn(|_| {})
            .unwrap()
            .into_execution()
            .unwrap_or_else(|_| panic!("the current plan must enter execution"));
        // Real graph and its current executed plan; Portal membership is the
        // explicit function-boundary input, not a claim of authored Portal launch.
        crate::mounting::derive_unbound_ancestry(
            turn.graph,
            crate::mounting::UiMountedPlanProjectionSource::Executed(
                turn.execution.runtime.active.active_plan_ref(),
            ),
            scroll,
            true,
            &[],
            Err(scroll),
        )
        .unwrap()
    };
    let (clip, work) = observed;
    assert_eq!(
        clip,
        crate::mounting::UiMountedAppearanceClip::Unresolved(
            crate::mounting::UiMountedAppearanceClipDenial::ScrollBindingUnavailable(scroll),
        )
    );
    assert_eq!(
        work, 5,
        "initial classification plus graph and topology reads for the Scroll node and PageRoot"
    );
    let _ = session.shutdown();
}

#[test]
fn unrelated_mosaic_mount_replacement_does_not_revoke_a_retained_consumer() {
    let role = support::validation_background_role(support::APPEARANCE_TOKEN);
    let builder = || {
        support::legacy_static_paint_appearance_component_builder(&role)
            .register_surface(crate::capability::SurfaceDescriptor::new(
                crate::capability::SurfaceId::new("appearance.mosaic.surface").unwrap(),
                crate::capability::SurfaceKind::primary_content(),
                crate::capability::ComponentId::new(support::APPEARANCE_NODE_B).unwrap(),
                crate::capability::SurfacePlacementClass::primary_region(),
                crate::capability::SurfaceStateClass::restorable(),
            ))
            .register_appearance_theme_bundle(super::test_support::theme_bundle())
            .unwrap()
    };
    let capabilities = builder()
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("clip fixture capabilities must prepare");
    let declaration = clip_declaration(&role, false);
    let submission = lower_rust_submission(
        crate::runtime::WorthUiSourceProvider::rust_authored("appearance-clip-binding")
            .with_rust_authored_input(declaration.clone()),
        [crate::runtime::WorthUiWatcherEvent::provider_revision(
            "appearance-clip-binding",
        )],
        capabilities.capabilities(),
    );
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    host.set_capabilities(worth_ui_host_native::staged_appearance_capability_report());
    let observer = host.clone();
    let mut session = builder()
        .with_candidate_submission(submission)
        .freeze()
        .map(|application| {
            crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
                application,
                host,
            )
        })
        .expect("Mosaic and independent appearance consumer must prepare")
        .launch()
        .expect("Mosaic source must launch through the real plan");
    let (_, target) = super::mounting_fixture::mount(&mut session, 1_000);
    let graph = session.graph();
    let topology = graph.lookup().topology_node(target).unwrap();
    let parent = topology.value().parent_node_identity().unwrap();
    assert_eq!(
        graph
            .lookup()
            .graph_node(parent)
            .unwrap()
            .value()
            .operator_kind(),
        crate::declaration::UiDeclarationPlanningOperatorKind::PageRoot,
        "the tested consumer is outside the separate Mosaic owner's ancestry"
    );
    publish_observation(&mut session, &observer, declaration, 1, false);
    let initial = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    assert!(!initial.fragments().is_empty());
    let worth_ui_host_contract::UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
        successor: Some(receipt),
        ..
    } = initial.fragments()[0].identity()
    else {
        panic!("initial appearance must bind a mounted node receipt");
    };
    let instance = receipt.mounted_instance();
    let incarnation = session
        .mounted
        .current_mounted_identity_basis(instance)
        .unwrap()
        .mount_incarnation();
    for (mounted, now) in [(true, 2), (false, 4)] {
        let declaration = clip_declaration(&role, mounted);
        let submission = lower_rust_submission(
            crate::runtime::WorthUiSourceProvider::rust_authored("appearance-clip-replacement")
                .with_rust_authored_input(declaration.clone()),
            [crate::runtime::WorthUiWatcherEvent::provider_revision(
                "appearance-clip-replacement",
            )],
            session.capabilities(),
        );
        let generation = session.active_generation_identity();
        super::succession_tests::replace_source(&mut session, &observer, submission, now, None);
        assert_ne!(session.active_generation_identity(), generation);
        assert_eq!(
            session
                .mounted
                .current_mounted_identity_basis(instance)
                .unwrap()
                .mount_incarnation(),
            incarnation
        );
        let accepted_frame = session.mounted.current_publication().unwrap().frame();
        publish_observation(&mut session, &observer, declaration, now + 1, false);
        assert_ne!(
            session.mounted.current_publication().unwrap().frame(),
            accepted_frame,
            "a Mosaic mount outside the consumer's occurrence neighborhood must not revoke it"
        );
    }
    let _ = session.shutdown();
}

fn publish_observation(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    observer: &crate::certification_support::ScriptedPresentationHost,
    declaration: worth_ui_dsl::WorthUiRustAuthoredArtifactInput,
    now: u64,
    expect_appearance_denial: bool,
) {
    let source_name = format!("appearance-clip-close-{now}");
    let source = lower_rust_submission(
        crate::runtime::WorthUiSourceProvider::rust_authored(&source_name)
            .with_rust_authored_input(declaration),
        [crate::runtime::WorthUiWatcherEvent::provider_revision(
            &source_name,
        )],
        session.capabilities(),
    );
    let observations = {
        let mut turn = session.begin_observation_turn().unwrap();
        turn.admit_source(source).unwrap();
        turn.seal().unwrap()
    };
    session.classify_observations(observations).unwrap();
    if now == 1 {
        session.advance_mounted_identity_frame().unwrap();
    }
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("static mounting remains admitted while appearance denies"));
    if !expect_appearance_denial {
        if now == 1 {
            observer.push_native_display_presented();
        } else {
            // Replacement already published static paint; appearance remains staged.
            observer.push_native_display_settled_without_effects();
        }
    }
    let outcome = session.present_prepared_mounted_frame_internal(
        frame,
        worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
        now,
    );
    use crate::mounting::UiMountedFrameOutcome as Outcome;
    match (expect_appearance_denial, outcome) {
        (false, Outcome::Published(_)) => {}
        (true, Outcome::AdmissionDenied(rejection)) => assert_eq!(
            rejection.denial(),
            crate::mounting::UiMountedPresentationAdmissionDenial::AppearanceOutputUnavailable,
        ),
        (_, Outcome::AdmissionDenied(rejection)) => {
            panic!("clip frame {now}: {:?}", rejection.denial())
        }
        (_, Outcome::RetentionDenied(rejection)) => {
            panic!("clip frame {now}: {:?}", rejection.denial())
        }
        (_, Outcome::CompletionDenied(denial)) => panic!("clip frame {now}: {denial:?}"),
        (_, Outcome::RejectedBeforeEffects(rejected)) => {
            panic!("clip frame {now}: {:?}", rejected.rejections())
        }
        (_, other) => panic!(
            "clip frame {now} did not publish: {:?}",
            std::mem::discriminant(&other)
        ),
    }
}
