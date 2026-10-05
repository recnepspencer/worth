//! Retained principal validation keeps the original installation issuer.

use super::*;

#[test]
fn retained_principal_validation_rejects_an_independent_same_content_issuer() {
    let runtime = WorthQueryInstallationRuntimeIdentity::fresh();
    let original = installed_index_in_runtime(runtime.retained());
    let independently_rooted = installed_index_in_runtime(runtime);
    assert_eq!(original.identity(), independently_rooted.identity());

    let schema = original
        .bind_application_schema(IdentitySchema::declaration().unwrap())
        .unwrap();
    let binding = schema
        .principal_binding(IdentityBinding::reference())
        .unwrap();
    let retained = original
        .retain_validated_principal_binding(&binding)
        .unwrap();
    assert!(original.retains_validated_principal_binding(&retained, &binding));
    assert!(original
        .rebuild()
        .retains_validated_principal_binding(&retained, &binding));

    assert_eq!(
        independently_rooted
            .validate_principal_binding(&binding)
            .unwrap_err()
            .kind(),
        WorthQueryPrincipalBindingInstallationDenialKind::AuthorityMismatch
    );
    assert!(!independently_rooted.retains_validated_principal_binding(&retained, &binding));
    assert!(!original
        .successor_generation()
        .retains_validated_principal_binding(&retained, &binding));
    assert!(!installed_index().retains_validated_principal_binding(&retained, &binding));

    let foreign_schema = independently_rooted
        .bind_application_schema(IdentitySchema::declaration().unwrap())
        .unwrap();
    let foreign_binding = foreign_schema
        .principal_binding(IdentityBinding::reference())
        .unwrap();
    assert_eq!(
        binding.binding_identity(),
        foreign_binding.binding_identity()
    );
    assert_ne!(
        binding.authority_identity(),
        foreign_binding.authority_identity()
    );
    assert!(!original.retains_validated_principal_binding(&retained, &foreign_binding));
}
