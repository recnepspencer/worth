//! Local typed hash-walk fences must preserve every root Clippy setting.
use crate::diagnostics::Diagnostic;
use std::path::Path;

const CRATE_ROOTS: &[&str] = &[
    "crates/worth-runtime-world",
    "crates/worth-runtime-bridge",
    "crates/worth-execution",
    "workspaces/worth-query/crates/worth-query-execution",
];
const HASH_METHODS: &[(&str, &[&str])] = &[
    (
        "HashMap",
        &[
            "iter",
            "iter_mut",
            "keys",
            "values",
            "values_mut",
            "into_keys",
            "into_values",
            "drain",
            "retain",
            "extract_if",
        ],
    ),
    (
        "HashSet",
        &[
            "iter",
            "drain",
            "retain",
            "extract_if",
            "difference",
            "symmetric_difference",
            "intersection",
            "union",
        ],
    ),
];
const REASON: &str = "canonical order: hash containers are for point lookup only";

pub(super) fn check(root: &Path) -> Result<Vec<Diagnostic>, String> {
    // Source-rule fixtures without a root Clippy configuration exercise other laws.
    let path = root.join("clippy.toml");
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let mut expected: toml::Value =
        toml::from_str(&std::fs::read_to_string(path).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    let inherited = expected
        .get("disallowed-methods")
        .cloned()
        .unwrap_or_else(|| toml::Value::Array(Vec::new()));
    let mut methods = Vec::new();
    for (container, names) in HASH_METHODS {
        for name in *names {
            let mut entry = toml::map::Map::new();
            entry.insert(
                "path".to_owned(),
                toml::Value::String(format!("std::collections::{container}::{name}")),
            );
            entry.insert("reason".to_owned(), toml::Value::String(REASON.to_owned()));
            methods.push(toml::Value::Table(entry));
        }
    }
    methods.extend(
        inherited
            .as_array()
            .ok_or("root disallowed-methods is not an array")?
            .iter()
            .cloned(),
    );
    expected
        .as_table_mut()
        .ok_or("root Clippy configuration is not a table")?
        .insert("disallowed-methods".to_owned(), toml::Value::Array(methods));
    let mut diagnostics = Vec::new();
    for crate_root in CRATE_ROOTS {
        if !root.join(crate_root).join("Cargo.toml").is_file() {
            continue;
        }
        let subject = format!("{crate_root}/clippy.toml");
        let local = std::fs::read_to_string(root.join(&subject))
            .map_err(|error| error.to_string())
            .and_then(|text| {
                toml::from_str::<toml::Value>(&text).map_err(|error| error.to_string())
            });
        if local.as_ref().ok() != Some(&expected) {
            diagnostics.push(Diagnostic::new(
                crate::diagnostics::DiagnosticCode::Bc7009CanonicalOrder,
                subject,
                "local hash-walk configuration must preserve all root Clippy settings and add only the declared hash-walk entries",
            ));
        }
    }
    Ok(diagnostics)
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_local_hash_walk_fence_cannot_shadow_root_lint_settings() {
        let root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hash_walk_configuration");
        let diagnostics = super::check(&root).unwrap();
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message().contains("root Clippy settings"));
    }
    use std::path::Path;
}
