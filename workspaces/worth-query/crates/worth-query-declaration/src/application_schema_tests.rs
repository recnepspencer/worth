use crate::facade::application_schema::{
    fragment::{self, *},
    *,
};
struct Schema;
struct Entity;
struct Aspect;

struct ProgramSchema;
struct ProgramEntity;
struct ProgramOperation;
struct SchemaOperation;
struct SchemaInput;
worth_query_portable_type!(SchemaInput => "worth.query.test.schema-input");
struct NamespacedSchema;
struct InvalidOwnerSchema;
struct DottedMemberSchema;
struct IdentifierEntity;

#[path = "application_schema_marker_identity_tests.rs"]
mod application_schema_marker_identity_tests;

#[path = "application_schema_tests/operation_dependencies.rs"]
mod operation_dependencies;

crate::worth_query_structured_value_binding!(
    SchemaInputBinding for SchemaInput {
        identity: "worth.query.test.schema-input"
    }
);
crate::worth_query_structured_value_binding!(
    ProgramInputBinding for () {
        identity: "worth.rust.unit"
    }
);

impl ApplicationOperationMarkerIdentity<Schema> for SchemaOperation {
    type InputBinding = SchemaInputBinding;
    const IDENTIFIER: &'static str = "SchemaOperation";
}

impl ApplicationOperationMarkerIdentity<ProgramSchema> for ProgramOperation {
    type InputBinding = ProgramInputBinding;
    const IDENTIFIER: &'static str = "ProgramOperation";
}

struct MacroNamespacedSchema;
struct MacroNamespacedEntity;
impl fragment::Entity for MacroNamespacedEntity {
    const NAME: &'static str = "MacroNamespacedEntity";
    type Aspects = ();
    type Relations = ();
}
impl Fragment for MacroNamespacedSchema {
    const NAME: &'static str = "namespaced";
    type Bindings = NoBindings;
    type Members = (MacroNamespacedEntity,);
}
impl ApplicationSchema for MacroNamespacedSchema {
    const OWNER: &'static str = "bank.world";
    const NAME: &'static str = "MacroNamespacedSchema";
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl OperationCreates<ProgramOperation> for ProgramEntity {}

#[test]
fn fragment_accepts_the_canonical_namespace_qualified_owner() {
    let declaration = MacroNamespacedSchema::declaration().unwrap();
    assert_eq!(declaration.erased().owner(), "bank.world");
}

impl ApplicationSchema for Schema {
    const OWNER: &'static str = "schema-test";
    const NAME: &'static str = "Schema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declaration(false)
    }
}

impl ApplicationSchema for ProgramSchema {
    const OWNER: &'static str = "schema-test";
    const NAME: &'static str = "ProgramSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::<Self>::from_fragment(
            application_schema_marker_identity_tests::ProgramFixture,
        )?
        .operation(
            program_operation()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .build()
    }
}

impl ApplicationSchema for NamespacedSchema {
    const OWNER: &'static str = "bank.world";
    const NAME: &'static str = "NamespacedSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::<Self>::from_fragment(
            application_schema_marker_identity_tests::Identifiers,
        )?
        .build()
    }
}

impl ApplicationSchema for InvalidOwnerSchema {
    const OWNER: &'static str = "bank..world";
    const NAME: &'static str = "InvalidOwnerSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::<Self>::from_fragment(
            application_schema_marker_identity_tests::Identifiers,
        )?
        .build()
    }
}

impl ApplicationSchema for DottedMemberSchema {
    const OWNER: &'static str = "bank.world";
    const NAME: &'static str = "DottedMemberSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::<Self>::from_test_members(vec![
            ApplicationSchemaMember::Entity {
                entity: "Identifier.Entity".into(),
            },
        ])
        .build()
    }
}

#[test]
fn namespace_qualified_owner_is_valid_but_empty_segments_and_dotted_members_are_not() {
    NamespacedSchema::declaration().unwrap();
    assert_eq!(
        InvalidOwnerSchema::declaration().unwrap_err(),
        ApplicationSchemaDeclarationDenial::InvalidIdentifier
    );
    assert_eq!(
        DottedMemberSchema::declaration().unwrap_err(),
        ApplicationSchemaDeclarationDenial::InvalidIdentifier
    );
}

