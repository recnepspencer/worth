//! Which rosters a two-rule host will admit at all.
//!
//! A host installs rule contracts; a roster declares which program reads which
//! of them as law. Installation refuses a roster that leaves an installed rule
//! unclaimed, and refuses a program that claims a rule this host never
//! installed. Both refusals happen through the ordinary public install entry.

use worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationDenial;
use worth_query_host::facade::declaration::application_program::ApplicationProgramDefinition;
use worth_query_host::facade::domain::WorthQueryProgramSupportDenial;

use crate::bounded_dimension_model::host::{
    publish_first_program_alone, publish_with_foreign_rule_rostered,
};
use crate::bounded_dimension_model::programs::ForeignRuleDimensionProgram;
use crate::bounded_dimension_model::schema::BoundedDimensionSchema;

#[test]
fn a_roster_leaving_an_installed_rule_unclaimed_is_refused() {
    let denial = publish_first_program_alone()
        .err()
        .expect("a roster that claims only one of two installed rules must be refused");
    let WorthQueryInMemoryApplicationDenial::Program(program) = denial else {
        panic!("the refusal must be a program installation refusal: {denial:?}");
    };
    let Some(WorthQueryProgramSupportDenial::UndeclaredInstalledRule { rule }) =
        program.support_denial()
    else {
        panic!("the refusal must be an unclaimed installed rule: {program:?}");
    };
    assert_eq!(
        (rule.identity(), rule.major(), rule.minor()),
        ("bounded-dimension-v2", 1, 0),
        "the refusal must name the installed rule no rostered program reads as law"
    );
}

#[test]
fn a_program_claiming_an_uninstalled_rule_is_refused() {
    let denial = publish_with_foreign_rule_rostered()
        .err()
        .expect("a program claiming a rule this host never installed must be refused");
    let WorthQueryInMemoryApplicationDenial::Program(program) = denial else {
        panic!("the refusal must be a program installation refusal: {denial:?}");
    };
    let Some(WorthQueryProgramSupportDenial::UnsupportedRuleContract {
        program: claimant,
        rule,
    }) = program.support_denial()
    else {
        panic!("the refusal must be an unsupported rule contract: {program:?}");
    };
    assert_eq!(
        (rule.identity(), rule.major(), rule.minor()),
        ("bounded-dimension-v3", 1, 0),
        "the refusal must name the rule contract this host cannot support"
    );
    assert_eq!(
        claimant,
        &<ForeignRuleDimensionProgram as ApplicationProgramDefinition<BoundedDimensionSchema>>::IDENTITY,
        "the refusal must name the program that claimed it"
    );
}
