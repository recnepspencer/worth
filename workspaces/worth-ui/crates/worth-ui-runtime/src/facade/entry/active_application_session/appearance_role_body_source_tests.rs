use super::support;

#[test]
fn role_body_successors_repaint_reject_retry_and_recover_with_exact_snapshot_basis() {
    let original = support::validation_background_role(support::APPEARANCE_TOKEN);
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    host.set_capabilities(worth_ui_host_native::appearance_capability_report());
    let mut session = support::appearance_component_builder(&original)
        .register_appearance_theme_bundle(two_slot_theme())
        .unwrap()
        .with_rust_authored_declaration_fixture(support::appearance_fixture(&original))
        .freeze()
        .map(|app| {
            crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
                app,
                host.clone(),
            )
        })
        .unwrap()
        .launch()
        .unwrap();
    super::mounting_fixture::mount(&mut session, 1_000);
    for (edit, (slot, expected, reject)) in [
        (support::APPEARANCE_BASE_TOKEN, [180, 40, 60, 255], false),
        (support::APPEARANCE_TOKEN, [17, 34, 51, 255], true),
        (support::APPEARANCE_BASE_TOKEN, [180, 40, 60, 255], false),
        (support::APPEARANCE_TOKEN, [17, 34, 51, 255], false),
    ]
    .into_iter()
    .enumerate()
    {
        let role = support::validation_background_role(slot);
        let previous_snapshot = session.capabilities().digest();
        let previous_generation = session.active_generation_identity();
        let previous_publication = session.mounted.current_publication().cloned();
        let source = support::appearance_candidate_submission(
            &session,
            &format!("role-body-edit-{edit}"),
            Some(&role),
        );
        let mut turn = session.begin_observation_turn().unwrap();
        turn.admit_source(source).unwrap();
        let observations = turn.seal().unwrap();
        let crate::runtime::observation::UiChangeClassificationOutcome::Changed(changed) =
            session.classify_observations(observations).unwrap()
        else {
            panic!("body-only source edit must require presentation");
        };
        let lifecycle = session
            .resolve_affected_scope(changed)
            .unwrap()
            .resolve_identity_lifecycle()
            .unwrap();
        let plan = session
            .compile_rebind_plan(
                lifecycle,
                crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
            )
            .unwrap();
        let previous_owners = session.appearance_owner_snapshot.clone();
        if reject {
            host.push_rejected();
        } else {
            host.push_native_display_settled_without_effects();
        }
        let retry = match session
            .prepare_rebind(
                plan,
                crate::runtime::rebind::UiRebindExecutionRequest::new(1),
            )
            .unwrap()
            .execute(1)
        {
            crate::runtime::rebind::UiRebindOutcome::Published(receipt) if !reject => {
                assert!(receipt.mounted_publication().is_some());
                None
            }
            crate::runtime::rebind::UiRebindOutcome::RejectedBeforeEffects(denial) if reject => {
                assert_eq!(
                    denial.cause(),
                    crate::runtime::rebind::UiRebindDenialCause::HostRejectedBeforeEffects
                );
                Some(
                    denial
                        .detach_retry_for_native()
                        .unwrap_or_else(|_| panic!("retain exact retry")),
                )
            }
            crate::runtime::rebind::UiRebindOutcome::RejectedBeforeEffects(denial) => {
                panic!("role-body cutover denied: {:?}", denial.cause())
            }
            _ => panic!("unexpected role-body outcome"),
        };
        if let Some(retry) = retry {
            assert_eq!(session.capabilities().digest(), previous_snapshot);
            assert_eq!(session.active_generation_identity(), previous_generation);
            assert_eq!(
                session.mounted.current_publication(),
                previous_publication.as_ref()
            );
            assert!(session
                .appearance_owner_snapshot
                .as_ref()
                .zip(previous_owners.as_ref())
                .is_some_and(|(current, previous)| current.same_publication_predecessor(previous)));
            super::test_support::assert_unpublished_surface(
                session
                    .mounted
                    .current_unpublished_appearance()
                    .unwrap()
                    .unwrap(),
                [180, 40, 60, 255],
            );
            host.push_native_display_settled_without_effects();
            assert!(matches!(
                retry.rebase_content_and_retry(&mut session, 2).unwrap(),
                crate::runtime::rebind::UiRebindOutcome::Published(_)
            ));
        }
        assert_ne!(session.capabilities().digest(), previous_snapshot);
        assert_ne!(session.active_generation_identity(), previous_generation);
        assert_eq!(
            session.capabilities().appearance_roles().get(role.role()),
            Some(&role)
        );
        super::test_support::assert_unpublished_surface(
            session
                .mounted
                .current_unpublished_appearance()
                .unwrap()
                .unwrap(),
            expected,
        );
    }
    let _ = session.shutdown();
}

fn two_slot_theme() -> crate::capability::FrozenAppearanceThemeCapabilities {
    let values = [
        (support::APPEARANCE_TOKEN, [17, 34, 51, 255]),
        (support::APPEARANCE_BASE_TOKEN, [180, 40, 60, 255]),
    ];
    let catalog = crate::capability::UiThemeSlotCatalog::admit(
        1,
        values.iter().map(|(slot, _)| {
            crate::capability::UiThemeSlotDeclaration::new(
                crate::capability::ThemeTokenId::new(*slot).unwrap(),
                crate::capability::ThemeTokenFamily::surface(),
                worth_ui_dsl::UiThemeValueKind::Color,
                crate::capability::ThemeTokenSource::application(),
                crate::capability::UiThemeSlotDisclosure::Public,
                crate::capability::UiThemeSlotSuccessorCompatibility::ExactMeaning,
                None,
            )
        }),
    )
    .unwrap();
    let identity = crate::capability::UiThemeDefinitionIdentity::new("theme.role-body").unwrap();
    let definition = crate::capability::UiThemeDefinition::admit(
        identity.clone(),
        1,
        &catalog,
        values.into_iter().map(|(slot, channels)| {
            (
                crate::capability::ThemeTokenId::new(slot).unwrap(),
                worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels(
                    channels,
                )),
            )
        }),
    )
    .unwrap();
    crate::capability::FrozenAppearanceThemeCapabilities::admit(catalog, identity, vec![definition])
        .unwrap()
}
