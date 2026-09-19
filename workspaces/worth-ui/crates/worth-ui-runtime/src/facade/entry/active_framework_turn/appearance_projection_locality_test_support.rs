use std::rc::Rc;

use super::{CANDIDATE_TOKEN, UNSTYLED_COMPONENT};
use crate::runtime::tests::appearance_component_session_test_support as support;

pub(super) fn admit_theme(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    token: crate::capability::ThemeTokenId,
    color: &str,
) {
    let definition = format!(
        "theme.appearance.locality-{}",
        color.trim_start_matches('#')
    );
    support::replace_appearance_theme_definition_for_test(session, &definition, &token);
}

pub(super) fn admit_owner_snapshot(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    role_a: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    role_b: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    source_name: &str,
) {
    let candidate = both_roles_submission(session, role_a, role_b, source_name);
    let observations = {
        let mut turn = session.begin_observation_turn().unwrap();
        turn.admit_source(candidate).unwrap();
        turn.seal().unwrap()
    };
    session.classify_observations(observations).unwrap();
    assert!(session.has_appearance_owner_snapshot_for_test());
}

fn both_roles_submission(
    session: &crate::facade::WorthUiActiveApplicationSession,
    role_a: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    role_b: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    source_name: &str,
) -> crate::runtime::WorthUiWatchedCandidateSubmission {
    let attachment_a = worth_ui_dsl::UiAppearanceRoleAttachmentDeclaration::new(
        role_a.role().clone(),
        role_a.revision(),
    );
    let attachment_b = worth_ui_dsl::UiAppearanceRoleAttachmentDeclaration::new(
        role_b.role().clone(),
        role_b.revision(),
    );
    let module = worth_ui_dsl::WorthUiRustAuthoredArtifactInputModule::new("appearance/consumer")
        .with_appearance_role(role_a.clone())
        .with_appearance_role(role_b.clone());
    let module = module
        .with_component_appearance_role(support::APPEARANCE_NODE_A, attachment_a)
        .unwrap();
    let module = module
        .with_component_appearance_role(support::APPEARANCE_NODE_B, attachment_b)
        .unwrap()
        .with_semantic_declaration(unstyled_semantic());
    let input = worth_ui_dsl::WorthUiRustAuthoredArtifactInput::from_modules([module]);
    crate::runtime::tests::source_ingress_boundary_test_support::lower_rust_submission(
        crate::runtime::WorthUiSourceProvider::rust_authored(source_name)
            .with_rust_authored_input(input),
        [crate::runtime::WorthUiWatcherEvent::provider_revision(
            source_name,
        )],
        session.capabilities(),
    )
}

pub(super) fn graph_node_for(
    session: &crate::facade::WorthUiActiveApplicationSession,
    name: &str,
) -> crate::graph::UiGraphNodeIdentity {
    let graph = session.graph();
    let found = graph.node_identities().find(|identity| {
        graph.lookup().graph_node(*identity).is_some_and(|node| {
            let value = node.value();
            let declaration_identity = value.declaration_identity();
            let authored_name = declaration_identity.authored_semantic_name();
            authored_name == name || authored_name.strip_prefix("component:") == Some(name)
        })
    });
    found.unwrap_or_else(|| {
        let available = graph
            .node_identities()
            .filter_map(|identity| {
                graph.lookup().graph_node(identity).map(|node| {
                    node.value()
                        .declaration_identity()
                        .authored_semantic_name()
                        .to_owned()
                })
            })
            .collect::<Vec<_>>();
        panic!("missing locality graph node {name}; available: {available:?}")
    })
}

pub(super) fn locality_fixture(
    role_a: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    role_b: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) -> crate::facade::WorthUiRustAuthoredDeclarationFixture {
    let attachment_b = worth_ui_dsl::UiAppearanceRoleAttachmentDeclaration::new(
        role_b.role().clone(),
        role_b.revision(),
    );
    support::appearance_fixture(role_a)
        .with_appearance_role("appearance/consumer", role_b.clone())
        .with_component_appearance_role(
            "appearance/consumer",
            support::APPEARANCE_NODE_B,
            attachment_b,
        )
        .with_semantic_artifact_spec(
            worth_ui_dsl::UiDslSemanticArtifactSpec::new(
                worth_ui_dsl::UiDslSemanticKey::new(UNSTYLED_COMPONENT),
                worth_ui_dsl::UiDslSemanticFamily::Control,
                worth_ui_dsl::UiDslSourceProvenance::rust_authored("appearance/consumer", 0),
            )
            .with_structural_token(worth_ui_dsl::UiDslStructuralToken::new(
                "control:appearance-locality-unstyled",
            ))
            .with_component_reference(
                worth_ui_dsl::UiDslComponentReference::new(UNSTYLED_COMPONENT).unwrap(),
            )
            .unwrap(),
        )
}

