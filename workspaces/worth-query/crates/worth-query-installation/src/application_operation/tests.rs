#[test]
fn activation_operation_compiles_its_union_from_selected_capability_targets() {
    let contract =
        crate::application_capability::tests::delegation_activation_fixture::activated_contract();
    let selected = worth_query_declaration::facade::application_capability::
        application_capability_delegation_activation_program_targets(&contract)
        .expect("selected activation target has derived effects");
    let members = vec![worth_query_declaration::facade::application_schema::
        ApplicationSchemaMember::ApplicationCapability { contract }];

    assert_eq!(
        super::contract_resolution::operation_program_from_members(
            &members,
            "Activation",
            <crate::application_capability::tests::delegation_activation_fixture::ActivationInputBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::IDENTITY_NAME,
        ),
        selected
    );
}
