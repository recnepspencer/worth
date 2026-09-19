//! How one support refusal crosses into the installation boundary: the subject
//! every caller has always read, and the structured reason that must survive
//! the crossing.

use super::program_support_fixture::{
    installed_support_schema, validated, BoundedProgram, CompleteProgram, FeaturelessProgram,
    RaisedRuleProgram, UninstalledActionProgram, AUDIT_RULE, BOUNDED_RULE, UNKNOWN_OPERATION,
};
use super::{WorthQueryProgramSupportAdmission, WorthQueryProgramSupportDenial};
use crate::application_program::{
    install_rostered_application_program, WorthQueryApplicationProgramInstallationDenial,
};

#[test]
fn a_program_that_governs_nothing_crosses_as_its_own_identity() {
    let installed_schema = installed_support_schema();
    let featureless = validated::<FeaturelessProgram>();
    let denial = WorthQueryProgramSupportAdmission::for_installed_schema(&installed_schema)
        .support(&featureless)
        .err()
        .expect("a program declaring no feature governs nothing");

    let crossed = crossed(&denial);

    assert_eq!(crossed.subject(), featureless.identity().as_str());
    assert_eq!(crossed.support_denial(), Some(&denial));
}

#[test]
fn an_uninstalled_rule_contract_crosses_as_the_rule_identity() {
    let installed_schema = installed_support_schema();
    let raised = validated::<RaisedRuleProgram>();
    let denial = WorthQueryProgramSupportAdmission::for_installed_schema(&installed_schema)
        .support(&raised)
        .err()
        .expect("version two of the bounded rule is not installed");

    let crossed = crossed(&denial);

    assert_eq!(crossed.subject(), BOUNDED_RULE);
    assert_eq!(crossed.support_denial(), Some(&denial));
}

#[test]
fn an_uninstalled_action_crosses_as_the_binding_it_acted_through() {
    let installed_schema = installed_support_schema();
    let acting = validated::<UninstalledActionProgram>();
    let denial = WorthQueryProgramSupportAdmission::for_installed_schema(&installed_schema)
        .support(&acting)
        .err()
        .expect("the retire operation is not installed");

    let crossed = crossed(&denial);

    assert_eq!(crossed.subject(), UNKNOWN_OPERATION);
    assert_eq!(crossed.support_denial(), Some(&denial));
}

#[test]
fn an_unowned_installed_rule_crosses_as_its_named_prose() {
    let installed_schema = installed_support_schema();
    let bounded = validated::<BoundedProgram>();
    let denial = WorthQueryProgramSupportAdmission::for_installed_schema(&installed_schema)
        .support(&bounded)
        .expect("the bounded program fits the installed contracts")
        .close()
        .err()
        .expect("the audit rule would be enforced with no declaring owner");

    let crossed = crossed(&denial);

    assert_eq!(
        crossed.subject(),
        format!("undeclared installed rule: {AUDIT_RULE}")
    );
    assert_eq!(crossed.support_denial(), Some(&denial));
}

#[test]
fn a_duplicate_roster_admission_crosses_as_its_rendered_refusal() {
    let installed_schema = installed_support_schema();
    let complete = validated::<CompleteProgram>();
    let denial = WorthQueryProgramSupportAdmission::for_installed_schema(&installed_schema)
        .support(&complete)
        .expect("the complete program fits the installed contracts")
        .support(&complete)
        .err()
        .expect("one revision cannot hold two roster entries");

    let crossed = crossed(&denial);

    assert_eq!(
        crossed.subject(),
        format!(
            "program {} already rostered at revision {}",
            complete.identity().as_str(),
            complete.revision()
        )
    );
    assert_eq!(crossed.support_denial(), Some(&denial));
}

#[test]
fn an_unrostered_program_crosses_as_its_rendered_refusal() {
    let installed_schema = installed_support_schema();
    let complete = validated::<CompleteProgram>();
    let roster = WorthQueryProgramSupportAdmission::for_installed_schema(&installed_schema)
        .support(&complete)
        .expect("the complete program owns the whole catalog")
        .close()
        .expect("the complete program leaves no rule unowned");
    let bounded = validated::<BoundedProgram>();
    let identity = bounded.identity().clone();
    let revision = bounded.revision().clone();

    let denial = install_rostered_application_program(bounded, &installed_schema, &roster)
        .err()
        .expect("the bounded program was never admitted into this roster");
    let crossed = crossed(&denial);

    assert_eq!(
        crossed.subject(),
        format!(
            "unrostered program {} at revision {revision}",
            identity.as_str()
        )
    );
    assert_eq!(crossed.support_denial(), Some(&denial));
}

#[test]
fn a_foreign_schema_binding_crosses_as_its_rendered_refusal() {
    let rostered_schema = installed_support_schema();
    let presented_schema = installed_support_schema();
    let roster = WorthQueryProgramSupportAdmission::for_installed_schema(&rostered_schema)
        .support(&validated::<CompleteProgram>())
        .expect("the complete program owns the whole catalog")
        .close()
        .expect("the complete program leaves no rule unowned");

    let denial = install_rostered_application_program(
        validated::<CompleteProgram>(),
        &presented_schema,
        &roster,
    )
    .err()
    .expect("a roster admitted against another installation cannot install here");
    let crossed = crossed(&denial);

    assert_eq!(
        crossed.subject(),
        format!(
            "foreign schema binding: roster {} presented {}",
            rostered_schema
                .binding_identity()
                .schema_identity()
                .render_hex(),
            presented_schema
                .binding_identity()
                .schema_identity()
                .render_hex()
        )
    );
    assert_eq!(crossed.support_denial(), Some(&denial));
}

/// Crosses one support refusal into the installation boundary exactly as the
/// installation entry points do.
fn crossed(
    denial: &WorthQueryProgramSupportDenial,
) -> WorthQueryApplicationProgramInstallationDenial {
    denial.clone().into()
}
