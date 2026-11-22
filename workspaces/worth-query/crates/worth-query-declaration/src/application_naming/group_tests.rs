//! Group scheme declarations bind one closed continuity rule to declared roles.
use crate::application_naming::{
    ApplicationGroupContinuityRule as Rule, ApplicationGroupMemberRole as Role,
    ApplicationGroupNameSchemeBuilder as Builder, ApplicationGroupSchemeDenial as Denial,
    ApplicationNameSchemeDenial, ApplicationNameSubjectKind,
};

struct Cad;
struct Space;

fn builder(rule: Rule) -> Builder<Cad, Space> {
    Builder::new("cad.space", "Space", rule).role_order(&["wall", "producer", "void"])
}

#[test]
fn continuity_rules_are_closed_with_stable_canonical_tags() {
    let rules = [Rule::Anchor, Rule::Boundary, Rule::ProducerRole];
    let tags: Vec<_> = rules.iter().map(|rule| rule.canonical_tag()).collect();
    let names: Vec<_> = rules.iter().map(|rule| rule.canonical_name()).collect();
    assert_eq!(tags, [1, 2, 3]);
    assert_eq!(names, ["anchor", "boundary", "producer-role"]);
    assert!(Rule::Anchor < Rule::Boundary && Rule::Boundary < Rule::ProducerRole);
}

#[test]
fn group_scheme_binds_rule_and_maps_member_roles() {
    let scheme = builder(Rule::Boundary)
        .boundary_role("wall")
        .anchor_role("producer")
        .build()
        .unwrap();
    let erased = scheme.erased();
    assert_eq!(erased.continuity_rule(), Rule::Boundary);
    assert_eq!(
        erased.name_scheme().subject_kind(),
        ApplicationNameSubjectKind::Group
    );
    assert_eq!(erased.name_scheme().subject_name(), "Space");
    assert_eq!(erased.member_role("wall"), Role::Boundary);
    assert_eq!(erased.member_role("producer"), Role::Anchor);
    assert_eq!(erased.member_role("void"), Role::Ordinary);
    assert_eq!(scheme.clone().into_erased(), *erased);
    let anchor_only = builder(Rule::Anchor).build().unwrap();
    assert_eq!(anchor_only.erased().anchor_role(), None);
}

#[test]
fn group_scheme_refuses_missing_undeclared_and_ambiguous_roles() {
    assert_eq!(
        builder(Rule::Boundary).anchor_role("producer").build(),
        Err(Denial::MissingContinuityRole {
            scheme_id: "cad.space".into(),
            rule: Rule::Boundary,
        })
    );
    assert_eq!(
        builder(Rule::ProducerRole).boundary_role("wall").build(),
        Err(Denial::MissingContinuityRole {
            scheme_id: "cad.space".into(),
            rule: Rule::ProducerRole,
        })
    );
    assert_eq!(
        builder(Rule::ProducerRole).anchor_role("owner").build(),
        Err(Denial::UndeclaredContinuityRole {
            scheme_id: "cad.space".into(),
            role: "owner".into(),
        })
    );
    assert_eq!(
        builder(Rule::Boundary)
            .anchor_role("wall")
            .boundary_role("wall")
            .build(),
        Err(Denial::AmbiguousContinuityRole {
            scheme_id: "cad.space".into(),
            role: "wall".into(),
        })
    );
    assert_eq!(
        Builder::<Cad, Space>::new("cad.space", "Space", Rule::Anchor).build(),
        Err(Denial::Scheme(
            ApplicationNameSchemeDenial::EmptyRoleOrder {
                scheme_id: "cad.space".into(),
            }
        ))
    );
}
