use super::*;

#[test]
fn finite_typed_program_validates() {
    let program = validate_application_program::<Schema, Valid>().expect("program is complete");
    assert_eq!(program.features().len(), 2);
    assert_eq!(program.connections().len(), 1);
    assert_eq!(program.rules().len(), 1);
    assert_eq!(program.inventories().len(), 1);
    assert!(validate_application_program::<Schema, NoRules>().is_ok());
    assert!(validate_application_program::<Schema, PlannedUnavailable>().is_ok());
    let unavailable_rule =
        validate_application_program::<Schema, PlannedUnavailableRule>().unwrap();
    assert!(unavailable_rule
        .rules()
        .iter()
        .all(|rule| rule.posture() == ApplicationProgramRulePosture::Unavailable));
    assert_eq!(
        denial::<AvailableOnlyByUnavailableRule>().kind(),
        ApplicationProgramValidationDenialKind::OrphanFeature,
    );
    assert_eq!(
        denial::<DuplicateRulePostures>().kind(),
        ApplicationProgramValidationDenialKind::DuplicateRule,
    );
    assert_eq!(
        denial::<AvailableRuleOnUnavailableFeature>().kind(),
        ApplicationProgramValidationDenialKind::AvailabilityMismatch,
    );
}

#[test]
fn duplicate_members_are_denied_with_exact_subjects() {
    let duplicate_feature = denial::<DuplicateFeature>();
    assert_eq!(
        duplicate_feature.kind(),
        ApplicationProgramValidationDenialKind::DuplicateFeature
    );
    assert_eq!(duplicate_feature.subject(), "source");
    let duplicate_connection = denial::<DuplicateConnection>();
    assert_eq!(
        duplicate_connection.kind(),
        ApplicationProgramValidationDenialKind::DuplicateConnection
    );
    assert_eq!(duplicate_connection.subject(), "source-to-target");
    let duplicate_inventory = denial::<DuplicateInventory>();
    assert_eq!(
        duplicate_inventory.kind(),
        ApplicationProgramValidationDenialKind::DuplicateInventory
    );
    assert_eq!(duplicate_inventory.subject(), "complete");
}

#[test]
fn empty_inventory_and_orphan_feature_are_denied() {
    assert_eq!(
        denial::<EmptyInventory>().kind(),
        ApplicationProgramValidationDenialKind::EmptyInventory
    );
    assert_eq!(
        denial::<OrphanFeature>().kind(),
        ApplicationProgramValidationDenialKind::OrphanFeature
    );
}

#[test]
fn unavailable_dependency_cannot_feed_available_work() {
    let denial = denial::<UnavailableDependency>();
    assert_eq!(
        denial.kind(),
        ApplicationProgramValidationDenialKind::AvailabilityMismatch
    );
    assert_eq!(denial.subject(), "source-to-target");
}

#[test]
fn connection_role_matches_target_posture() {
    for program_denial in [
        denial::<UnavailableTargetWithDependent>(),
        denial::<AvailableTargetWithUnavailable>(),
    ] {
        assert_eq!(
            program_denial.kind(),
            ApplicationProgramValidationDenialKind::AvailabilityMismatch
        );
        assert_eq!(program_denial.subject(), "source-to-target");
    }
}

#[test]
fn fan_in_is_rejected_until_query_can_settle_one_coherent_basis() {
    assert_eq!(
        denial::<FanIn>().kind(),
        ApplicationProgramValidationDenialKind::UnsupportedFanIn
    );
}

#[test]
fn missing_required_input_is_denied() {
    assert_eq!(
        denial::<MissingInput>().kind(),
        ApplicationProgramValidationDenialKind::MissingRequiredInput
    );
}

#[test]
fn same_name_foreign_types_do_not_enter_the_graph() {
    impostors::same_name_foreign_types_do_not_enter_the_graph();
}

#[test]
fn duplicate_inventory_output_is_denied() {
    impostors::duplicate_inventory_output_is_denied();
}

#[test]
fn cycles_and_incomplete_or_unknown_inventories_are_denied() {
    let cycle = denial::<CycleProgram>();
    assert_eq!(cycle.kind(), ApplicationProgramValidationDenialKind::Cycle);
    assert_eq!(cycle.subject(), "source");

    let incomplete = denial::<IncompleteInventoryProgram>();
    assert_eq!(
        incomplete.kind(),
        ApplicationProgramValidationDenialKind::IncompleteInventory
    );
    assert_eq!(incomplete.subject(), "target.second-result");

    impostors::foreign_and_unknown_inventory_outputs_are_denied();
}
