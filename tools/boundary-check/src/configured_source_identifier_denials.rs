use crate::config::SourceIdentifierDenialConfig;
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::Arc,
};
use syn::visit::Visit;
mod inputs;
#[cfg(test)]
mod width_tests;

enum Work<'a> {
    Diagnostic(Diagnostic),
    Inspect {
        path: String,
        names: Arc<BTreeSet<String>>,
        rule: &'a SourceIdentifierDenialConfig,
    },
}

pub(crate) struct PreparedInputs<'a>(Vec<inputs::Input<'a>>);

pub(crate) fn prepare_source_identifier_denials<'a>(
    workspace: &Path,
    rules: &'a [SourceIdentifierDenialConfig],
) -> PreparedInputs<'a> {
    let inputs = inputs::collect(workspace, rules);
    crate::source_documents::prefetch(inputs.iter().filter_map(|input| match input {
        inputs::Input::File { path, .. } => Some(path.clone()),
        _ => None,
    }));
    PreparedInputs(inputs)
}

#[cfg(test)]
fn validate_source_identifier_denials(
    workspace: &Path,
    rules: &[SourceIdentifierDenialConfig],
) -> Vec<Diagnostic> {
    validate_prepared_identifier_denials(
        workspace,
        prepare_source_identifier_denials(workspace, rules),
    )
}

pub(crate) fn validate_prepared_identifier_denials(
    workspace: &Path,
    inputs: PreparedInputs<'_>,
) -> Vec<Diagnostic> {
    let mut facts = BTreeMap::new();
    let work: Vec<_> = inputs
        .0
        .into_iter()
        .map(|input| match input {
            inputs::Input::Diagnostic(diagnostic) => Work::Diagnostic(diagnostic),
            inputs::Input::File { path, rule } => {
                let names = facts
                    .entry(path.clone())
                    .or_insert_with(|| names_from_file(workspace, &path));
                match names {
                    Ok(names) => Work::Inspect {
                        path: path
                            .strip_prefix(workspace)
                            .unwrap_or(&path)
                            .display()
                            .to_string(),
                        names: Arc::clone(names),
                        rule,
                    },
                    Err(diagnostic) => Work::Diagnostic(diagnostic.clone()),
                }
            }
        })
        .collect();
    crate::source_workers::map(&work, crate::source_documents::width(), |work| match work {
        Work::Diagnostic(diagnostic) => vec![diagnostic.clone()],
        Work::Inspect { path, names, rule } => diagnostics_for_names(path, names, rule),
    })
    .into_iter()
    .flatten()
    .collect()
}

fn names_from_file(workspace: &Path, path: &Path) -> Result<Arc<BTreeSet<String>>, Diagnostic> {
    let source = crate::source_documents::read_to_string(path)
        .map_err(|error| unreadable_source_diagnostic(workspace, path, &error))?;
    let file = crate::source_syntax::parse_named(path, &source).map_err(|_| {
        Diagnostic::new(
            DiagnosticCode::Bc2001BandDependencyViolation,
            path.strip_prefix(workspace)
                .unwrap_or(path)
                .display()
                .to_string(),
            "source could not be parsed for forbidden authority identifiers",
        )
    })?;
    Ok(Arc::new(identifiers(&file)))
}

fn unreadable_source_diagnostic(
    workspace: &Path,
    path: &Path,
    error: &std::io::Error,
) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::Bc2001BandDependencyViolation,
        path.strip_prefix(workspace)
            .unwrap_or(path)
            .display()
            .to_string(),
        format!("configured source boundary could not be read: {error}"),
    )
}

#[cfg(test)]
fn diagnostics_for_source(
    path: &str,
    source: &str,
    rule: &SourceIdentifierDenialConfig,
) -> Vec<Diagnostic> {
    match crate::source_syntax::parse_file(source) {
        Ok(file) => diagnostics_for_names(path, &identifiers(&file), rule),
        Err(_) => vec![Diagnostic::new(
            DiagnosticCode::Bc2001BandDependencyViolation,
            path,
            "source could not be parsed for forbidden authority identifiers",
        )],
    }
}

fn diagnostics_for_names(
    path: &str,
    names: &BTreeSet<String>,
    rule: &SourceIdentifierDenialConfig,
) -> Vec<Diagnostic> {
    names
        .iter()
        .filter(|name| {
            rule.forbidden_identifiers
                .iter()
                .any(|denied| denied == *name)
                || rule
                    .forbidden_identifier_fragments
                    .iter()
                    .any(|denied| name.contains(denied))
        })
        .map(|identifier| {
            Diagnostic::new(
                DiagnosticCode::Bc2001BandDependencyViolation,
                path,
                format!(
                    "source contains forbidden identifier `{identifier}`: {}",
                    rule.guidance
                ),
            )
        })
        .collect()
}

