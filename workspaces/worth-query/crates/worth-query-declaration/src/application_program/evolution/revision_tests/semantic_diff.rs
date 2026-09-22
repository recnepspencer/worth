use super::{
    description_of, AddedActionProgram, BaselineProgram, ConnectedTopologyProgram,
    MultipleRuleVersionsProgram, RaisedRuleVersionProgram, RenamedBaselineProgram,
    ShiftedRuleVersionsProgram, UnconnectedTopologyProgram,
};
use crate::application_program::{
    ApplicationConnectionDeclaration, ApplicationSemanticChangeKind,
    ApplicationSemanticDescription, ApplicationSemanticDiff, ApplicationSemanticDiffDenial,
    ApplicationSemanticFamily,
};

const TEST_WORK_LIMIT: usize = 1_024;

#[test]
fn rust_type_renames_leave_every_semantic_family_equivalent() {
    let diff = compare::<BaselineProgram, RenamedBaselineProgram>(TEST_WORK_LIMIT)
        .expect("identical authored meaning fits the comparison budget");

    assert!(diff.changes().is_empty());
    assert_eq!(diff.equivalent_families(), ApplicationSemanticFamily::ALL);
    assert!(diff.comparison_work_units() > 0);
}

#[test]
fn rule_contract_changes_are_typed_without_prejudging_live_state() {
    let diff = compare::<BaselineProgram, RaisedRuleVersionProgram>(TEST_WORK_LIMIT)
        .expect("the rule comparison fits the work budget");

    let change = diff
        .changes()
        .iter()
        .find(|change| change.family() == ApplicationSemanticFamily::Rules)
        .expect("the rule family records the version change");
    assert_eq!(change.kind(), ApplicationSemanticChangeKind::Changed);
    assert!(change.source_meaning().is_some());
    assert!(change.target_meaning().is_some());
    assert_ne!(change.source_meaning(), change.target_meaning());
    assert!(change.migration_assessment_requirement().is_none());
}

#[test]
fn multiple_legal_rule_versions_are_compared_without_overwriting_an_occurrence() {
    let diff = compare::<MultipleRuleVersionsProgram, ShiftedRuleVersionsProgram>(TEST_WORK_LIMIT)
        .expect("all rule occurrences fit the comparison budget");

    let rule_changes = diff
        .changes()
        .iter()
        .filter(|change| change.family() == ApplicationSemanticFamily::Rules)
        .collect::<Vec<_>>();
    assert_eq!(rule_changes.len(), 1);
    assert_eq!(
        rule_changes[0].kind(),
        ApplicationSemanticChangeKind::Changed
    );
    assert!(rule_changes[0]
        .source_meaning()
        .is_some_and(|meaning| meaning.contains("major=1:1")));
    assert!(rule_changes[0]
        .target_meaning()
        .is_some_and(|meaning| meaning.contains("major=1:2")));
}

#[test]
fn scoped_connection_change_survives_a_same_named_sibling_and_requires_assessment() {
    let source = connection_description(vec![
        connection("shared|route", "west:one", "sink", "old|port"),
        connection("shared|route", "east", "sink", "stable"),
    ]);
    let target = connection_description(vec![
        connection("shared|route", "west:one", "sink", "new:port"),
        connection("shared|route", "east", "sink", "stable"),
    ]);

    let diff = ApplicationSemanticDiff::compare(&source, &target, TEST_WORK_LIMIT)
        .expect("the scoped connection comparison fits the work budget");
    let connection_changes = diff
        .changes()
        .iter()
        .filter(|change| change.family() == ApplicationSemanticFamily::Connections)
        .collect::<Vec<_>>();

    assert_eq!(connection_changes.len(), 1);
    assert_eq!(
        connection_changes[0].kind(),
        ApplicationSemanticChangeKind::Changed
    );
    assert!(connection_changes[0]
        .source_meaning()
        .is_some_and(|meaning| meaning.contains("source-port=8:old|port")));
    assert!(connection_changes[0]
        .target_meaning()
        .is_some_and(|meaning| meaning.contains("source-port=8:new:port")));
    assert!(connection_changes[0]
        .migration_assessment_requirement()
        .is_some());
    assert_eq!(
        source
            .facts()
            .iter()
            .filter(|fact| fact.family() == ApplicationSemanticFamily::Connections)
            .map(|fact| fact.subject())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        2,
        "same-named connections in different legal scopes need distinct subjects"
    );
}

#[test]
fn additive_operation_and_connection_meaning_stay_in_their_own_families() {
    let action = compare::<BaselineProgram, AddedActionProgram>(TEST_WORK_LIMIT)
        .expect("the action comparison fits the budget");
    assert!(action.changes().iter().any(|change| {
        change.family() == ApplicationSemanticFamily::Operations
            && change.kind() == ApplicationSemanticChangeKind::Added
    }));
    assert!(!action
        .changes()
        .iter()
        .any(|change| change.migration_assessment_requirement().is_some()));

    let connection =
        compare::<UnconnectedTopologyProgram, ConnectedTopologyProgram>(TEST_WORK_LIMIT)
            .expect("the connection comparison fits the budget");
    assert!(connection.changes().iter().any(|change| {
        change.family() == ApplicationSemanticFamily::Connections
            && change.kind() == ApplicationSemanticChangeKind::Added
    }));
}

#[test]
fn comparison_work_is_charged_and_denied_before_a_partial_diff_escapes() {
    let denial = compare::<BaselineProgram, AddedActionProgram>(0)
        .expect_err("zero work cannot compare two populated descriptions");

    assert!(matches!(
        denial,
        ApplicationSemanticDiffDenial::WorkLimitExceeded {
            maximum_work_units: 0,
            consumed_work_units,
        } if consumed_work_units > 0
    ));
}

fn compare<Source, Target>(
    maximum_work_units: usize,
) -> Result<ApplicationSemanticDiff, ApplicationSemanticDiffDenial>
where
    Source: super::ApplicationProgramDefinition<super::RevisionSchema>,
    Target: super::ApplicationProgramDefinition<super::RevisionSchema>,
{
    ApplicationSemanticDiff::compare(
        &description_of::<Source>(),
        &description_of::<Target>(),
        maximum_work_units,
    )
}

fn connection_description(
    connections: Vec<ApplicationConnectionDeclaration>,
) -> ApplicationSemanticDescription {
    ApplicationSemanticDescription::from_validated_parts(
        super::revision_of::<BaselineProgram>(),
        &[],
        &[],
        &connections,
        &[],
    )
}

fn connection(
    identity: &'static str,
    source_instance: &'static str,
    target_instance: &'static str,
    source_port: &'static str,
) -> ApplicationConnectionDeclaration {
    ApplicationConnectionDeclaration::new(
        identity,
        source_instance,
        "source|feature",
        source_port,
        target_instance,
        "target:feature",
        "target|port",
        false,
        true,
    )
}
