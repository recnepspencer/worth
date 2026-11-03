//! Real installation issuance and retained operation affinity across schema loans.
use super::*;
use std::sync::Arc;

fn operation(
    schema: &crate::facade::WorthQueryInstalledApplicationSchema<TestSchema>,
) -> crate::facade::WorthQueryInstalledApplicationOperation<
    TestSchema,
    TestOperation<TestSchema>,
    TestInput,
> {
    schema
        .installed_operation(ApplicationOperationRef::from_declaration())
        .unwrap()
}

#[test]
fn retained_operation_is_shared_across_schema_rebinds_and_outlives_the_typed_loan() {
    let index = installed_index();
    let schema = index
        .bind_application_schema(TestSchema::declaration().unwrap())
        .unwrap();
    let catalog = schema.installed_view().retain_operation_catalog();
    assert!(catalog.operation("TestOperation").is_none());
    let installed = operation(&schema);
    let seal = installed.authority_identity_bytes();
    let retained = catalog
        .operation("TestOperation")
        .expect("successful installation registers its actual handle");
    let view = retained.installed_view();
    assert_eq!(view.binding_identity(), installed.binding_identity());
    assert_eq!(view.authority_identity_bytes(), seal);
    assert!(
        std::ptr::eq(view.contracts(), installed.contracts()),
        "the same compiled Arc backs the view"
    );
    drop(installed);
    drop(schema);
    let rebound = index
        .bind_application_schema(TestSchema::declaration().unwrap())
        .unwrap();
    assert!(Arc::ptr_eq(
        &catalog,
        &rebound.installed_view().retain_operation_catalog()
    ));
    let repeated = operation(&rebound);
    index.validate_application_operation(&repeated).unwrap();
    assert!(
        Arc::ptr_eq(&retained, &catalog.operation("TestOperation").unwrap()),
        "certified duplicates retain the original actual owner"
    );
    assert_eq!(retained.installed_view().authority_identity_bytes(), seal);
}

#[test]
fn a_foreign_real_operation_cannot_replace_a_catalogued_installation() {
    let index = installed_index();
    let schema = index
        .bind_application_schema(TestSchema::declaration().unwrap())
        .unwrap();
    let _installed = operation(&schema);
    let catalog = schema.installed_view().retain_operation_catalog();
    let original = catalog.operation("TestOperation").unwrap();
    let other = installed_index();
    let foreign_schema = other
        .bind_application_schema(TestSchema::declaration().unwrap())
        .unwrap();
    let foreign = operation(&foreign_schema);
    other.validate_application_operation(&foreign).unwrap();
    assert_ne!(
        foreign.binding_identity(),
        original.installed_view().binding_identity()
    );
    let denial = catalog
        .register(
            foreign_schema
                .installed_view()
                .retain_operation_catalog()
                .operation(foreign.operation())
                .unwrap(),
        )
        .expect_err("a genuine foreign handle is not this installation");
    assert_eq!(denial.kind(), crate::facade::WorthQueryApplicationOperationInstallationDenialKind::OperationMeaningChanged);
    assert!(Arc::ptr_eq(
        &original,
        &catalog.operation("TestOperation").unwrap()
    ));
}
