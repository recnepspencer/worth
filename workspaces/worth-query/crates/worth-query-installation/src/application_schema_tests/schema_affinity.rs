//! Runtime, generation and installed meaning affinity of schema bindings.
use super::*;

#[test]
fn installed_schema_binding_is_runtime_generation_and_meaning_affine() {
    let index = installed_index();
    let binding = index
        .bind_application_schema(TestSchema::declaration().unwrap())
        .unwrap();
    index.validate_application_schema(&binding).unwrap();

    let foreign = installed_index();
    let denial = foreign.validate_application_schema(&binding).unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryInstalledApplicationSchemaDenialKind::ForeignRuntime
    );

    let successor = index.successor_generation();
    let denial = successor.validate_application_schema(&binding).unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryInstalledApplicationSchemaDenialKind::StaleGeneration
    );

    let denial = index
        .bind_application_schema(DriftedSchema::declaration().unwrap())
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryInstalledApplicationSchemaDenialKind::SchemaMeaningChanged
    );
}

#[test]
fn installed_schema_binding_rejects_package_and_admission_identity_drift() {
    let runtime = WorthQueryInstallationRuntimeIdentity::fresh();
    let same_runtime_for_package_drift = runtime.retained();
    let same_runtime_for_admission_drift = runtime.retained();
    let index = installed_index_with(runtime, false, "support");
    let binding = index
        .bind_application_schema(TestSchema::declaration().unwrap())
        .unwrap();

    let package_drift = installed_index_with(same_runtime_for_package_drift, true, "support");
    let denial = package_drift
        .validate_application_schema(&binding)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryInstalledApplicationSchemaDenialKind::PackageIdentityChanged
    );

    let admission_drift =
        installed_index_with(same_runtime_for_admission_drift, false, "other-support");
    let denial = admission_drift
        .validate_application_schema(&binding)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryInstalledApplicationSchemaDenialKind::AdmissionIdentityChanged
    );
}
