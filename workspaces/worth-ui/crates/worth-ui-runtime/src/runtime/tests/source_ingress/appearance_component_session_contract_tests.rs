use super::*;

#[test]
fn component_descriptor_rejects_the_actual_backdrop_contract() {
    let token = crate::capability::ThemeTokenId::new(APPEARANCE_TOKEN).unwrap();
    let result = appearance_component_with_contract(
        ACTIVE_COMPONENT,
        token,
        worth_ui_dsl::UiAppearanceAspectContract::backdrop(),
    );
    assert_eq!(
        result,
        Err(
            crate::capability::ComponentAppearanceAspectContractDenial::BackdropContractOnComponent
        )
    );
}

#[test]
fn direct_semantic_attachment_changes_use_admitted_graph_declaration_identity() {
    let role = validation_background_role(APPEARANCE_TOKEN);
    for attached in [APPEARANCE_NODE_A, APPEARANCE_NODE_B] {
        let mut session = source_backed_two_node_appearance_session(&role);
        let generation = session.active_generation_identity();
        let source = two_node_appearance_candidate_submission(
            &session,
            "two-node-appearance-current",
            &role,
            attached,
        );
        let mut turn = session.begin_observation_turn().unwrap();
        turn.admit_source(source).unwrap();
        let observations = turn.seal().unwrap();
        let classified = session.classify_observations(observations).unwrap();
        if attached == APPEARANCE_NODE_A {
            assert!(matches!(
                classified,
                crate::runtime::observation::UiChangeClassificationOutcome::ObservedNoChange(_)
            ));
        } else {
            let crate::runtime::observation::UiChangeClassificationOutcome::Changed(change) =
                classified
            else {
                panic!("moving the attachment changes both actual graph declarations");
            };
            for identity in [APPEARANCE_NODE_A, APPEARANCE_NODE_B] {
                let expected = crate::fact_contract::UiAuthoredFactSelector::node(identity);
                assert_eq!(
                    change
                        .facts()
                        .iter()
                        .filter_map(|fact| fact.authored_source())
                        .filter(|fact| fact.selector() == &expected
                            && fact.kind()
                                == crate::fact_contract::UiAuthoredFactKind::SemanticsChanged)
                        .count(),
                    1
                );
            }
        }
        assert_eq!(session.active_generation_identity(), generation);
        let _ = session.shutdown();
    }
}
