//! Payload fields that read an expression: the `derived` and `condition`
//! payload sources, their canonical form, and their use-site checks.

use super::expression_compilation::{compile_source, denials, PROJECTIONS};
use crate::{
    WorthUiDslCompileDiagnosticCode as Code, WorthUiIntentPayloadSource,
    WorthUiIntentPayloadSourceSpec, WorthUiSealedSemanticPackage, WorthUiSemanticDeclaration,
};

const EXPRESSIONS: &str = r#"
condition pulse.writable {
    operand ready query-scalar pulse.ready;
    when (ready)
}
derived pulse.caption {
    operand label query-scalar pulse.label;
    result text;
    value (label)
}
derived pulse.subtitle {
    operand label query-scalar pulse.label;
    result text;
    value (label == "on" ? "active" : label)
}
derived pulse.count {
    operand count application-unsigned64 app.count;
    result integer;
    value (exact_cast<Int64>(count) + 1)
}
derived pulse.tone {
    operand label query-scalar pulse.label;
    result token;
    value (label == "on" ? "active" : label)
}
"#;

fn intent(payload: &str) -> String {
    format!(
        "intent pulse.intent.open {{
            definition pulse.intent.open
            interaction activate
            {payload}
            operability pulse.intent.open.operability
                mutability-committed-draft
                readiness-committed-draft
                policy-application-boolean pulse.policy
            confirmation pulse.intent.open.confirmation not-required
            concurrency target-route-single-flight
            consequences mounted-posture
        }}"
    )
}

fn compile_with(payload: &str) -> WorthUiSealedSemanticPackage {
    compile_source(&format!(
        "{PROJECTIONS}\n{EXPRESSIONS}\n{}",
        intent(payload)
    ))
    .expect("expression-sourced payload should compile")
}

fn payload_denials(payload: &str) -> Vec<(Code, String)> {
    denials(&format!("{EXPRESSIONS}\n{}", intent(payload)))
}

fn payload_sources(package: &WorthUiSealedSemanticPackage) -> Vec<WorthUiIntentPayloadSourceSpec> {
    package
        .module_ids()
        .iter()
        .flat_map(|module| package.declaration_views(module).into_iter().flatten())
        .find_map(|view| match view.declaration() {
            WorthUiSemanticDeclaration::SemanticArtifact(artifact) => {
                artifact.declaration().intent_declaration().cloned()
            }
            _ => None,
        })
        .expect("the package seals the intent declaration")
        .payload_sources()
        .to_vec()
}

#[test]
fn a_payload_field_reads_a_derived_value_and_a_condition() {
    let package = compile_with(
        "payload label derived pulse.caption
         payload enabled condition pulse.writable",
    );

    assert_eq!(
        payload_sources(&package),
        [
            WorthUiIntentPayloadSourceSpec::derived("label", "pulse.caption"),
            WorthUiIntentPayloadSourceSpec::condition("enabled", "pulse.writable"),
        ]
    );
    assert_eq!(
        payload_sources(&package)[0].source(),
        &WorthUiIntentPayloadSource::Derived {
            expression: "pulse.caption".into()
        }
    );
}

#[test]
fn the_payload_source_and_its_expression_are_part_of_the_package_identity() {
    let constant = compile_with("payload label constant-text \"pulse\"");
    let caption = compile_with("payload label derived pulse.caption");
    let subtitle = compile_with("payload label derived pulse.subtitle");
    let again = compile_with("payload label derived pulse.caption");

    assert_ne!(constant.identity(), caption.identity());
    assert_ne!(caption.identity(), subtitle.identity());
    assert_eq!(caption.identity(), again.identity());
}

#[test]
fn a_payload_field_takes_exactly_one_expression() {
    let trailing = payload_denials("payload label derived pulse.caption pulse.subtitle");

    assert_eq!(trailing.len(), 1);
    assert_eq!(trailing[0].0, Code::InvalidIntentDeclaration);
    assert!(trailing[0]
        .1
        .contains("unknown intent declaration clause `pulse.subtitle`"));
}

#[test]
fn an_undeclared_payload_expression_is_an_unknown_use_site() {
    let derived = payload_denials("payload label derived pulse.missing");
    let condition = payload_denials("payload enabled condition pulse.missing");

    assert_eq!(derived.len(), 1);
    assert_eq!(derived[0].0, Code::UnknownExpressionUseSite);
    assert!(derived[0].1.contains("payload field `label`"));
    assert!(derived[0].1.contains("derived value `pulse.missing`"));
    assert_eq!(condition.len(), 1);
    assert_eq!(condition[0].0, Code::UnknownExpressionUseSite);
    assert!(condition[0].1.contains("condition `pulse.missing`"));
}

#[test]
fn a_payload_source_reads_its_expression_in_its_own_role() {
    let derived_reads_condition = payload_denials("payload enabled derived pulse.writable");
    let condition_reads_derived = payload_denials("payload label condition pulse.caption");

    assert_eq!(derived_reads_condition.len(), 1);
    assert_eq!(
        derived_reads_condition[0].0,
        Code::ExpressionUseSiteRoleMismatch
    );
    assert!(derived_reads_condition[0]
        .1
        .contains("which is a condition, not a derived value"));
    assert_eq!(condition_reads_derived.len(), 1);
    assert_eq!(
        condition_reads_derived[0].0,
        Code::ExpressionUseSiteRoleMismatch
    );
    assert!(condition_reads_derived[0]
        .1
        .contains("derived declaration, not a condition"));
}

#[test]
fn a_derived_token_is_never_a_payload_value() {
    let found = payload_denials("payload tone derived pulse.tone");

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].0, Code::ExpressionUseSiteRoleMismatch);
    assert!(found[0]
        .1
        .contains("a derived token, which no payload field carries"));
}

#[test]
fn a_derived_integer_is_left_to_the_payload_schema() {
    let package = compile_with("payload count derived pulse.count");

    assert_eq!(
        payload_sources(&package),
        [WorthUiIntentPayloadSourceSpec::derived(
            "count",
            "pulse.count"
        )]
    );
}

#[test]
fn a_refused_payload_expression_reports_only_its_admission_diagnostic() {
    let found = denials(&format!(
        "derived pulse.broken {{
            operand rows query-scalar pulse.rows;
            result text;
            value (rows)
        }}
        {}",
        intent("payload label derived pulse.broken")
    ));

    assert_eq!(found.len(), 1, "{found:?}");
    assert_ne!(found[0].0, Code::UnknownExpressionUseSite);
    assert_ne!(found[0].0, Code::ExpressionUseSiteRoleMismatch);
}
