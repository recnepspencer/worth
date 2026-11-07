use worth_query_testkit_decl::fragment_fixture::*;

struct WrongFieldType;
impl<S> Registration<S> for WrongFieldType {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<S>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<S>, ApplicationSchemaDeclarationDenial> {
        let operation = ApplicationOperationRef::<S, SetMetric, ()>::from_declaration();
        let field: RecordField<_, _, String> =
            record_field(Fixture, Item, Metrics, "components[1]").resolve()?;
        Ok(builder
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_read_record_field(operation, field))
    }
}
struct WrongTypeSchema;
impl Fragment for WrongTypeSchema {
    const NAME: &'static str = "wrong_type";
    type Bindings = Bindings;
    type Members = (Item, WrongFieldType);
}

#[test]
fn field_handle_lookup_checks_the_value_type_at_schema_build() {
    assert!(
        matches!(declare(WrongTypeSchema), Err(ApplicationSchemaDeclarationDenial::InvalidRecordField { field }) if field.field() == "components[1]")
    );
}

struct Entry;
impl Entity for Entry {
    const NAME: &'static str = "entry";
    type Aspects = (Label, Option<Metrics>);
    type Relations = ();
}
impl OperationCreates<SetMetric> for Entry {}
struct Consumers<const MODE: u8>;
impl<const MODE: u8> Fragment for Consumers<MODE> {
    const NAME: &'static str = "fixture";
    type Bindings = Bindings;
    type Members = (Item, Entry, FieldConsumers<MODE>);
}
struct FieldConsumers<const MODE: u8>;
impl<S, const MODE: u8> Registration<S> for FieldConsumers<MODE> {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<S>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<S>, ApplicationSchemaDeclarationDenial> {
        let field: RecordField<_, _, u64> =
            record_field(Consumers::<MODE>, Item, Metrics, "components[1]").resolve()?;
        let operation = ApplicationOperationRef::<S, SetMetric, ()>::from_declaration();
        let body = if MODE == 0 {
            operation
                .body()
                .expect_record(field.clone(), 3)
                .unwrap()
                .assign_record(field, 5)
                .unwrap()
                .finish()
                .unwrap()
        } else {
            let entity = ApplicationEntityRef::<S, Entry>::from_schema_identifier(Entry::NAME);
            let mut creation = operation
                .body()
                .expect_record(field, 3)
                .unwrap()
                .create_in_scope(entity, "child")
                .unwrap();
            if MODE != 2 {
                let caption = record_field(Consumers::<MODE>, Entry, Label, "text").resolve()?;
                creation = creation
                    .initialize_record(caption, "created".to_owned())
                    .unwrap();
            }
            if MODE == 3 {
                let partial =
                    record_field(Consumers::<MODE>, Entry, Metrics, "components[1]").resolve()?;
                creation = creation.initialize_record(partial, 7u64).unwrap();
            }
            creation.finish().unwrap()
        };
        Ok(builder.operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish()
                .with_body(body)
                .unwrap(),
        ))
    }
}

#[test]
fn preconditions_and_created_fields_use_the_same_record_handle() {
    for create in [false, true] {
        let schema = if create {
            declare(Consumers::<1>).unwrap().into_erased()
        } else {
            declare(Consumers::<0>).unwrap().into_erased()
        };
        let body = schema
            .members()
            .iter()
            .find_map(|member| match member {
                ApplicationSchemaMember::OperationBody { body, .. } => Some(body),
                _ => None,
            })
            .unwrap();
        let field = if create {
            body.creation().unwrap().fields()[0].field()
        } else {
            body.scope_fields()[0].field()
        };
        assert_eq!(field.field(), if create { "text" } else { "components[1]" });
    }
}

#[test]
fn created_records_require_complete_required_and_selected_optional_aspects() {
    assert!(matches!(
        declare(Consumers::<2>),
        Err(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency { .. })
    ));
    assert!(matches!(
        declare(Consumers::<3>),
        Err(ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency { .. })
    ));
}

#[test]
fn policies_and_requirement_operands_use_the_record_handle() {
    let field: RecordField<_, _, u64> = record_field(Fixture, Item, Metrics, "components[1]")
        .resolve()
        .unwrap();
    let operation = ApplicationOperationRef::<(), SetMetric, ()>::from_declaration();
    operation
        .requirement_operands()
        .record_field("coordinate", field.clone())
        .unwrap();
    let entity = ApplicationEntityRef::<(), Item>::from_schema_identifier(Item::NAME);
    let policy = ApplicationAuthorizationPathBuilder::from_principal(entity)
        .where_record_equal(field.clone(), 11)
        .unwrap()
        .allow(entity);
    assert_eq!(policy.predicates()[0].field(), field.locus().field());
}

impl ApplicationSchema for WrongTypeSchema {
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declare(Self)
    }
}

impl<const MODE: u8> ApplicationSchema for Consumers<MODE> {
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declare(Self)
    }
}
