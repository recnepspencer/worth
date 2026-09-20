//! Every axis of declared program meaning that the canonical revision must
//! separate, and every axis it must deliberately ignore.

use super::{
    manifest_of, revision_of, AddedActionProgram, AddedFeatureProgram, ApplicationProgramAuthoring,
    BaselineProgram, ConnectedTopologyProgram, MutationSensitiveRuleProgram,
    RaisedRuleVersionProgram, RenamedBaselineProgram, RevisionSchema, UnconnectedTopologyProgram,
};

#[test]
fn a_validated_program_hands_out_its_canonical_revision() {
    let program = ApplicationProgramAuthoring::<RevisionSchema, BaselineProgram>::begin()
        .validated_program()
        .expect("the baseline program is declaration-valid");

    let revision = program.revision();
    assert_eq!(revision.as_bytes().len(), 32);
    assert_eq!(revision.to_string().len(), 64);
    assert_ne!(revision.as_bytes(), &[0_u8; 32]);
}

#[test]
fn validating_one_program_twice_mints_the_same_revision() {
    assert_eq!(
        revision_of::<BaselineProgram>(),
        revision_of::<BaselineProgram>()
    );
}

#[test]
fn a_renamed_program_type_with_identical_declared_meaning_keeps_its_revision() {
    assert_eq!(
        revision_of::<BaselineProgram>(),
        revision_of::<RenamedBaselineProgram>()
    );
}

#[test]
fn a_raised_rule_version_changes_the_revision() {
    assert_ne!(
        revision_of::<BaselineProgram>(),
        revision_of::<RaisedRuleVersionProgram>()
    );
}

#[test]
fn an_added_action_changes_the_revision() {
    assert_ne!(
        revision_of::<BaselineProgram>(),
        revision_of::<AddedActionProgram>()
    );
}

#[test]
fn action_locality_granule_has_a_named_canonical_manifest_axis() {
    let manifest = manifest_of::<BaselineProgram>();
    let action = manifest
        .records()
        .iter()
        .find(|record| record.starts_with("action|"))
        .expect("the baseline program declares one action");

    assert!(action.contains("|locality=|granule=|change="));
}

#[test]
fn an_added_feature_changes_the_revision() {
    assert_ne!(
        revision_of::<BaselineProgram>(),
        revision_of::<AddedFeatureProgram>()
    );
}

#[test]
fn a_moved_rule_execution_point_changes_the_revision() {
    assert_ne!(
        revision_of::<BaselineProgram>(),
        revision_of::<MutationSensitiveRuleProgram>()
    );
}

#[test]
fn a_rule_execution_point_reaches_the_record_as_its_named_canonical_token() {
    let committed = manifest_of::<BaselineProgram>();
    let mutation_sensitive = manifest_of::<MutationSensitiveRuleProgram>();

    assert!(committed
        .records()
        .iter()
        .any(|record| record.contains("|point=CommitBoundary|")));
    assert!(mutation_sensitive
        .records()
        .iter()
        .any(|record| record.contains("|point=MutationSensitive|")));
}

#[test]
fn a_declared_output_connection_changes_the_revision() {
    assert_ne!(
        revision_of::<UnconnectedTopologyProgram>(),
        revision_of::<ConnectedTopologyProgram>()
    );
}

#[test]
fn a_declared_output_connection_enters_the_normalized_manifest() {
    let unconnected = manifest_of::<UnconnectedTopologyProgram>();
    let connected = manifest_of::<ConnectedTopologyProgram>();

    assert!(!unconnected
        .records()
        .iter()
        .any(|record| record.starts_with("connection|")));
    assert!(connected
        .records()
        .iter()
        .any(|record| record.starts_with("connection|worth.query.tests.bounded-audit.v1|")));
}
