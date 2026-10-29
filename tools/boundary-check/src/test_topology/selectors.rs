//! Governed gates select category prefixes, never behaviour-name regular expressions.
use super::{category_paths::CATEGORIES, failure};
use crate::diagnostics::Diagnostic;
use std::path::Path;

pub(super) fn validate(root: &Path) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for relative in [
        "scripts/ci/query_gate.py",
        "workspaces/worth-query/.config/nextest.toml",
        "workspaces/worth-query-bank-world/.config/nextest.toml",
    ] {
        let path = root.join(relative);
        if !path.exists() {
            continue;
        }
        let text = match crate::source_documents::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                diagnostics.push(failure(relative, error.to_string()));
                continue;
            }
        };
        for suffix in text.split("test(/").skip(1) {
            let expression = suffix.split("/)").next().unwrap_or(suffix);
            if !CATEGORIES.iter().any(|c| expression == format!("^{c}::")) {
                diagnostics.push(failure(
                    relative,
                    format!("name regex `{expression}` is forbidden; select a category prefix"),
                ));
            }
        }
        if relative.ends_with(".py")
            && ["re.search(", "re.match(", "re.fullmatch(", "re.compile("]
                .iter()
                .any(|p| text.contains(p))
        {
            diagnostics.push(failure(
                relative,
                "name regex in Query gate; use category-prefix selection",
            ));
        }
    }
    diagnostics
}
