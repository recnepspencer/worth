use super::*;

#[test]
fn equivalent_installed_queries_converge_and_identity_dimensions_do_not_alias() {
    let schema = installed_schema();
    let left = schema.certification_query(query_reference()).unwrap();
    let equivalent = schema.certification_query(query_reference()).unwrap();
    let changed_order =
        definition(ApplicationQueryOrderingDirection::Ascending, "sequence").into_erased();
    let changed_shape =
        definition(ApplicationQueryOrderingDirection::Descending, "position").into_erased();

    assert_eq!(left.identity(), equivalent.identity());
    assert!(left.shares_compiled_contract_with(&equivalent));
    assert_eq!(
        left.read_family_binding().identity(),
        equivalent.read_family_binding().identity()
    );
    assert_eq!(
        left.read_family_binding().planning_contract(),
        left.read_graph()
    );
    assert_eq!(
        left.read_family_binding().canonical_planning_identity(),
        left.read_graph().canonical_planning_basis().digest()
    );
    assert_eq!(
        left.graph_obligations().identity(),
        equivalent.graph_obligations().identity()
    );
    assert_eq!(left.graph_obligations().rows().len(), 1);
    assert_eq!(
        left.graph_obligations().rows()[0].kind(),
        crate::graph_obligation::WorthQueryInstalledGraphObligationKind::GraphRead
    );
    assert_eq!(
        left.graph_obligations()
            .installation_evidence()
            .canonical_work()
            .digest_text_materializations(),
        0
    );
    assert_ne!(
        definition(ApplicationQueryOrderingDirection::Descending, "sequence")
            .into_erased()
            .canonical_basis(),
        changed_order.canonical_basis()
    );
    assert_ne!(
        definition(ApplicationQueryOrderingDirection::Descending, "sequence")
            .into_erased()
            .canonical_basis(),
        changed_shape.canonical_basis()
    );
    assert_eq!(
        left.read_graph().relations()[0].relation(),
        "AccountActivity"
    );
    assert_eq!(
        left.read_graph().ordering()[0].collection_path(),
        "root/relation[0]"
    );
    assert_eq!(
        left.read_graph().ordering()[0].slot_type(),
        <ActivitySequenceSlot as worth_query_declaration::facade::portable_identity::WorthQueryPortableType>::PORTABLE_TYPE_IDENTITY.as_str()
    );
}

#[test]
fn authorization_scope_is_identity_bearing() {
    let account_scoped = definition_for::<ActivityQuery, Account, ActivitySequenceSlot>(
        ActivityQuery::reference(),
        Account::reference(),
        ApplicationQueryOrderingDirection::Descending,
        "sequence",
    )
    .into_erased();
    let activity_scoped = definition_for::<ActivityScopedQuery, Activity, ActivitySequenceSlot>(
        ActivityScopedQuery::reference(),
        Activity::reference(),
        ApplicationQueryOrderingDirection::Descending,
        "sequence",
    )
    .into_erased();

    assert_ne!(
        account_scoped.canonical_basis(),
        activity_scoped.canonical_basis()
    );
    assert_eq!(account_scoped.scope_entity(), "Account");
    assert_eq!(activity_scoped.scope_entity(), "Activity");
}
