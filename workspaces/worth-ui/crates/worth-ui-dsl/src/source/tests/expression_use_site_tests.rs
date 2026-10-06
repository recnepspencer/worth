use super::expression_compilation::{compile_source, denials, PROJECTIONS};
use crate::{
    WorthUiDslCompileDiagnosticCode as Code, WorthUiDslCompiler, WorthUiExpressionOperand,
    WorthUiExpressionOperandSource, WorthUiExpressionResultType, WorthUiIntentConcurrencyScope,
    WorthUiIntentConfirmationContractSpec, WorthUiIntentConsequenceContractSpec,
    WorthUiIntentDeclarationSpec, WorthUiIntentInteractionFamily,
    WorthUiIntentMutabilitySourceSpec, WorthUiIntentOperabilityContractSpec,
    WorthUiIntentPolicySourceSpec, WorthUiIntentReadinessSourceSpec,
    WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredArtifactInputModule,
    WorthUiSealedSemanticPackage, WorthUiSemanticDeclaration,
};

const CONDITIONS: &str = r#"
condition pulse.writable {
    operand ready query-scalar pulse.ready;
    when (ready)
}
derived pulse.caption {
    operand label query-scalar pulse.label;
    result text;
    value (label)
}
"#;

fn intent(operability: &str) -> String {
    format!(
        "intent pulse.intent.open {{
            definition pulse.intent.open
            interaction activate
            operability pulse.intent.open.operability
                {operability}
            confirmation pulse.intent.open.confirmation not-required
            concurrency target-route-single-flight
            consequences mounted-posture
        }}"
    )
}

fn operability_of(
    package: &WorthUiSealedSemanticPackage,
    identity: &str,
) -> WorthUiIntentOperabilityContractSpec {
    package
        .module_ids()
        .iter()
        .flat_map(|module| package.declaration_views(module).into_iter().flatten())
        .find_map(|view| match view.declaration() {
            WorthUiSemanticDeclaration::SemanticArtifact(artifact)
                if artifact.declaration().key().as_str() == identity =>
            {
                artifact.declaration().intent_declaration().cloned()
            }
            _ => None,
        })
        .expect("the package seals the intent declaration")
        .operability()
        .clone()
}

fn compile_with(operability: &str) -> WorthUiSealedSemanticPackage {
    compile_source(&format!(
        "{PROJECTIONS}\n{CONDITIONS}\n{}",
        intent(operability)
    ))
    .expect("condition-sourced operability should compile")
}

fn use_site_denials(operability: &str) -> Vec<(Code, String)> {
    denials(&format!("{CONDITIONS}\n{}", intent(operability)))
}

#[test]
fn every_operability_axis_reads_a_condition() {
    let package = compile_with(
        "mutability-condition pulse.writable
         readiness-condition pulse.writable
         policy-condition pulse.writable",
    );
    let operability = operability_of(&package, "pulse.intent.open");

    assert_eq!(
        operability.mutability(),
        &WorthUiIntentMutabilitySourceSpec::condition("pulse.writable")
    );
    assert_eq!(
        operability.readiness(),
        &WorthUiIntentReadinessSourceSpec::condition("pulse.writable")
    );
    assert_eq!(
        operability.policy(),
        &WorthUiIntentPolicySourceSpec::condition("pulse.writable")
    );
    assert_eq!(operability.policy().application_fact(), None);
}

#[test]
fn the_source_kind_is_part_of_the_package_identity() {
    let fact = compile_with(
        "mutability-application-boolean pulse.writable
         readiness-committed-draft
         policy-application-boolean pulse.policy",
    );
    let condition = compile_with(
        "mutability-condition pulse.writable
         readiness-committed-draft
         policy-application-boolean pulse.policy",
    );

    assert_ne!(fact.identity(), condition.identity());
}

#[test]
fn one_axis_takes_exactly_one_source() {
    let second_source = use_site_denials(
        "mutability-application-boolean pulse.mutable
         mutability-condition pulse.writable
         readiness-committed-draft
         policy-condition pulse.writable",
    );
    let trailing = use_site_denials(
        "mutability-condition pulse.writable
         readiness-committed-draft
         policy-condition pulse.writable pulse.writable",
    );

    assert_eq!(second_source.len(), 1);
    assert_eq!(second_source[0].0, Code::InvalidIntentDeclaration);
    assert!(second_source[0]
        .1
        .contains("unknown intent readiness source kind `mutability-condition`"));
    assert_eq!(trailing.len(), 1);
    assert_eq!(trailing[0].0, Code::InvalidIntentDeclaration);
    assert!(trailing[0]
        .1
        .contains("unknown intent declaration clause `pulse.writable`"));
}

