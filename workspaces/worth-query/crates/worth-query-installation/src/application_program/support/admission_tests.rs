use super::super::program_support_fixture::{
    installed_support_schema, validated, AuditProgram, BoundedProgram, CompleteProgram,
    FeaturelessProgram, RaisedRuleProgram, UninstalledActionProgram, AUDIT_RULE, BOUNDED_RULE,
    UNKNOWN_OPERATION,
};
use super::{WorthQueryProgramSupportAdmission, WorthQueryProgramSupportDenial};
use crate::application_program::{
    install_application_program, install_rostered_application_program,
};

#[test]
fn a_roster_of_two_programs_owns_a_catalog_neither_owns_alone() {
    let installed_schema = installed_support_schema();
    let bounded = validated::<BoundedProgram>();
    let audit = validated::<AuditProgram>();

    let roster = WorthQueryProgramSupportAdmission::for_installed_schema(&installed_schema)
        .support(&bounded)
        .expect("the bounded program fits the installed contracts")
        .support(&audit)
        .expect("the audit program fits the installed contracts")
        .close()
        .expect("together the two programs own every installed rule");

    assert_eq!(roster.entries().len(), 2);
    assert_eq!(
        roster.schema_binding(),
        &installed_schema.binding_identity()
    );
    let entry = roster
        .entry(bounded.revision())
        .expect("the bounded program answers to its own revision");
    assert_eq!(entry.identity(), bounded.identity());
    assert_eq!(entry.rules().len(), 1);
    assert_eq!(entry.rules()[0].identity(), BOUNDED_RULE);
    assert!(entry.declares_rule(&entry.rules()[0]));
    assert!(!roster
        .entry(audit.revision())
        .expect("the audit program is rostered")
        .declares_rule(&entry.rules()[0]));
}

#[test]
fn closing_a_roster_denies_an_installed_rule_no_program_declares() {
    let installed_schema = installed_support_schema();
    let bounded = validated::<BoundedProgram>();

    let denial = WorthQueryProgramSupportAdmission::for_installed_schema(&installed_schema)
        .support(&bounded)
        .expect("the bounded program fits the installed contracts")
        .close()
        .err()
        .expect("the audit rule would be enforced with no declaring owner");

    match denial {
        WorthQueryProgramSupportDenial::UndeclaredInstalledRule { rule } => {
            assert_eq!(rule.identity(), AUDIT_RULE);
        }
        other => panic!("expected an undeclared installed rule, got {other:?}"),
    }
}

#[test]
fn support_denies_a_rule_version_this_host_never_installed() {
    let installed_schema = installed_support_schema();
    let raised = validated::<RaisedRuleProgram>();

    let denial = WorthQueryProgramSupportAdmission::for_installed_schema(&installed_schema)
        .support(&raised)
        .err()
        .expect("version two of the bounded rule is not installed");

    match denial {
        WorthQueryProgramSupportDenial::UnsupportedRuleContract { program, rule } => {
            assert_eq!(&program, raised.identity());
            assert_eq!(rule.identity(), BOUNDED_RULE);
            assert_eq!(rule.major(), 2);
        }
        other => panic!("expected an unsupported rule contract, got {other:?}"),
    }
}

#[test]
fn support_denies_an_action_on_an_operation_this_host_never_installed() {
    let installed_schema = installed_support_schema();
    let acting = validated::<UninstalledActionProgram>();

    let denial = WorthQueryProgramSupportAdmission::for_installed_schema(&installed_schema)
        .support(&acting)
        .err()
        .expect("the retire operation is not installed");

    match denial {
        WorthQueryProgramSupportDenial::UnsupportedAction { program, binding } => {
            assert_eq!(&program, acting.identity());
            assert_eq!(binding, UNKNOWN_OPERATION);
        }
        other => panic!("expected an unsupported action, got {other:?}"),
    }
}

#[test]
fn support_denies_a_program_that_governs_nothing() {
    let installed_schema = installed_support_schema();
    let featureless = validated::<FeaturelessProgram>();

    let denial = WorthQueryProgramSupportAdmission::for_installed_schema(&installed_schema)
        .support(&featureless)
        .err()
        .expect("a program declaring no feature governs nothing");

    assert_eq!(
        denial,
        WorthQueryProgramSupportDenial::EmptyProgram {
            program: featureless.identity().clone(),
        }
    );
}

#[test]
fn installing_one_rostered_program_carries_its_canonical_revision() {
    let installed_schema = installed_support_schema();
    let bounded = validated::<BoundedProgram>();
    let audit = validated::<AuditProgram>();
    let expected = bounded.revision().clone();
    let roster = WorthQueryProgramSupportAdmission::for_installed_schema(&installed_schema)
        .support(&bounded)
        .expect("the bounded program fits the installed contracts")
        .support(&audit)
        .expect("the audit program fits the installed contracts")
        .close()
        .expect("together the two programs own every installed rule");

    let installed = install_rostered_application_program(bounded, &installed_schema, &roster)
        .expect("a rostered program installs against its own installation");

    assert_eq!(installed.revision(), &expected);
    assert_eq!(
        installed.schema_binding(),
        &installed_schema.binding_identity()
    );
}

#[test]
fn installing_a_program_the_roster_never_admitted_is_denied() {
    let installed_schema = installed_support_schema();
    let complete = validated::<CompleteProgram>();
    let roster = WorthQueryProgramSupportAdmission::for_installed_schema(&installed_schema)
        .support(&complete)
        .expect("the complete program owns the whole catalog")
        .close()
        .expect("the complete program leaves no rule unowned");
    let bounded = validated::<BoundedProgram>();
    let revision = bounded.revision().clone();

    let denial = install_rostered_application_program(bounded, &installed_schema, &roster)
        .err()
        .expect("the bounded program was never admitted into this roster");

    match denial {
        WorthQueryProgramSupportDenial::UnrosteredProgram {
            revision: denied, ..
        } => {
            assert_eq!(denied, revision);
        }
        other => panic!("expected an unrostered program, got {other:?}"),
    }
}

#[test]
fn the_single_program_install_path_still_demands_sole_ownership() {
    let installed_schema = installed_support_schema();

    let installed = install_application_program(validated::<CompleteProgram>(), &installed_schema)
        .expect("the complete program owns the installed catalog by itself");
    assert_eq!(
        installed.revision(),
        validated::<CompleteProgram>().revision()
    );

    let denial = install_application_program(validated::<BoundedProgram>(), &installed_schema)
        .err()
        .expect("the bounded program leaves the audit rule unowned");
    assert_eq!(
        denial.subject(),
        format!("undeclared installed rule: {AUDIT_RULE}")
    );
}

#[test]
fn support_denies_the_same_revision_admitted_twice() {
    let installed_schema = installed_support_schema();
    let complete = validated::<CompleteProgram>();

    let denial = WorthQueryProgramSupportAdmission::for_installed_schema(&installed_schema)
        .support(&complete)
        .expect("the complete program fits the installed contracts")
        .support(&complete)
        .err()
        .expect("one revision cannot hold two roster entries");

    assert_eq!(
        denial,
        WorthQueryProgramSupportDenial::DuplicateProgram {
            program: complete.identity().clone(),
            revision: complete.revision().clone(),
        }
    );
}
