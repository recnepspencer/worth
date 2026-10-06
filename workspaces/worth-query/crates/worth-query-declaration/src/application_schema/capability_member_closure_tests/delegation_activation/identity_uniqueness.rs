//! A delegation activation keys its child grant by a unique identity field.

use super::*;

#[test]
fn activation_identity_must_be_a_unique_field() {
    let mut members = activation_members();
    for member in &mut members {
        if let ApplicationSchemaMember::Field { field, unique, .. } = member {
            if field == "Identity" {
                *unique = false;
            }
        }
    }
    assert_eq!(
        build_from_members(members),
        Err(ApplicationSchemaDeclarationDenial::DelegationActivationIdentityNotUnique)
    );
}
