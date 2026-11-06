//! Typed flat records install field reads; untrusted scalar shapes retain whole reads.
use super::*;

#[test]
fn scalar_descriptors_compile_whole_reads_and_flat_records_install_column_reads() {
    let source = FlatSchema::declaration().unwrap();
    assert_portable_mask(source.erased(), false);
    assert_portable_mask(&scalar(false, false), true);
    let package = crate::facade::WorthQueryPortableDomainPackage::new(
        crate::facade::WorthQueryPortableDomainIdentity::new(FlatSchema::OWNER, 1, 0),
    )
    .application_schema(source.clone())
    .validate()
    .unwrap();
    let admitted =
        crate::facade::WorthQueryInstallationAdmissionProfile::new("support", "configuration")
            .admit(package)
            .unwrap();
    let installed = crate::facade::WorthQueryInstalledPackageIndex::build(
        crate::facade::WorthQueryInstallationRuntimeIdentity::fresh(),
        crate::facade::WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
    .bind_application_schema(source)
    .unwrap();
    let operation = installed.installed_operation(Read::reference()).unwrap();
    let [crate::domain_operation::WorthQueryOperationGraphReadScope::NativeProjection(scope)] =
        operation.contracts().graph_reads().roles()[0].read_scopes()
    else {
        panic!("exact native projection")
    };
    assert!(scope.admits_field(&FieldKey::new("value").unwrap()));
    assert!(!scope.admits_field(&FieldKey::new("sibling").unwrap()));
    assert_eq!(scope.atomic_field_count(), Some(1));
    let native = installed
        .native_contracts()
        .aspect("Record", value().aspect())
        .unwrap();
    let projection =
        crate::domain_operation::WorthQueryOperationNativeProjectionContract::from_installed(
            native,
            aspects().projection_mask().whole_aspect(),
        )
        .unwrap();
    let scope = crate::domain_operation::WorthQueryOperationApplicationProjectionScope::new(
        installed.binding_identity(),
        "Record".into(),
        native.contract().key().clone(),
        projection,
        native.binding().clone(),
    );
    assert!(!scope.admits_field(&FieldKey::new("value").unwrap()));
    assert_eq!(scope.atomic_field_count(), None);
}
fn assert_portable_mask(declaration: &ErasedApplicationSchemaDeclaration, whole: bool) {
    let native = compile_portable_native_contract_records(declaration).unwrap();
    let operations = crate::application_operation::compile_portable_operation_contract_records(
        declaration,
        &native,
    )
    .unwrap();
    let [crate::package::WorthQueryPortableOperationGraphReadScope::NativeProjection {
        mask, ..
    }] = operations[0].graph_reads()
    else {
        panic!("one native projection")
    };
    assert_eq!(mask.is_whole_aspect(), whole);
    if !whole {
        assert_eq!(
            mask,
            &aspects().projection_mask().fields(["value"]).unwrap()
        );
    }
}
