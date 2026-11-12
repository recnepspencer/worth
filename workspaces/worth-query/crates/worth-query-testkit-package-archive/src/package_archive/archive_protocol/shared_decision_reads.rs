//! A genuine typed package supplies shared contents, operation and compiled graph references.
use worth_query_declaration::facade::application_schema::{fragment::*, *};
pub use worth_query_declaration::facade::application_schema::{
    ApplicationDecisionReadScopeMeaning, ApplicationSchemaMember,
};
pub use worth_query_installation::facade::{
    WorthQueryExpectedPortablePackageIdentity, WorthQueryPortableOperationGraphReadScope,
    WorthQueryPortablePackageReconstruction, WorthQueryPortablePackageReconstructionLimits,
    WorthQueryPortablePackageRecord, WorthQueryPortablePackageRecordFamily,
    WorthQueryValidatedPortableDomainPackage,
};
use worth_query_installation::facade::{
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};
pub use worth_query_package_archive::facade::*;

struct Schema;
struct One;
struct Two;
struct Common;
impl Fragment for Schema {
    const NAME: &'static str = "archive-shared";
    type Bindings = NoBindings;
    type Members = (One, Two);
}
impl Entity for One {
    const NAME: &'static str = "One";
    type Aspects = ();
    type Relations = ();
}
impl Entity for Two {
    const NAME: &'static str = "Two";
    type Aspects = ();
    type Relations = ();
}
impl OperationReads<Common> for One {}
impl OperationReads<Common> for Two {}
impl OperationReads<Read> for One {}
impl OperationReads<Read> for Common {}
worth_query_declaration::worth_query_structured_value_binding!(Input for () { identity: "archive.input" });
worth_query_declaration::worth_query_operation!(Read for Schema, input Input);
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "archive.shared";
    const NAME: &'static str = "SharedArchive";
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declaration(false)
    }
}
fn declaration(
    reverse: bool,
) -> Result<ApplicationSchemaDeclaration<Schema>, ApplicationSchemaDeclarationDenial> {
    let scope = ApplicationDecisionReadScopeDefinition::<Schema, Common>::new();
    let one = entity(Schema, One);
    let two = entity(Schema, Two);
    let scope = if reverse {
        scope.entity(two).entity(one)
    } else {
        scope.entity(one).entity(two)
    };
    let schema = ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(Schema)?
        .decision_read_scope(scope);
    let reference = schema.decision_read_scope_reference::<Common>()?;
    schema
        .operation_read_entity(Read::reference(), one)
        .operation_decision_read_scope(Read::reference(), reference)
        .operation(
            Read::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .build()
}
pub fn package(
    reverse: bool,
) -> (
    WorthQueryValidatedPortableDomainPackage,
    ApplicationDecisionReadScopeMeaning,
) {
    let declaration = declaration(reverse).unwrap();
    let scope = declaration
        .erased()
        .members()
        .iter()
        .find_map(|member| match member {
            ApplicationSchemaMember::DecisionReadScope { definition } => Some(definition.clone()),
            _ => None,
        })
        .unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        Schema::OWNER,
        1,
        0,
    ))
    .application_schema(declaration)
    .validate()
    .unwrap();
    (package, scope)
}
