//! A genuine optional-aspect declaration and its installed Bank testkit views.
use std::collections::BTreeSet;

use worth_foundational::facade::{AspectContractRevision, AspectIdentity};
use worth_query_decl::facade::application_schema::*;

use super::{
    installed_program_targets, InstalledProgramTarget, WorthQueryInstallationAdmissionProfile,
    WorthQueryInstallationGeneration, WorthQueryInstallationRuntimeIdentity,
    WorthQueryInstalledPackageIndex, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage,
};

struct RemovalSchema;
worth_query_decl::facade::worth_query_entity!(Subject for RemovalSchema);
worth_query_decl::facade::worth_query_aspect!(OptionalFacts for RemovalSchema, Subject;
    identity = AspectIdentity(0xB4000001), revision = AspectContractRevision(1));
worth_query_decl::facade::worth_query_field!(Value for RemovalSchema, Subject, OptionalFacts:
    u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_decl::facade::worth_query_field!(Sibling for RemovalSchema, Subject, OptionalFacts:
    u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_decl::facade::worth_query_structured_value_binding!(InputBinding for () {
    identity: "worth.rust.unit"
});
worth_query_decl::facade::worth_query_operation!(RemoveFacts for RemovalSchema, input InputBinding);
worth_query_decl::facade::worth_query_operation_reads!(RemoveFacts => [Value, Sibling]);
worth_query_decl::facade::worth_query_operation_removes_aspects!(RemoveFacts => [OptionalFacts]);

impl ApplicationSchema for RemovalSchema {
    const OWNER: &'static str = "bank-removal-compatibility";
    const NAME: &'static str = "RemovalSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::for_schema()
            .entity(Subject::reference())
            .optional_aspect(Subject::reference(), OptionalFacts::reference())
            .field(Subject::reference(), Value::reference())
            .field(Subject::reference(), Sibling::reference())
            .operation(
                RemoveFacts::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_read_field(RemoveFacts::reference(), Value::reference())
            .operation_read_field(RemoveFacts::reference(), Sibling::reference())
            .operation_remove_aspect(RemoveFacts::reference(), OptionalFacts::reference())
            .build()
    }
}

pub fn declared_and_installed_targets() -> (BTreeSet<String>, BTreeSet<InstalledProgramTarget>) {
    let declaration = RemovalSchema::declaration().unwrap();
    let declared = declaration
        .erased()
        .members()
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::OperationProgram { operation, target }
                if operation == "RemoveFacts" =>
            {
                Some(
                    crate::facade::domain_contracts::schema_inventory::support::program_target(
                        target,
                    ),
                )
            }
            _ => None,
        })
        .collect();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "bank-removal-compatibility",
        1,
        0,
    ))
    .application_schema(declaration.clone())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    let index = WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap();
    let installed = index.bind_application_schema(declaration).unwrap();
    let operation = installed
        .installed_operation(RemoveFacts::reference())
        .unwrap();
    (declared, installed_program_targets(operation.contracts()))
}