#[test]
fn application_schema_identity_converges_across_member_order() {
    let first = declaration(false).unwrap();
    let second = declaration(true).unwrap();
    assert_eq!(first.identity(), second.identity());
    assert_eq!(first.erased().members(), second.erased().members());
}

#[test]
fn field_without_its_exact_aspect_is_denied() {
    let mut members = declaration(false).unwrap().erased().members().to_vec();
    members.retain(|member| !matches!(member, ApplicationSchemaMember::Aspect { .. }));
    let denial = ApplicationSchemaDeclarationBuilder::<Schema>::from_test_members(members)
        .build()
        .unwrap_err();
    assert_eq!(denial, ApplicationSchemaDeclarationDenial::MissingAspect);
}

#[test]
fn currency_field_without_its_declared_currency_is_denied() {
    let mut members = declaration(false).unwrap().erased().members().to_vec();
    for member in &mut members {
        if let ApplicationSchemaMember::Field { unit, .. } = member {
            *unit = Some("UsdCurrency".into());
        }
    }
    let denial = ApplicationSchemaDeclarationBuilder::<Schema>::from_test_members(members)
        .build()
        .unwrap_err();
    assert_eq!(denial, ApplicationSchemaDeclarationDenial::MissingUnit);
}

#[test]
fn compile_capability_without_installed_operation_edge_is_denied() {
    let declaration = ProgramSchema::declaration().unwrap();
    let binding = ApplicationSchemaBindingIdentity::from_installed_parts(
        1,
        1,
        worth_foundational::facade::CanonicalDigestId::new([1; 32]),
        worth_foundational::facade::CanonicalDigestId::new([2; 32]),
    );
    let context = ApplicationSchemaAuthoringContext::from_installed_declaration(
        binding,
        declaration.erased(),
        declaration.member_provenance(),
    );
    let denial = TypedOperationBuilder::new(program_operation())
        .with_installed_context(context)
        .input(())
        .create(program_entity())
        .build()
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        ApplicationSchemaAuthoringDenialKind::OperationProgramNotInstalled
    );
}

#[test]
fn operation_program_with_missing_target_member_is_denied() {
    let denial = ApplicationSchemaDeclarationBuilder::<ProgramSchema>::for_schema()
        .operation(
            program_operation()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_create(program_operation(), program_entity())
        .build()
        .unwrap_err();
    assert!(matches!(
        denial,
        ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency { .. }
    ));
}

#[test]
fn mutation_precondition_without_the_exact_decision_read_is_denied() {
    let denial = ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(
        application_schema_marker_identity_tests::Fixture,
    )
    .unwrap()
    .operation(
        schema_operation()
            .definition()
            .no_external_effect()
            .no_aftermath()
            .finish(),
    )
    .operation_expected_fact(
        schema_operation(),
        application_schema_marker_identity_tests::selected_field(),
    )
    .build()
    .unwrap_err();
    assert!(matches!(
        denial,
        ApplicationSchemaDeclarationDenial::MissingOperationMutationPreconditionDependency { .. }
    ));
}

fn declaration(
    reverse: bool,
) -> Result<ApplicationSchemaDeclaration<Schema>, ApplicationSchemaDeclarationDenial> {
    let declared = ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(
        application_schema_marker_identity_tests::Fixture,
    )?
    .build()?;
    let mut members = declared.erased().members().to_vec();
    if reverse {
        members.reverse();
    }
    ApplicationSchemaDeclarationBuilder::<Schema>::from_test_members(members).build()
}

fn schema_operation() -> ApplicationOperationRef<Schema, SchemaOperation, SchemaInput> {
    ApplicationOperationRef::from_declaration()
}

fn program_entity() -> ApplicationEntityRef<ProgramSchema, ProgramEntity> {
    ApplicationEntityRef::from_schema_identifier("ProgramEntity")
}

fn program_operation() -> ApplicationOperationRef<ProgramSchema, ProgramOperation, ()> {
    ApplicationOperationRef::from_declaration()
}
