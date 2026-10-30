use serde::{Deserialize, Serialize};

use super::ExpectedCompilerDenial;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckedCompilerDiagnostic {
    pub level: String,
    pub code: Option<String>,
    pub message: String,
    pub rendered: String,
}

pub(super) fn checked_diagnostics(
    messages: &[serde_json::Value],
    workspace_root: &str,
) -> Vec<CheckedCompilerDiagnostic> {
    messages
        .iter()
        .filter(|message| message["level"].as_str() == Some("error"))
        .map(|message| CheckedCompilerDiagnostic {
            level: "error".to_owned(),
            code: message["code"]["code"].as_str().map(str::to_owned),
            message: normalize(&semantic_message_text(message), workspace_root),
            rendered: normalize(
                message["rendered"].as_str().unwrap_or_default(),
                workspace_root,
            ),
        })
        .collect()
}

pub(super) fn validate_denial(
    expected: &ExpectedCompilerDenial,
    diagnostics: &[CheckedCompilerDiagnostic],
    stderr: &str,
    fixture_attribution: &[bool],
) -> Result<(), String> {
    if diagnostics.is_empty() {
        return Err(format!(
            "Cargo failed without a structured compiler error; stderr:\n{stderr}"
        ));
    }
    let observed_text = diagnostics
        .iter()
        .flat_map(|diagnostic| [&diagnostic.message, &diagnostic.rendered])
        .cloned()
        .chain(std::iter::once(stderr.to_owned()))
        .collect::<Vec<_>>()
        .join("\n");
    for forbidden in &expected.forbidden_setup_fragments {
        if observed_text.contains(forbidden) {
            return Err(format!(
                "compiler denial was caused by forbidden setup failure {forbidden:?}"
            ));
        }
    }
    if !expected.error_codes.is_empty() {
        return diagnostics
            .iter()
            .zip(fixture_attribution)
            .any(|(diagnostic, attributed)| {
                *attributed
                    && diagnostic
                        .code
                        .as_ref()
                        .is_some_and(|code| expected.error_codes.contains(code))
                    && expected
                        .required_semantic_fragments
                        .iter()
                        .all(|fragment| diagnostic.message.contains(fragment))
            })
            .then_some(())
            .ok_or_else(|| {
                format!(
                    "compiler denial omitted a fixture-attributed error with code {:?} and semantic fragments {:?}",
                    expected.error_codes, expected.required_semantic_fragments
                )
            });
    }
    let semantic_text = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    for fragment in &expected.required_semantic_fragments {
        if !semantic_text.contains(fragment) {
            return Err(format!(
                "compiler denial missed semantic fragment {fragment:?}"
            ));
        }
    }
    Ok(())
}

fn semantic_message_text(message: &serde_json::Value) -> String {
    let mut semantic = Vec::new();
    collect_semantic_messages(message, &mut semantic);
    semantic.join("\n")
}

fn collect_semantic_messages(message: &serde_json::Value, semantic: &mut Vec<String>) {
    if let Some(text) = message["message"].as_str().filter(|text| !text.is_empty()) {
        semantic.push(text.to_owned());
    }
    if let Some(spans) = message["spans"].as_array() {
        semantic.extend(
            spans
                .iter()
                .filter_map(|span| span["label"].as_str())
                .filter(|label| !label.is_empty())
                .map(str::to_owned),
        );
    }
    if let Some(children) = message["children"].as_array() {
        for child in children {
            collect_semantic_messages(child, semantic);
        }
    }
}

fn normalize(value: &str, workspace_root: &str) -> String {
    value
        .replace(workspace_root, "<workspace>")
        .replace('\\', "/")
}
