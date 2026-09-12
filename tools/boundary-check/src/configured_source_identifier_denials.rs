use std::{collections::BTreeSet, fs, path::Path};

use syn::visit::Visit;

use crate::config::SourceIdentifierDenialConfig;
use crate::diagnostics::{Diagnostic, DiagnosticCode};

pub(crate) fn validate_source_identifier_denials(
    workspace: &Path,
    rules: &[SourceIdentifierDenialConfig],
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for rule in rules {
        visit_rust_sources(
            workspace,
            &workspace.join(&rule.root),
            rule,
            &mut diagnostics,
        );
    }
    diagnostics
}

fn visit_rust_sources(
    workspace: &Path,
    path: &Path,
    rule: &SourceIdentifierDenialConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if rule
        .exclude_paths
        .iter()
        .map(|excluded| workspace.join(excluded))
        .any(|excluded| path == excluded || path.starts_with(excluded))
    {
        return;
    }
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) => {
            diagnostics.push(unreadable_source_diagnostic(workspace, path, &error));
            return;
        }
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                diagnostics.push(unreadable_source_diagnostic(workspace, path, &error));
                continue;
            }
        };
        let path = entry.path();
        if rule
            .exclude_paths
            .iter()
            .map(|excluded| workspace.join(excluded))
            .any(|excluded| path == excluded || path.starts_with(excluded))
        {
            continue;
        }
        if path.is_dir() {
            visit_rust_sources(workspace, &path, rule, diagnostics);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let source = match fs::read_to_string(&path) {
                Ok(source) => source,
                Err(error) => {
                    diagnostics.push(unreadable_source_diagnostic(workspace, &path, &error));
                    continue;
                }
            };
            diagnostics.extend(diagnostics_for_source(
                &path
                    .strip_prefix(workspace)
                    .unwrap_or(&path)
                    .display()
                    .to_string(),
                &source,
                rule,
            ));
        }
    }
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

fn diagnostics_for_source(
    path: &str,
    source: &str,
    rule: &SourceIdentifierDenialConfig,
) -> Vec<Diagnostic> {
    let Ok(file) = syn::parse_file(source) else {
        return vec![Diagnostic::new(
            DiagnosticCode::Bc2001BandDependencyViolation,
            path,
            "source could not be parsed for forbidden authority identifiers",
        )];
    };
    let mut visitor = ForbiddenIdentifierVisitor {
        forbidden: &rule.forbidden_identifiers,
        forbidden_fragments: &rule.forbidden_identifier_fragments,
        found: BTreeSet::new(),
    };
    visitor.visit_file(&file);
    visitor
        .found
        .into_iter()
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

struct ForbiddenIdentifierVisitor<'a> {
    forbidden: &'a [String],
    forbidden_fragments: &'a [String],
    found: BTreeSet<String>,
}

impl Visit<'_> for ForbiddenIdentifierVisitor<'_> {
    fn visit_ident(&mut self, identifier: &proc_macro2::Ident) {
        let identifier = identifier.to_string();
        if self.forbidden.iter().any(|denied| denied == &identifier)
            || self
                .forbidden_fragments
                .iter()
                .any(|denied| identifier.contains(denied))
        {
            self.found.insert(identifier);
        }
    }

    fn visit_macro(&mut self, node: &syn::Macro) {
        syn::visit::visit_macro(self, node);
        self.visit_token_stream(node.tokens.clone());
    }
}

impl ForbiddenIdentifierVisitor<'_> {
    fn visit_token_stream(&mut self, tokens: proc_macro2::TokenStream) {
        for token in tokens {
            match token {
                proc_macro2::TokenTree::Ident(identifier) => self.visit_ident(&identifier),
                proc_macro2::TokenTree::Group(group) => self.visit_token_stream(group.stream()),
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