fn unstyled_semantic() -> worth_ui_dsl::WorthUiSemanticArtifactDeclaration {
    worth_ui_dsl::WorthUiSemanticArtifactDeclaration::new(
        worth_ui_dsl::UiDslSemanticKey::new(UNSTYLED_COMPONENT),
        worth_ui_dsl::UiDslSemanticFamily::Control,
    )
    .with_structural_token(worth_ui_dsl::UiDslStructuralToken::new(
        "control:appearance-locality-unstyled",
    ))
    .with_component_reference(
        worth_ui_dsl::UiDslComponentReference::new(UNSTYLED_COMPONENT).unwrap(),
    )
    .unwrap()
}

pub(super) fn background_role(
    identity: &str,
    token: &str,
) -> worth_ui_dsl::UiAppearanceRoleDeclaration {
    let aspect = worth_ui_dsl::UiAppearanceAspect::Background;
    let contract = worth_ui_dsl::UiAppearanceAspectContract::component([aspect], []).unwrap();
    let partition = worth_ui_dsl::UiAppearancePartitionAuthoring::new([])
        .with_cell(worth_ui_dsl::UiAppearanceCell::when([]).uses_slot(
            worth_ui_dsl::UiThemeSlotIdentity::new(token).unwrap(),
            worth_ui_dsl::UiThemeValueKind::Color,
        ))
        .compile(aspect)
        .unwrap();
    worth_ui_dsl::UiAppearanceRoleDeclaration::admit(
        worth_ui_dsl::UiAppearanceRoleIdentity::new(identity).unwrap(),
        worth_ui_dsl::UiAppearanceRoleRevision::new(1).unwrap(),
        worth_ui_dsl::UiAppearanceRoleApplicability::AnyComponent,
        &contract,
        [(aspect, partition)],
    )
    .unwrap()
}

pub(super) fn theme_bundle() -> crate::capability::FrozenAppearanceThemeCapabilities {
    let token_a = crate::capability::ThemeTokenId::new(support::APPEARANCE_TOKEN).unwrap();
    let token_b = crate::capability::ThemeTokenId::new(CANDIDATE_TOKEN).unwrap();
    let slot = |token| {
        crate::capability::UiThemeSlotDeclaration::new(
            token,
            crate::capability::ThemeTokenFamily::surface(),
            worth_ui_dsl::UiThemeValueKind::Color,
            crate::capability::ThemeTokenSource::application(),
            crate::capability::UiThemeSlotDisclosure::Public,
            crate::capability::UiThemeSlotSuccessorCompatibility::ExactMeaning,
            None,
        )
    };
    let catalog = crate::capability::UiThemeSlotCatalog::admit(
        1,
        [slot(token_a.clone()), slot(token_b.clone())],
    )
    .unwrap();
    let definition = |name: &str, a, b| {
        crate::capability::UiThemeDefinition::admit(
            crate::capability::UiThemeDefinitionIdentity::new(name).unwrap(),
            1,
            &catalog,
            [
                (
                    token_a.clone(),
                    worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels(a)),
                ),
                (
                    token_b.clone(),
                    worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels(b)),
                ),
            ],
        )
        .unwrap()
    };
    // Each successor changes precisely the slot invalidated by the scenario.
    let definitions = vec![
        definition(
            "theme.appearance.locality",
            [17, 34, 51, 255],
            [68, 85, 102, 255],
        ),
        definition(
            "theme.appearance.locality-405060",
            [64, 80, 96, 255],
            [68, 85, 102, 255],
        ),
        definition(
            "theme.appearance.locality-607080",
            [64, 80, 96, 255],
            [96, 112, 128, 255],
        ),
        definition(
            "theme.appearance.locality-506070",
            [80, 96, 112, 255],
            [96, 112, 128, 255],
        ),
        definition(
            "theme.appearance.locality-708090",
            [112, 128, 144, 255],
            [96, 112, 128, 255],
        ),
        definition(
            "theme.appearance.locality-8090a0",
            [128, 144, 160, 255],
            [96, 112, 128, 255],
        ),
        definition(
            "theme.appearance.locality-90a0b0",
            [128, 144, 160, 255],
            [144, 160, 176, 255],
        ),
        definition(
            "theme.appearance.locality-a0b0c0",
            [160, 176, 192, 255],
            [144, 160, 176, 255],
        ),
        definition(
            "theme.appearance.locality-b0c0d0",
            [160, 176, 192, 255],
            [176, 192, 208, 255],
        ),
    ];
    crate::capability::FrozenAppearanceThemeCapabilities::admit(
        catalog,
        crate::capability::UiThemeDefinitionIdentity::new("theme.appearance.locality").unwrap(),
        definitions,
    )
    .unwrap()
}

