use super::*;

struct OptionalWorkSchema;
impl OperationReads<TestOperation<OptionalWorkSchema>> for FixtureEntity<OptionalWorkSchema> {}
impl ApplicationSchema for OptionalWorkSchema {
    const OWNER: &'static str = "typed-test";
    const NAME: &'static str = "OptionalWorkSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        let operation =
            ApplicationOperationRef::<Self, TestOperation<Self>, TestInput>::from_declaration();
        let entity =
            ApplicationEntityRef::<Self, FixtureEntity<Self>>::from_schema_identifier("TestEntity");
        ApplicationSchemaDeclarationBuilder::<Self>::for_schema()
            .entity(entity)
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_read_entity(operation, entity)
            .build()
    }
}

#[test]
fn installed_operation_without_projection_budget_has_no_intrinsic_work_limit() {
    let operation = ApplicationOperationRef::<
        OptionalWorkSchema,
        TestOperation<OptionalWorkSchema>,
        TestInput,
    >::from_declaration();
    let declaration = OptionalWorkSchema::declaration().expect("no intrinsic projection ceiling");
    let index = installed_index_for(declaration.clone());
    let schema = index.bind_application_schema(declaration).unwrap();
    let installed = schema.installed_operation(operation).unwrap();
    index.validate_application_operation(&installed).unwrap();
    assert_eq!(installed.contracts().projection_work_budget(), None);
}