fn identifiers(file: &syn::File) -> BTreeSet<String> {
    let mut visitor = Identifiers(BTreeSet::new());
    visitor.visit_file(file);
    visitor.0
}
struct Identifiers(BTreeSet<String>);
impl Visit<'_> for Identifiers {
    fn visit_ident(&mut self, identifier: &proc_macro2::Ident) {
        self.0.insert(identifier.to_string());
    }
    fn visit_macro(&mut self, node: &syn::Macro) {
        syn::visit::visit_macro(self, node);
        self.tokens(node.tokens.clone());
    }
}
impl Identifiers {
    fn tokens(&mut self, tokens: proc_macro2::TokenStream) {
        for token in tokens {
            match token {
                proc_macro2::TokenTree::Ident(identifier) => self.visit_ident(&identifier),
                proc_macro2::TokenTree::Group(group) => self.tokens(group.stream()),
                proc_macro2::TokenTree::Punct(_) | proc_macro2::TokenTree::Literal(_) => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        diagnostics_for_source, validate_source_identifier_denials, SourceIdentifierDenialConfig,
    };

    fn rule() -> SourceIdentifierDenialConfig {
        SourceIdentifierDenialConfig {
            root: "physical_runtime".into(),
            exclude_paths: Vec::new(),
            forbidden_identifiers: vec![
                "serde_json".into(),
                "BranchWriterAuthority".into(),
                "MVCC".into(),
            ],
            forbidden_identifier_fragments: Vec::new(),
            guidance: "wrong authority layer".into(),
        }
    }

    #[test]
    fn configured_isolation_entry_retirement_fences_both_former_producers() {
        let config: crate::config::Road1Config =
            toml::from_str(include_str!("../config/road1.toml")).unwrap();
        for owner in [
            "worth-store-physical-isolation",
            "worth-store-physical-certification",
        ] {
            let root = format!("workspaces/worth-store/crates/{owner}/src");
            let rule = config
                .source_identifier_denials
                .iter()
                .find(|rule| rule.root == root)
                .expect("retired entry producer fence");
            assert_eq!(diagnostics_for_source(
                "entry.rs",
                "struct PhysicalIsolationEntryIdentity; struct RecoveryReadinessBasis; fn admit_physical_isolation_entry() {}",
                rule,
            ).len(), 3);
            assert!(diagnostics_for_source(
                "root_epoch_basis.rs",
                "struct PhysicalIsolationRootEpochBasis;",
                rule,
            )
            .is_empty());
        }
    }

    #[test]
    fn rust_identifiers_are_rejected_without_matching_comments_or_strings() {
        assert_eq!(
            diagnostics_for_source(
                "physical_runtime/work/shortcut.rs",
                "use serde_json::Value; type BranchWriterAuthority = Value;",
                &rule(),
            )
            .len(),
            2
        );
        assert!(diagnostics_for_source(
            "physical_runtime/work/honest.rs",
            "// serde_json BranchWriterAuthority MVCC\nconst NOTE: &str = \"MVCC\";",
            &rule(),
        )
        .is_empty());
        assert_eq!(
            diagnostics_for_source(
                "physical_runtime/work/macro_shortcut.rs",
                "macro_rules! shortcut { () => { struct BranchWriterAuthority; } }",
                &rule(),
            )
            .len(),
            1
        );
    }

    #[test]
    fn raw_identifiers_are_rejected_like_their_bare_spelling() {
        let mut rule = rule();
        rule.forbidden_identifier_fragments = vec!["BranchWriter".into()];
        assert_eq!(
            diagnostics_for_source(
                "physical_runtime/work/raw_shortcut.rs",
                "use r#serde_json::Value; struct r#StoreBranchWriterLease;",
                &rule,
            )
            .len(),
            2
        );
    }

    #[test]
    fn terminal_json_compatibility_reexports_are_rejected() {
        let mut rule = rule();
        rule.forbidden_identifiers = vec![
            "project_store_boundary_fact_to_terminal_json".into(),
            "StoreTerminalJsonReadmission".into(),
        ];
        let diagnostics = diagnostics_for_source(
            "physical_runtime/work/json_shortcut.rs",
            "use worth_store_aspect_native::{project_store_boundary_fact_to_terminal_json as project, StoreTerminalJsonReadmission};",
            &rule,
        );
        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn semantic_authority_fragments_reject_composed_names_without_rejecting_physical_branches() {
        let mut rule = rule();
        rule.forbidden_identifiers.clear();
        rule.forbidden_identifier_fragments = vec![
            "BranchWriter".into(),
            "branch_label".into(),
            "BRANCH_WRITER".into(),
            "SemanticGeneration".into(),
        ];
        let diagnostics = diagnostics_for_source(
            "physical_runtime/record_serving/authority.rs",
            "struct StoreBranchWriterRegistry; const branch_label_scope: u8 = 1; static BRANCH_WRITER_AUTHORITY: u8 = 1; struct SemanticGenerationLease; enum PhysicalRoutingTree { Branch }",
            &rule,
        );
        assert_eq!(diagnostics.len(), 4);
    }

    #[test]
    fn missing_governed_source_root_fails_closed() {
        let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut rule = rule();
        rule.root = "definitely-missing-governed-source-root".into();

        let diagnostics = validate_source_identifier_denials(workspace, &[rule]);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0]
            .message()
            .contains("configured source boundary could not be read"));
    }

    #[test]
    fn excluded_source_paths_are_the_only_escape_from_a_governed_root() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let workspace = std::env::temp_dir().join(format!(
            "worth-boundary-identifier-exclusion-{}-{nonce}",
            std::process::id()
        ));
        let governed = workspace.join("governed");
        let excluded = governed.join("allowed");
        std::fs::create_dir_all(&excluded).expect("create governed fixture");
        std::fs::write(governed.join("denied.rs"), "use worth_query_host::facade;")
            .expect("write denied fixture");
        std::fs::write(excluded.join("allowed.rs"), "use worth_query_host::facade;")
            .expect("write excluded fixture");
        std::fs::write(
            governed.join("allowed_file.rs"),
            "use worth_query_host::facade;",
        )
        .expect("write excluded file fixture");
        let rule = SourceIdentifierDenialConfig {
            root: "governed".into(),
            exclude_paths: vec!["governed/allowed".into(), "governed/allowed_file.rs".into()],
            forbidden_identifiers: vec!["worth_query_host".into()],
            forbidden_identifier_fragments: Vec::new(),
            guidance: "Query imports stay in the adapter".into(),
        };

        let diagnostics = validate_source_identifier_denials(&workspace, &[rule]);

        std::fs::remove_dir_all(&workspace).expect("remove governed fixture");
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].subject().ends_with("denied.rs"));
    }
}
