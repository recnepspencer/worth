//! What the host says a branch owes when it is asked to change program.

use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_declaration::facade::application_program::{
    ApplicationSemanticChangeKind, ApplicationSemanticDiffDenial, ApplicationSemanticFamily,
};
use worth_query_declaration::facade::application_schema::ApplicationInvariantScopeTarget;

use super::super::program_support_fixture::{
    installed_support_schema, validated, AuditProgram, BoundedProgram, CompleteProgram, AUDIT_RULE,
    BOUNDED_RULE,
};
use super::{WorthQueryProgramAdoptionRequirementsDenial, WorthQueryProgramCustodyInventoryKind};
use crate::application_program::{
    WorthQueryProgramSupportAdmission, WorthQueryProgramSupportRoster,
};
use crate::facade::WorthQueryInstalledApplicationSchema;

struct AdoptionFixture {
    installed_schema:
        WorthQueryInstalledApplicationSchema<super::super::program_support_fixture::SupportSchema>,
    roster: WorthQueryProgramSupportRoster<super::super::program_support_fixture::SupportSchema>,
    bounded: ApplicationProgramRevision,
    complete: ApplicationProgramRevision,
}

/// A host that supports a one-rule program and the two-rule program a branch
/// could move to, built through the real admission path.
fn fixture() -> AdoptionFixture {
    let installed_schema = installed_support_schema();
    let bounded = validated::<BoundedProgram>();
    let complete = validated::<CompleteProgram>();
    let bounded_revision = bounded.revision().clone();
    let complete_revision = complete.revision().clone();
    let roster = WorthQueryProgramSupportAdmission::for_installed_schema(&installed_schema)
        .support(&bounded)
        .expect("the bounded program fits the installed contracts")
        .support(&complete)
        .expect("the complete program fits the installed contracts")
        .close()
        .expect("the complete program owns every installed rule");
    AdoptionFixture {
        installed_schema,
        roster,
        bounded: bounded_revision,
        complete: complete_revision,
    }
}

#[test]
fn a_rule_the_target_adds_arrives_with_the_scope_the_catalog_declares_for_it() {
    let fixture = fixture();

    let requirements = fixture
        .roster
        .adoption_requirements(
            &fixture.installed_schema,
            &fixture.bounded,
            &fixture.complete,
        )
        .expect("both programs are rostered on this installed schema");

    assert_eq!(requirements.source(), &fixture.bounded);
    assert_eq!(requirements.target(), &fixture.complete);
    assert_eq!(
        requirements.schema_binding(),
        &fixture.installed_schema.binding_identity()
    );
    assert!(requirements.requires_existing_state_validation());
    assert!(!requirements.requires_migration_assessment());
    assert!(!requirements.semantically_equivalent());
    assert!(requirements.semantic_diff().changes().iter().any(|change| {
        change.family() == ApplicationSemanticFamily::Rules
            && change.kind() == ApplicationSemanticChangeKind::Added
    }));

    let added = requirements.added_rules();
    assert_eq!(added.len(), 1, "only the audit rule begins governing");
    let audit = &added[0];
    assert_eq!(audit.rule().identity(), AUDIT_RULE);
    assert_eq!((audit.rule().major(), audit.rule().minor()), (1, 0));
    assert_eq!(
        audit.validation_scope(),
        [ApplicationInvariantScopeTarget::Entity(
            "SupportEntity".to_owned()
        )]
    );
    assert!(
        !added
            .iter()
            .any(|added| added.rule().identity() == BOUNDED_RULE),
        "a rule both programs declare has been governing all along"
    );
}

#[test]
fn dropping_a_rule_demands_nothing_of_the_state_that_already_exists() {
    let fixture = fixture();

    let requirements = fixture
        .roster
        .adoption_requirements(
            &fixture.installed_schema,
            &fixture.complete,
            &fixture.bounded,
        )
        .expect("both programs are rostered on this installed schema");

    assert!(requirements.added_rules().is_empty());
    assert!(!requirements.requires_existing_state_validation());
}

#[test]
fn re_adopting_the_running_program_is_an_empty_demand_and_not_a_denial() {
    let fixture = fixture();

    let requirements = fixture
        .roster
        .adoption_requirements(
            &fixture.installed_schema,
            &fixture.complete,
            &fixture.complete,
        )
        .expect("a rostered program can be named as both source and target");

    assert!(requirements.added_rules().is_empty());
    assert!(!requirements.requires_existing_state_validation());
    assert_eq!(requirements.source(), requirements.target());
    assert!(requirements.semantically_equivalent());
    assert_eq!(
        requirements.semantic_diff().equivalent_families(),
        ApplicationSemanticFamily::ALL
    );
}

