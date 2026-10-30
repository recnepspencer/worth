//! Owner-local portable readmission uses real validated and installed contracts.
use super::*;
use crate::facade::*;
use worth_foundational::facade::{AspectContractRevision, AspectIdentity};
use worth_query_declaration::facade::application_schema::*;
struct Schema;
worth_query_declaration::worth_query_entity!(Entity for Schema);
worth_query_declaration::worth_query_aspect!(Input for Schema, Entity; identity = AspectIdentity(0x91800200), revision = AspectContractRevision(1));
worth_query_declaration::worth_query_field!(Left for Schema, Entity, Input: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_declaration::worth_query_field!(Right for Schema, Entity, Input: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_declaration::worth_query_structured_value_binding!(InputBinding for () { identity: "worth.rust.unit" });
worth_query_declaration::worth_query_operation!(Remove for Schema, input InputBinding);
worth_query_declaration::worth_query_operation_reads!(Remove => [Left, Right]);
worth_query_declaration::worth_query_operation_removes_aspects!(Remove => [Input]);
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "removal-court";
    const NAME: &'static str = "Schema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::for_schema()
            .entity(Entity::reference())
            .optional_aspect(Entity::reference(), Input::reference())
            .field(Entity::reference(), Left::reference())
            .field(Entity::reference(), Right::reference())
            .operation(
                Remove::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_read_field(Remove::reference(), Left::reference())
            .operation_read_field(Remove::reference(), Right::reference())
            .operation_remove_aspect(Remove::reference(), Input::reference())
            .build()
    }
}
fn installed() -> WorthQueryInstalledApplicationSchema<Schema> {
    let declaration = Schema::declaration().unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "removal-court",
        1,
        0,
    ))
    .application_schema(declaration.clone())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
    .bind_application_schema(declaration)
    .unwrap()
}
#[test]
fn installed_parent_removal_readmission_requires_complete_same_binding_native_reads() {
    let schema = installed();
    let operation = schema.installed_operation(Remove::reference()).unwrap();
    let binding = schema.binding_identity();
    let contract = schema
        .native_contracts()
        .aspect("Entity", "Input")
        .unwrap()
        .contract()
        .clone();
    let portable = [
        crate::package::WorthQueryPortableOperationTouchScope::RemoveAspect {
            schema: "Schema".into(),
            entity: "Entity".into(),
            contract,
        },
    ];
    let reads = operation.contracts().graph_reads();
    let (touches, _) =
        install_portable_graph_touches(&binding, schema.native_contracts(), &portable, reads)
            .unwrap();
    assert!(
        matches!(&touches.scopes()[0], WorthQueryOperationTouchScope::RemoveAspect(scope) if scope.schema() == &binding)
    );
    let missing = crate::domain_operation::WorthQueryOperationGraphReadContract::NotRequired;
    assert!(install_portable_graph_touches(
        &binding,
        schema.native_contracts(),
        &portable,
        &missing
    )
    .is_err());
    let foreign = installed();
    let foreign_operation = foreign.installed_operation(Remove::reference()).unwrap();
    assert_ne!(foreign.binding_identity(), binding);
    assert!(install_portable_graph_touches(
        &binding,
        schema.native_contracts(),
        &portable,
        foreign_operation.contracts().graph_reads()
    )
    .is_err());
}