pub(super) fn prepare_and_publish(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    now: u64,
    native_effects: bool,
) -> super::LocalityMetrics {
    for _ in session.inspect_mounted_identity().surface_bindings() {
        if native_effects {
            host.push_native_display_presented();
        } else {
            host.push_native_display_settled_without_effects();
        }
    }
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|stop| match stop {
            crate::facade::entry::WorthUiMountedFrameExecutionStop::Preparation(denial) => {
                panic!("locality frame should prepare: {denial:?}")
            }
            _ => panic!("locality frame stopped before preparation"),
        });
    let candidate_projection = frame.projection_rc_for_test();
    // Tick 5 is the first multi-consumer successor with a published owner basis.
    if now == 2
        && frame
            .appearance_selection_cost_report()
            .selected_instance_count()
            == 1
    {
        frame.verify_mixed_appearance_reconstruction_denial_and_retry();
    }
    if now == 5 {
        frame.verify_unpublished_appearance_member_denial();
    }
    if now == 6 {
        super::retirement_tests::verify_retry(session, &frame);
    }
    let canonical_consumers = frame.appearance_invalidation_batch().map_or(0, |batch| {
        batch.graph_consumers().len()
            + batch
                .mounted_consumers()
                .iter()
                .map(|(graph_node, _)| *graph_node)
                .collect::<std::collections::BTreeSet<_>>()
                .len()
    });
    let surface_projection = frame
        .surfaces()
        .first()
        .expect("locality frame has bound surfaces")
        .projection_owner();
    assert!(Rc::ptr_eq(&candidate_projection, &surface_projection));
    let report = frame.appearance_selection_cost_report();
    let outcome = session.present_prepared_mounted_frame_internal(
        frame,
        worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
        now,
    );
    let host_completed_without_effects =
        !native_effects && presentation_completed_without_effects(&outcome);
    let motion_commands = match &outcome {
        crate::mounting::UiMountedFrameOutcome::Published(receipt)
        | crate::mounting::UiMountedFrameOutcome::Reconciled(receipt) => {
            receipt.cost_report().appearance_motion_commands_visited()
        }
        _ => 0,
    };
    assert!(matches!(
        outcome,
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));
    let published_projection = session
        .current_mounted_projection_rc_for_test()
        .expect("published locality frame retains its owner");
    assert!(Rc::ptr_eq(&candidate_projection, &published_projection));
    super::LocalityMetrics {
        selected: report.selected_instance_count(),
        materialized: report.materialized_context_count(),
        canonical_consumers,
        index_entries: report.index_entries_touched(),
        lifecycle_retired: report.lifecycle_memberships_retired(),
        key_probes: report.membership_key_probes(),
        copied_nodes: report.membership_copied_avl_nodes(),
        traversed: report.membership_traversed_entries(),
        motion_commands,
        host_completed_without_effects,
    }
}

fn presentation_completed_without_effects(
    outcome: &crate::mounting::UiMountedFrameOutcome,
) -> bool {
    let mut completed_without_effects = false;
    if let crate::mounting::UiMountedFrameOutcome::Published(receipt)
    | crate::mounting::UiMountedFrameOutcome::Reconciled(receipt) = outcome
    {
        receipt.with_surface_presentations(|surfaces| {
            completed_without_effects = !surfaces.is_empty()
                && surfaces.iter().all(|surface| {
                    surface.effects().families().is_empty()
                        && surface.adapter_cost()
                            == worth_ui_host_contract::UiHostPresentationCostReport::default()
                });
        });
    }
    completed_without_effects
}