#[test]
fn semantic_comparison_refuses_work_beyond_the_callers_declared_limit() {
    let fixture = fixture();

    let denial = fixture
        .roster
        .adoption_requirements_with_maximum_work(
            &fixture.installed_schema,
            &fixture.bounded,
            &fixture.complete,
            0,
        )
        .expect_err("a populated pair cannot be compared without work");

    assert!(matches!(
        denial,
        WorthQueryProgramAdoptionRequirementsDenial::SemanticComparison(
            ApplicationSemanticDiffDenial::WorkLimitExceeded {
                maximum_work_units: 0,
                consumed_work_units,
            }
        ) if consumed_work_units > 0
    ));
}

#[test]
fn requirements_compiled_twice_are_equal_and_the_direction_is_not_symmetric() {
    let fixture = fixture();

    let forward = fixture
        .roster
        .adoption_requirements(
            &fixture.installed_schema,
            &fixture.bounded,
            &fixture.complete,
        )
        .expect("both programs are rostered");
    let again = fixture
        .roster
        .adoption_requirements(
            &fixture.installed_schema,
            &fixture.bounded,
            &fixture.complete,
        )
        .expect("both programs are rostered");
    let backward = fixture
        .roster
        .adoption_requirements(
            &fixture.installed_schema,
            &fixture.complete,
            &fixture.bounded,
        )
        .expect("both programs are rostered");

    assert_eq!(forward, again);
    assert_ne!(forward, backward);
}

#[test]
fn removed_operation_requires_live_custody_inventory_without_inventing_custody() {
    let installed_schema = installed_support_schema();
    let complete = validated::<CompleteProgram>();
    let audit = validated::<AuditProgram>();
    let roster = WorthQueryProgramSupportAdmission::for_installed_schema(&installed_schema)
        .support(&complete)
        .expect("the complete source is supported")
        .support(&audit)
        .expect("the audit target is supported")
        .close()
        .expect("the pair covers the installed rules");

    let requirements = roster
        .adoption_requirements(&installed_schema, complete.revision(), audit.revision())
        .expect("the static impact can be compiled");

    assert!(requirements.requires_custody_inventory());
    assert!(requirements
        .custody_inventory_requirements()
        .iter()
        .any(|required| {
            required.kind() == WorthQueryProgramCustodyInventoryKind::OperationContinuation
        }));
    assert!(!requirements
        .custody_inventory_requirements()
        .iter()
        .any(|required| {
            required.kind() == WorthQueryProgramCustodyInventoryKind::ExternalEffectRecovery
        }));
}

#[test]
fn a_program_this_host_does_not_support_names_itself_in_the_refusal() {
    let fixture = fixture();
    let foreign = validated::<AuditProgram>().revision().clone();

    let unrostered_target = fixture
        .roster
        .adoption_requirements(&fixture.installed_schema, &fixture.bounded, &foreign)
        .expect_err("the audit program was never admitted to this roster");
    assert_eq!(
        unrostered_target,
        WorthQueryProgramAdoptionRequirementsDenial::UnrosteredTarget {
            revision: foreign.clone()
        }
    );

    let unrostered_source = fixture
        .roster
        .adoption_requirements(&fixture.installed_schema, &foreign, &fixture.complete)
        .expect_err("a branch running an unknown program declares nothing this host can compare");
    assert_eq!(
        unrostered_source,
        WorthQueryProgramAdoptionRequirementsDenial::UnrosteredSource { revision: foreign }
    );
}

#[test]
fn a_roster_admitted_against_another_installation_resolves_no_rules_here() {
    let fixture = fixture();
    let other_installation = installed_support_schema();

    let denial = fixture
        .roster
        .adoption_requirements(&other_installation, &fixture.bounded, &fixture.complete)
        .expect_err("a roster speaks only for the installation that admitted it");

    assert_eq!(
        denial,
        WorthQueryProgramAdoptionRequirementsDenial::ForeignSchemaBinding {
            roster: fixture.installed_schema.binding_identity(),
            installed: other_installation.binding_identity(),
        }
    );
}
