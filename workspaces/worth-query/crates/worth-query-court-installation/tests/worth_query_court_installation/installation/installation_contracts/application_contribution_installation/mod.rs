pub(crate) use worth_query_testkit_installation::facade::installation::installation_contracts::application_contribution_installation::*;

#[test]
fn installed_catalog_resolves_declaration_owned_contribution_closure() {
    let installed = installed_index()
        .bind_application_schema(ContributionSchema::declaration().unwrap())
        .unwrap();
    let catalog = installed.contributions();
    let topology = catalog.get("worth.tests.topology.v1").unwrap();

    assert_eq!(catalog.len(), 1);
    assert_eq!(topology.identity().as_str(), "worth.tests.topology.v1");
    assert_eq!(topology.member_ordinals(), &[0]);
    assert!(matches!(
        topology.members().next(),
        Some(ApplicationSchemaMember::Entity { entity }) if entity == "Component"
    ));
}

#[test]
fn reassigning_same_member_to_another_contribution_changes_installed_meaning() {
    let index = installed_index();
    let reassigned = ApplicationSchemaDeclarationBuilder::<ContributionSchema>::for_schema()
        .contributions()
        .register::<ReassignedContribution>()
        .unwrap()
        .build()
        .unwrap();

    let denial = index.bind_application_schema(reassigned).unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryInstalledApplicationSchemaDenialKind::SchemaMeaningChanged
    );
}
