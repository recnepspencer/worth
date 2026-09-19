use std::collections::BTreeSet;

use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
use worth_query_declaration::facade::application_schema::ApplicationInvariantExecutionPoint;

use super::super::program_support_fixture::{
    installed_support_schema, validated, AuditProgram, BoundedProgram, RaisedRuleProgram,
    SupportSchema, AUDIT_RULE, BOUNDED_RULE,
};
use super::compatibility::{
    declared_rule_keys, first_rule_outside_catalog, first_unowned_installed_rule,
    installed_rule_keys, WorthQueryProgramRuleKey,
};

fn declared_by<Program>() -> BTreeSet<WorthQueryProgramRuleKey>
where
    Program: ApplicationProgramDefinition<SupportSchema>,
{
    declared_rule_keys(validated::<Program>().rules())
}

fn named(keys: &BTreeSet<WorthQueryProgramRuleKey>) -> Vec<String> {
    keys.iter()
        .map(WorthQueryProgramRuleKey::to_string)
        .collect()
}

#[test]
fn installed_rule_keys_name_every_invariant_the_host_compiled() {
    let installed = installed_rule_keys(&installed_support_schema());

    assert_eq!(
        named(&installed),
        vec![
            format!("{AUDIT_RULE} v1.0 at CommitBoundary"),
            format!("{BOUNDED_RULE} v1.0 at CommitBoundary"),
        ]
    );
}

#[test]
fn a_declared_rule_key_records_identity_version_and_execution_point() {
    let declared = declared_by::<BoundedProgram>();
    let rule = declared.iter().next().expect("the program declares a rule");

    assert_eq!(rule.identity(), BOUNDED_RULE);
    assert_eq!(rule.major(), 1);
    assert_eq!(rule.minor(), 0);
    assert_eq!(
        rule.execution_point(),
        ApplicationInvariantExecutionPoint::CommitBoundary
    );
}

#[test]
fn a_declared_subset_of_the_installed_catalog_names_no_rule_outside_it() {
    let installed = installed_rule_keys(&installed_support_schema());

    assert_eq!(
        first_rule_outside_catalog(&declared_by::<BoundedProgram>(), &installed),
        None
    );
}

#[test]
fn a_raised_rule_version_falls_outside_the_installed_catalog() {
    let installed = installed_rule_keys(&installed_support_schema());
    let declared = declared_by::<RaisedRuleProgram>();

    let outside = first_rule_outside_catalog(&declared, &installed)
        .expect("version two of the bounded rule was never installed");

    assert_eq!(outside.identity(), BOUNDED_RULE);
    assert_eq!(outside.major(), 2);
}

#[test]
fn an_installed_rule_no_rostered_program_declares_is_unowned() {
    let installed = installed_rule_keys(&installed_support_schema());

    let unowned = first_unowned_installed_rule(&installed, &declared_by::<BoundedProgram>())
        .expect("the audit rule is installed but undeclared here");

    assert_eq!(unowned.identity(), AUDIT_RULE);
}

#[test]
fn two_programs_whose_union_is_the_catalog_leave_no_rule_unowned() {
    let installed = installed_rule_keys(&installed_support_schema());
    let rostered = declared_by::<BoundedProgram>()
        .union(&declared_by::<AuditProgram>())
        .cloned()
        .collect();

    assert_eq!(first_unowned_installed_rule(&installed, &rostered), None);
}
