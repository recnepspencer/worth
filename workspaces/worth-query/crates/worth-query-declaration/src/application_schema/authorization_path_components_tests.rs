//! Equality-only paths keep their legacy canonical components; comparisons and
//! principal-field constraints add components only when present.
use worth_foundational::facade::AspectValue;

use super::authorization_path_components::application_authorization_path_canonical_components as components;
use super::authorization_policy::{
    ApplicationAuthorizationComparison as Comparison, ApplicationAuthorizationFieldConstraint,
    ApplicationAuthorizationPath, ApplicationAuthorizationPathEffect,
    ApplicationAuthorizationPredicate,
    WorthQueryPortableApplicationAuthorizationFieldConstraintParts,
    WorthQueryPortableApplicationAuthorizationPathParts,
    WorthQueryPortableApplicationAuthorizationPredicateParts,
};
use crate::archive_decode_authority::ARCHIVE_DECODE;

fn path(comparison: Comparison, constrained: bool) -> ApplicationAuthorizationPath {
    let predicate = ApplicationAuthorizationPredicate::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableApplicationAuthorizationPredicateParts {
            traversal_ordinal: 0,
            entity: "User".into(),
            aspect: "UserFacts".into(),
            field: "Level".into(),
            value: AspectValue::UInt64(2),
            comparison,
        },
    );
    let constraint = ApplicationAuthorizationFieldConstraint::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableApplicationAuthorizationFieldConstraintParts {
            traversal_ordinal: 0,
            entity: "User".into(),
            aspect: "UserFacts".into(),
            field: "Level".into(),
            comparison: Comparison::StrictlyLess,
            principal_aspect: "UserFacts".into(),
            principal_field: "Floor".into(),
        },
    );
    ApplicationAuthorizationPath::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableApplicationAuthorizationPathParts {
            effect: ApplicationAuthorizationPathEffect::Allow,
            principal_entity: "User".into(),
            scope_entity: "User".into(),
            traversals: Vec::new(),
            predicates: vec![predicate],
            field_constraints: if constrained {
                vec![constraint]
            } else {
                Vec::new()
            },
        },
    )
}

fn loci(path: &ApplicationAuthorizationPath) -> Vec<String> {
    components(path)
        .iter()
        .map(|component| component.locus().to_owned())
        .collect()
}

#[test]
fn equality_paths_keep_exactly_the_legacy_component_loci() {
    assert_eq!(
        loci(&path(Comparison::Equal, false)),
        [
            "effect",
            "principal",
            "scope",
            "traversal-count",
            "predicate-count",
            "predicate.0.traversal-ordinal",
            "predicate.0.entity",
            "predicate.0.aspect",
            "predicate.0.field",
            "predicate.0.value",
        ]
    );
}

#[test]
fn comparisons_and_constraints_change_the_identity_only_when_present() {
    let legacy = components(&path(Comparison::Equal, false));
    let ordered = components(&path(Comparison::AtLeast, false));
    let constrained = components(&path(Comparison::Equal, true));
    assert_ne!(legacy, ordered);
    assert_ne!(legacy, constrained);
    assert!(loci(&path(Comparison::AtLeast, false)).contains(&"predicate.0.comparison".to_owned()));
    let constrained_loci = loci(&path(Comparison::Equal, true));
    assert!(constrained_loci.contains(&"field-constraint-count".to_owned()));
    assert!(constrained_loci.contains(&"field-constraint.0.principal-field".to_owned()));
    assert_eq!(legacy.len(), 10);
    assert_eq!(constrained.len(), 18);
}