#[test]
fn an_undeclared_condition_is_an_unknown_use_site() {
    let found = use_site_denials(
        "mutability-committed-draft
         readiness-committed-draft
         policy-condition pulse.missing",
    );

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].0, Code::UnknownExpressionUseSite);
    assert!(found[0].1.contains("operability policy"));
    assert!(found[0].1.contains("`pulse.missing`"));
}

#[test]
fn a_derived_declaration_cannot_source_operability() {
    let found = use_site_denials(
        "mutability-committed-draft
         readiness-condition pulse.caption
         policy-application-boolean pulse.policy",
    );

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].0, Code::ExpressionUseSiteRoleMismatch);
    assert!(found[0].1.contains("operability readiness"));
    assert!(found[0].1.contains("derived declaration, not a condition"));
}

#[test]
fn a_refused_condition_reports_only_its_admission_diagnostic() {
    let found = denials(&format!(
        "condition pulse.broken {{
            operand rows query-scalar pulse.rows;
            when (rows)
        }}
        {}",
        intent(
            "mutability-condition pulse.broken
             readiness-committed-draft
             policy-application-boolean pulse.policy",
        )
    ));

    assert_eq!(found.len(), 1, "{found:?}");
    assert_ne!(found[0].0, Code::UnknownExpressionUseSite);
    assert_ne!(found[0].0, Code::ExpressionUseSiteRoleMismatch);
}

fn rust_intent(policy: WorthUiIntentPolicySourceSpec) -> WorthUiIntentDeclarationSpec {
    WorthUiIntentDeclarationSpec::new(
        "pulse.intent.open",
        "pulse.intent.open",
        WorthUiIntentInteractionFamily::Activate,
        WorthUiIntentOperabilityContractSpec::new(
            "pulse.intent.open.operability",
            WorthUiIntentMutabilitySourceSpec::condition("pulse.writable"),
            WorthUiIntentReadinessSourceSpec::committed_draft(),
            policy,
        ),
        WorthUiIntentConfirmationContractSpec::not_required("pulse.intent.open.confirmation"),
        WorthUiIntentConcurrencyScope::TargetRouteSingleFlight,
        WorthUiIntentConsequenceContractSpec::mounted_posture(),
    )
}

fn compile_rust(
    policy: WorthUiIntentPolicySourceSpec,
) -> Result<WorthUiSealedSemanticPackage, crate::WorthUiDslCompileReport> {
    let module = WorthUiRustAuthoredArtifactInputModule::new("main.wui")
        .try_with_condition(
            "pulse.writable",
            [WorthUiExpressionOperand::new(
                "ready",
                WorthUiExpressionOperandSource::ApplicationBoolean {
                    fact: "pulse.ready".to_owned(),
                },
            )],
            "ready",
        )
        .unwrap()
        .try_with_derived(
            "pulse.caption",
            [WorthUiExpressionOperand::new(
                "name",
                WorthUiExpressionOperandSource::ApplicationText {
                    fact: "pulse.name".to_owned(),
                },
            )],
            WorthUiExpressionResultType::Text,
            "name",
        )
        .unwrap()
        .with_intent_declaration(rust_intent(policy));
    WorthUiDslCompiler::compile_rust_authored(&WorthUiRustAuthoredArtifactInput::from_modules([
        module,
    ]))
}

#[test]
fn rust_authored_condition_sources_seal_and_are_checked_like_source() {
    let package = compile_rust(WorthUiIntentPolicySourceSpec::condition("pulse.writable"))
        .expect("Rust-authored condition sources should compile");
    let codes = |policy| {
        compile_rust(policy)
            .expect_err("an invalid use site fails the package")
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.identity().code())
            .collect::<Vec<_>>()
    };

    assert_eq!(
        operability_of(&package, "pulse.intent.open").mutability(),
        &WorthUiIntentMutabilitySourceSpec::condition("pulse.writable")
    );
    assert_eq!(
        codes(WorthUiIntentPolicySourceSpec::condition("pulse.missing")),
        [Code::UnknownExpressionUseSite]
    );
    assert_eq!(
        codes(WorthUiIntentPolicySourceSpec::condition("pulse.caption")),
        [Code::ExpressionUseSiteRoleMismatch]
    );
}
