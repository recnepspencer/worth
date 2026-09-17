use crate::facade::{
    WorthQueryPortableDefinition, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage, WorthQueryPortablePackageValidationDenialKind,
};

fn package_with_order(reversed: bool) -> WorthQueryPortableDomainPackage {
    let definitions = [
        WorthQueryPortableDefinition::invariant("geometry.connected", "requires-outgoing:1:2:1"),
        WorthQueryPortableDefinition::graph_read_operation(
            "geometry.read",
            "direct-edge:relation-2",
        ),
    ];
    let mut package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "worth.geometry",
        1,
        0,
    ));
    if reversed {
        for definition in definitions.into_iter().rev() {
            package = package.definition(definition);
        }
    } else {
        for definition in definitions {
            package = package.definition(definition);
        }
    }
    package
        .requires_capability("query-read")
        .requires_configuration("query")
}

#[test]
fn portable_package_identity_is_declaration_order_independent() {
    let canonical = package_with_order(false).validate().unwrap();
    let reversed = package_with_order(true).validate().unwrap();
    assert_eq!(canonical.identity(), reversed.identity());
    assert_eq!(canonical.definitions(), reversed.definitions());
}

#[test]
fn one_field_definition_drift_is_a_typed_conflict() {
    let denial = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "worth.geometry",
        1,
        0,
    ))
    .definition(WorthQueryPortableDefinition::graph_read_operation(
        "geometry.read",
        "direct-edge",
    ))
    .definition(WorthQueryPortableDefinition::graph_read_operation(
        "geometry.read",
        "successor-walk",
    ))
    .validate()
    .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryPortablePackageValidationDenialKind::ConflictingDefinition
    );
    assert_eq!(denial.slot(), "geometry.read");
}

#[test]
fn malformed_portable_input_is_denied_without_panicking() {
    let denial =
        WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new("", 1, 0))
            .validate()
            .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryPortablePackageValidationDenialKind::EmptyDomainOwner
    );
}

#[test]
fn package_byte_budget_denial_reports_attempted_and_maximum_work() {
    let denial = package_with_order(false)
        .validate_with_canonical_work_limit(64)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryPortablePackageValidationDenialKind::CanonicalEncodedByteBudgetExceeded
    );
    assert_eq!(denial.maximum_canonical_bytes(), Some(64));
    assert!(denial.attempted_canonical_bytes().unwrap() > 64);
}

#[test]
fn duplicate_contribution_policy_is_denied_instead_of_silently_rewritten() {
    let denial = package_with_order(false)
        .permits_contribution("query-index")
        .permits_contribution("query-index")
        .validate()
        .unwrap_err();

    assert_eq!(
        denial.kind(),
        WorthQueryPortablePackageValidationDenialKind::DuplicateContributionCategory
    );
}

#[test]
fn delimiter_like_text_cannot_alias_package_identity_fields() {
    let package = || {
        WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
            "worth.geometry",
            1,
            0,
        ))
    };
    let left = package()
        .definition(WorthQueryPortableDefinition::graph_read_operation(
            "geometry:read",
            "direct",
        ))
        .validate()
        .unwrap();
    let right = package()
        .definition(WorthQueryPortableDefinition::graph_read_operation(
            "geometry",
            "read:direct",
        ))
        .validate()
        .unwrap();

    assert_ne!(left.identity(), right.identity());
}
