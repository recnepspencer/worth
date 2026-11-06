//! Real package validation, admission, schema binding and operation installation.
use crate::installation::aspect_native_identity::model;
use worth_query_declaration::facade::application_schema::*;
use worth_query_installation::facade::*;

#[test]
fn maximum_admitted_package_installs_and_one_more_byte_refuses_at_the_door() {
    // Package declaration meaning has a shared 64 MiB ceiling at this revision.
    // The 32,768 / 4 MiB admission identity is only the support metadata.
    const MAXIMUM: usize = 64 * 1024 * 1024;
    let declaration = model::declaration::<40, 12>(false).unwrap();
    let package = |length| {
        model::package(&declaration).definition(WorthQueryPortableDefinition::invariant(
            "finite-definition",
            "x".repeat(length),
        ))
    };
    let baseline = package(1)
        .validate()
        .unwrap()
        .canonical_work()
        .canonical_encoded_bytes();
    let mut length = MAXIMUM - baseline + 1;
    let validated = loop {
        match package(length).validate() {
            Ok(validated) => {
                let bytes = validated.canonical_work().canonical_encoded_bytes();
                if bytes == MAXIMUM {
                    break validated;
                }
                length += MAXIMUM - bytes;
            }
            Err(denial) => {
                assert_eq!(denial.kind(), WorthQueryPortablePackageValidationDenialKind::CanonicalEncodedByteBudgetExceeded);
                let attempted = denial.attempted_canonical_bytes().unwrap();
                assert_eq!(denial.maximum_canonical_bytes(), Some(MAXIMUM));
                length -= attempted - MAXIMUM;
            }
        }
    };
    let index = model::index_from(validated);
    let operation = index
        .bind_application_schema(declaration.clone())
        .unwrap()
        .installed_operation(model::operation())
        .expect("largest admitted package installs");
    assert!(
        operation
            .graph_obligations()
            .installation_evidence()
            .canonical_work()
            .canonical_entries()
            > 4096
    );
    let denial = package(length + 1).validate().unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryPortablePackageValidationDenialKind::CanonicalEncodedByteBudgetExceeded
    );
    assert_eq!(denial.maximum_canonical_bytes(), Some(MAXIMUM));
    assert_eq!(denial.attempted_canonical_bytes(), Some(MAXIMUM + 1));
}

fn assert_authorization_installs<const MANY: bool>() {
    let declaration = model::authorization::Authorized::<MANY>::declaration().unwrap();
    let index = model::index(&declaration);
    let schema = index
        .bind_application_schema(declaration)
        .expect("admitted authorization identities install");
    let operation = schema.installed_operation(model::operation()).unwrap();
    assert_eq!(operation.contracts().ability_requirements().len(), 1);
    let requirement = &operation.contracts().ability_requirements()[0];
    assert!(requirement.canonical_work().canonical_entries() > 4096);
}

#[test]
fn admitted_authorization_path_and_policy_outgrow_the_former_fixed_ceiling() {
    assert_authorization_installs::<false>();
    assert_authorization_installs::<true>();
}
