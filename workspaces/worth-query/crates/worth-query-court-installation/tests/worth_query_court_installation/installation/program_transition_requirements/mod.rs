//! Comparison observes both actual installations, rather than re-roster A on B.
mod fixture;
mod retained_rule;
use fixture::*;
use worth_query_declaration::facade::{application_program::*, application_schema::*};
use worth_query_installation::facade::WorthQueryProgramAdoptionRequirementsDenial as Denial;

#[test]
fn separate_installed_programs_compare_prior_meaning_and_resolve_target_rule_scopes() {
    let p = pair();
    assert!(p.source_roster.entry(&p.target).is_none());
    assert!(p.target_roster.entry(&p.source).is_none());
    let requirements = p
        .target_roster
        .adoption_requirements_from_installed_predecessor(
            &p.after,
            &p.source_roster,
            &p.before,
            &p.source,
            &p.target,
            65_536,
        )
        .expect("each program belongs to its own separately admitted schema");
    assert_eq!(requirements.source(), &p.source);
    assert_eq!(requirements.target(), &p.target);
    assert_eq!(requirements.schema_binding(), &p.after.binding_identity());
    assert!(!requirements.semantically_equivalent());
    assert!(requirements.requires_existing_state_validation());
    let added = requirements.added_rules();
    assert_eq!(added.len(), 1);
    assert_eq!(added[0].rule().identity(), "TargetRule");
    assert_eq!(
        added[0].validation_scope(),
        [ApplicationInvariantScopeTarget::Entity("Record".into())]
    );
    assert!(requirements
        .semantic_diff()
        .changes()
        .iter()
        .any(|change| change.family() == ApplicationSemanticFamily::Rules
            && change.kind() == ApplicationSemanticChangeKind::Added));
    let reverse = p
        .source_roster
        .adoption_requirements_from_installed_predecessor(
            &p.before,
            &p.target_roster,
            &p.after,
            &p.target,
            &p.source,
            65_536,
        )
        .expect("the reverse direction drops the target rule");
    assert!(!reverse.semantically_equivalent());
    assert!(reverse.added_rules().is_empty());
    assert!(!reverse.requires_existing_state_validation());
    assert_ne!(requirements, reverse);
}

#[test]
fn single_installation_adoption_uses_the_same_installed_pair_comparison() {
    let p = pair();
    let ordinary = p
        .target_roster
        .adoption_requirements(&p.after, &p.current, &p.target)
        .unwrap();
    let paired = p
        .target_roster
        .adoption_requirements_from_installed_predecessor(
            &p.after,
            &p.target_roster,
            &p.after,
            &p.current,
            &p.target,
            65_536,
        )
        .unwrap();
    assert_eq!(ordinary, paired);
    assert!(!paired.semantically_equivalent());
    assert_eq!(paired.source(), &p.current);
    assert_eq!(paired.target(), &p.target);
    assert_eq!(paired.added_rules().len(), 1);
    assert_eq!(paired.added_rules()[0].rule().identity(), "TargetRule");
    assert_eq!(
        paired.added_rules()[0].validation_scope(),
        [ApplicationInvariantScopeTarget::Entity("Record".into())]
    );
}

#[test]
fn installed_pair_comparison_refuses_either_foreign_binding_and_either_unrostered_revision() {
    let p = pair();
    let foreign_before = install::<Before>();
    let foreign_after = install::<After>();
    let source_denial = p
        .target_roster
        .adoption_requirements_from_installed_predecessor(
            &p.after,
            &p.source_roster,
            &foreign_before,
            &p.source,
            &p.target,
            65_536,
        )
        .unwrap_err();
    assert_eq!(
        source_denial,
        Denial::ForeignSchemaBinding {
            roster: p.before.binding_identity(),
            installed: foreign_before.binding_identity(),
        }
    );
    let target_denial = p
        .target_roster
        .adoption_requirements_from_installed_predecessor(
            &foreign_after,
            &p.source_roster,
            &p.before,
            &p.source,
            &p.target,
            65_536,
        )
        .unwrap_err();
    assert_eq!(
        target_denial,
        Denial::ForeignSchemaBinding {
            roster: p.after.binding_identity(),
            installed: foreign_after.binding_identity(),
        }
    );
    assert_eq!(
        p.target_roster
            .adoption_requirements_from_installed_predecessor(
                &p.after,
                &p.source_roster,
                &p.before,
                &p.target,
                &p.target,
                65_536,
            )
            .unwrap_err(),
        Denial::UnrosteredSource { revision: p.target }
    );
    assert_eq!(
        p.target_roster
            .adoption_requirements_from_installed_predecessor(
                &p.after,
                &p.source_roster,
                &p.before,
                &p.source,
                &p.source,
                65_536,
            )
            .unwrap_err(),
        Denial::UnrosteredTarget { revision: p.source }
    );
}

#[test]
fn installed_pair_comparison_preserves_the_declared_semantic_comparison_limit() {
    let p = pair();
    let denial = p
        .target_roster
        .adoption_requirements_from_installed_predecessor(
            &p.after,
            &p.source_roster,
            &p.before,
            &p.source,
            &p.target,
            0,
        )
        .unwrap_err();
    assert!(
        matches!(denial, Denial::SemanticComparison(ApplicationSemanticDiffDenial::WorkLimitExceeded {
        maximum_work_units: 0, consumed_work_units,
    }) if consumed_work_units > 0)
    );
}
