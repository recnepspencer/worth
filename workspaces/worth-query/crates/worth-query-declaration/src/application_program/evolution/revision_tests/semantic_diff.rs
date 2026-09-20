use super::{
    description_of, AddedActionProgram, BaselineProgram, ConnectedTopologyProgram,
    RaisedRuleVersionProgram, RenamedBaselineProgram, UnconnectedTopologyProgram,
};
use crate::application_program::{
    ApplicationSemanticChangeKind, ApplicationSemanticDiff, ApplicationSemanticDiffDenial,
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
