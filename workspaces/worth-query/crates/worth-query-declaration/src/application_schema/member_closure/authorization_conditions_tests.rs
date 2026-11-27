//! Deny paths read only always-present values; ordered comparisons close only
//! on order-faithful families.
use worth_foundational::facade::{
    AbsenceLaw, AspectContractRevision, AspectIdentity, AspectValue, ScalarAspectType,
};

use super::super::super::{
    ApplicationAuthorizationComparison as Comparison, ApplicationAuthorizationFieldConstraint,
    ApplicationAuthorizationPath, ApplicationAuthorizationPathEffect as Effect,
    ApplicationAuthorizationPredicate, ApplicationAuthorizationRuleDenial as RuleDenial,
    ApplicationFieldPresence, ApplicationSchemaMember,
    WorthQueryPortableApplicationAuthorizationFieldConstraintParts as ConstraintParts,
    WorthQueryPortableApplicationAuthorizationPathParts as PathParts,
    WorthQueryPortableApplicationAuthorizationPredicateParts as PredicateParts,
};
use super::super::ClosureIndex;
use crate::archive_decode_authority::ARCHIVE_DECODE;

fn aspect(aspect: &str, absence: AbsenceLaw) -> ApplicationSchemaMember {
    ApplicationSchemaMember::Aspect {
        absence,
        entity: "Case".to_owned(),
        aspect: aspect.to_owned(),
        identity: AspectIdentity(0x9a1d_0001),
        revision: AspectContractRevision(1),
    }
}

fn field(
    aspect: &str,
    family: ScalarAspectType,
    presence: ApplicationFieldPresence,
) -> ApplicationSchemaMember {
    ApplicationSchemaMember::Field {
        content_format: None,
        entity: "Case".to_owned(),
        aspect: aspect.to_owned(),
        field: "Level".to_owned(),
        presence,
        scalar_family: family,
        canonical: None,
        value_type: "rebac.level".to_owned(),
        unit: None,
        frame: None,
        writable: true,
        equality_queryable: true,
        unique: false,
    }
}

/// `Base` is always present; `Escalation` is `Option<A>` (an extension member
/// or an `Exclusive<..>` case lowers the same way); `Seeded` is defaulted.
fn members() -> Vec<ApplicationSchemaMember> {
    let required = ApplicationFieldPresence::Required;
    vec![
        aspect("Base", AbsenceLaw::Required),
        field("Base", ScalarAspectType::UInt64, required),
        aspect("Escalation", AbsenceLaw::Optional),
        field("Escalation", ScalarAspectType::UInt64, required),
        aspect("Seeded", AbsenceLaw::Defaulted),
        field("Seeded", ScalarAspectType::UInt64, required),
        aspect("Money", AbsenceLaw::Required),
        field("Money", ScalarAspectType::Decimal, required),
    ]
}

fn path(
    effect: Effect,
    aspect: &str,
    principal_aspect: Option<&str>,
) -> ApplicationAuthorizationPath {
    let predicate = |aspect: &str| {
        ApplicationAuthorizationPredicate::from_untrusted_parts(
            ARCHIVE_DECODE,
            PredicateParts {
                traversal_ordinal: 0,
                entity: "Case".into(),
                aspect: aspect.into(),
                field: "Level".into(),
                value: AspectValue::UInt64(1),
                comparison: Comparison::Equal,
            },
        )
    };
    let constraint = principal_aspect.map(|principal_aspect| {
        ApplicationAuthorizationFieldConstraint::from_untrusted_parts(
            ARCHIVE_DECODE,
            ConstraintParts {
                traversal_ordinal: 0,
                entity: "Case".into(),
                aspect: "Base".into(),
                field: "Level".into(),
                comparison: Comparison::Equal,
                principal_aspect: principal_aspect.into(),
                principal_field: "Level".into(),
            },
        )
    });
    ApplicationAuthorizationPath::from_untrusted_parts(
        ARCHIVE_DECODE,
        PathParts {
            effect,
            principal_entity: "Case".into(),
            scope_entity: "Case".into(),
            traversals: Vec::new(),
            predicates: vec![predicate(aspect)],
            field_constraints: constraint.into_iter().collect(),
        },
    )
}

#[test]
fn a_deny_path_reads_only_required_fields_of_required_aspects() {
    let members = members();
    let closure = ClosureIndex::new(&members);
    assert_eq!(
        closure.deny_reads_absent_value(&path(Effect::Deny, "Base", None)),
        None
    );
    for (aspect, why) in [
        ("Escalation", "its aspect can be absent"),
        ("Seeded", "its aspect can be absent"),
    ] {
        assert_eq!(
            closure.deny_reads_absent_value(&path(Effect::Deny, aspect, None)),
            Some(RuleDenial::DenyConditionOnOptionalField {
                path: "Case".to_owned(),
                condition: format!("Case.{aspect}.Level ({why})"),
            })
        );
        assert!(!closure.path_conditions_are_closed(&path(Effect::Deny, aspect, None), &["Case"]));
        // An allow path over the same value fails closed on absence and is admitted.
        assert_eq!(
            closure.deny_reads_absent_value(&path(Effect::Allow, aspect, None)),
            None
        );
    }
    // The principal side of a constraint is read too.
    assert!(matches!(
        closure.deny_reads_absent_value(&path(Effect::Deny, "Base", Some("Escalation"))),
        Some(RuleDenial::DenyConditionOnOptionalField { condition, .. })
            if condition == "Case.Escalation.Level (its aspect can be absent)"
    ));
}

#[test]
fn an_optional_field_of_a_required_aspect_is_refused_in_a_deny_path() {
    let mut members = members();
    members.push(field(
        "Base",
        ScalarAspectType::UInt64,
        ApplicationFieldPresence::Optional,
    ));
    members.remove(1);
    let closure = ClosureIndex::new(&members);
    assert!(matches!(
        closure.deny_reads_absent_value(&path(Effect::Deny, "Base", None)),
        Some(RuleDenial::DenyConditionOnOptionalField { condition, .. })
            if condition == "Case.Base.Level (the field can be absent)"
    ));
}

#[test]
fn ordered_comparisons_close_only_on_order_faithful_families() {
    let members = members();
    let closure = ClosureIndex::new(&members);
    assert!(closure.comparison_is_faithful(Comparison::AtMost, "Case", "Base", "Level"));
    assert!(!closure.comparison_is_faithful(Comparison::AtMost, "Case", "Money", "Level"));
    assert!(!closure.comparison_is_faithful(Comparison::AtLeast, "Case", "Money", "Level"));
    // `ne` lowers to the strict pair, which is sound on every family.
    assert!(closure.comparison_is_faithful(Comparison::StrictlyLess, "Case", "Money", "Level"));
    assert!(closure.comparison_is_faithful(Comparison::Equal, "Case", "Money", "Level"));
}
