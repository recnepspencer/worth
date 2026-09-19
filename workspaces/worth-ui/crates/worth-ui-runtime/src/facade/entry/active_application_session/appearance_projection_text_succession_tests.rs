use super::super::{fixture, set_spans, set_text, text_contract};

#[test]
fn prepared_text_succession_uses_candidate_contract_and_keeps_owner_edits_until_commit() {
    let role = fixture::foreground_role();
    let declared = text_contract();
    let dynamic = crate::capability::ComponentSemanticTextContract::spanned(
        declared.theme_token().clone(),
        declared.layer_semantic_order(),
        declared.scalar_spans().iter().map(|span| {
            crate::capability::ComponentSemanticTextSpanContract::new(
                span.original_range(),
                span.foreground_token().clone(),
                span.style().clone(),
            )
            .unwrap()
        }),
    )
    .unwrap();
    assert_ne!(dynamic, declared);
    let (mut owner, _) = fixture::session_with_text(&role, 65_537, Some(declared.clone()));
    let (identity, previous_graph) = text_node(&owner);
    owner
        .register_application_semantic_text(identity.clone(), previous_graph)
        .unwrap();
    set_text(&mut owner, 0, "AB");
    set_spans(&mut owner, 1, dynamic.scalar_spans().to_vec());

    for changed_contract in [false, true] {
        let contract = if changed_contract {
            crate::capability::ComponentSemanticTextContract::spanned(
                declared.theme_token().clone(),
                declared.layer_semantic_order() + 1,
                declared.scalar_spans().iter().cloned(),
            )
            .unwrap()
        } else {
            declared.clone()
        };
        let (candidate, _) = fixture::session_with_text_at_module(
            &role,
            contract.clone(),
            "appearance/relocated.wui",
        );
        let (_, graph) = text_node(&candidate);
        assert_ne!(
            previous_graph, graph,
            "independently admitted module paths must issue distinct graph identities"
        );
        let prepared = owner
            .presentation
            .prepare_text_succession(
                owner.capabilities(),
                candidate.capabilities(),
                candidate.graph(),
            )
            .unwrap();
        assert!(prepared.is_current(&owner.presentation));
        assert_content(
            &owner.presentation.project_complete().unwrap(),
            previous_graph,
            &dynamic,
        );
        assert_content(
            &prepared.project().unwrap(),
            graph,
            if changed_contract {
                &contract
            } else {
                &dynamic
            },
        );
        if changed_contract {
            prepared.commit(&mut owner.presentation);
            assert_content(
                &owner.presentation.project_complete().unwrap(),
                graph,
                &contract,
            );
        }
        let _ = candidate.shutdown();
    }
    let _ = owner.shutdown();
}

#[test]
fn candidate_text_registration_adds_empty_owner() {
    let role = fixture::role();
    let contract = text_contract();
    let (mut owner, _) = fixture::session_with_text(&role, 65_537, None);
    let (candidate, _) = fixture::session_with_text(&role, 65_537, Some(contract));
    let (_, graph) = text_node(&candidate);
    let added = owner
        .presentation
        .prepare_text_succession(
            owner.capabilities(),
            candidate.capabilities(),
            candidate.graph(),
        )
        .unwrap();
    assert!(added.project().unwrap().content().is_empty());
    added.commit(&mut owner.presentation);
    set_text(&mut owner, 0, "AB");
    assert!(owner
        .presentation
        .project()
        .unwrap()
        .content()
        .get(graph)
        .is_some());
    let _ = candidate.shutdown();
    let _ = owner.shutdown();
}

#[test]
fn candidate_text_registration_removes_old_capability() {
    let role = fixture::role();
    let (mut owner, _) = fixture::session_with_text(&role, 65_537, Some(text_contract()));
    let (candidate, _) = fixture::session_with_text(&role, 65_537, None);
    let (identity, graph) = text_node(&owner);
    owner
        .register_application_semantic_text(identity, graph)
        .unwrap();
    set_text(&mut owner, 0, "AB");
    let removed = owner
        .presentation
        .prepare_text_succession(
            owner.capabilities(),
            candidate.capabilities(),
            candidate.graph(),
        )
        .unwrap();
    assert!(removed.project().unwrap().content().is_empty());
    assert!(!owner.presentation.project().unwrap().content().is_empty());
    removed.commit(&mut owner.presentation);
    assert!(owner
        .presentation
        .project_complete()
        .unwrap()
        .content()
        .is_empty());
    let _ = candidate.shutdown();
    let _ = owner.shutdown();
}

fn text_node(
    session: &crate::facade::WorthUiActiveApplicationSession,
) -> (Box<str>, crate::graph::UiGraphNodeIdentity) {
    let identity = format!(
        "component:{}",
        crate::runtime::tests::appearance_component_session_test_support::APPEARANCE_NODE_A
    );
    let graph = session
        .graph()
        .snapshot()
        .nodes()
        .iter()
        .find(|node| node.declaration_identity().authored_semantic_name() == identity)
        .unwrap()
        .graph_node_identity();
    (identity.into(), graph)
}

fn assert_content(
    projection: &crate::runtime::presentation_state::UiApplicationPresentationProjection,
    graph: crate::graph::UiGraphNodeIdentity,
    contract: &crate::capability::ComponentSemanticTextContract,
) {
    let content = projection.content();
    let crate::mounting::UiMountedSemanticTextContent::Scalar(row) = content.get(graph).unwrap()
    else {
        panic!("scalar text");
    };
    let crate::mounting::UiMountedSemanticTextValueDirective::Replace(value) = row.value() else {
        panic!("prepared logical value");
    };
    assert_eq!(value.as_ref(), "AB");
    assert_eq!(row.formatting().unwrap().contract(), contract);
}
