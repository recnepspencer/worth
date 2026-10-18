//! Resolve local aliases and distinguish declared generic/external roots.
//! Unknown paths retain the isolation visitor's conservative name checks.
use super::*;

pub(super) fn standard_crates() -> BTreeSet<String> {
    ["std", "core", "alloc"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

/// Cargo dependency keys are the extern-prelude names, including renamed deps.
/// Missing/unreadable/malformed manifests grant no dependency exemptions.
pub(super) fn external_crates(workspace: &Path, path: &str) -> BTreeSet<String> {
    let mut crates = standard_crates();
    let source = workspace.join(path);
    for directory in source.ancestors().skip(1) {
        if !directory.starts_with(workspace) {
            break;
        }
        let manifest = directory.join("Cargo.toml");
        if !manifest.exists() {
            continue;
        }
        if let Ok(text) = crate::source_documents::read_to_string(&manifest)
            && let Ok(value) = toml::from_str::<toml::Value>(&text)
        {
            for section in ["dependencies", "dev-dependencies"] {
                if let Some(dependencies) = value.get(section).and_then(toml::Value::as_table) {
                    crates.extend(dependencies.keys().map(|name| name.replace('-', "_")));
                }
            }
        }
        break;
    }
    crates
}

impl IsolationVisitor<'_> {
    /// A type parameter is not a workflow-owned concrete type. A resolved
    /// extern-prelude path cannot name an item defined in this same crate.
    pub(super) fn independent_root(&self, segments: &[String]) -> bool {
        if segments
            .first()
            .is_some_and(|root| self.generic_types.contains(root))
        {
            return true;
        }
        let mut path = segments.to_vec();
        for _ in 0..MAXIMUM_ALIAS_DEPTH {
            let Some(root) = path.first() else {
                return false;
            };
            if matches!(root.as_str(), "crate" | "self" | "super") {
                return false;
            }
            if let Some(target) = self.bindings.target(&self.module, root) {
                if target.is_empty() {
                    return false;
                }
                path = [target, &path[1..]].concat();
            } else {
                return self.external_crates.contains(root);
            }
        }
        false
    }

    /// The crate-absolute form of a path, or the path as written when it
    /// names no module of this crate.
    pub(super) fn resolve(&self, segments: &[String]) -> Vec<String> {
        self.resolve_local(segments)
            .unwrap_or_else(|| segments.to_vec())
    }

    /// The crate-absolute form of a `crate::`, `self::` or `super::` path, or
    /// of one that starts with a name a `use` in this module binds. An alias
    /// chain too deep to follow, or an ambiguous alias, resolves to the empty
    /// path, which a glob refuses.
    pub(super) fn resolve_local(&self, segments: &[String]) -> Option<Vec<String>> {
        let mut segments = segments.to_vec();
        for _ in 0..MAXIMUM_ALIAS_DEPTH {
            match segments.first().map(String::as_str) {
                Some("crate") => return Some(segments[1..].to_vec()),
                Some("self") => return Some([&self.module[..], &segments[1..]].concat()),
                Some("super") => {
                    let depth = segments.iter().take_while(|s| *s == "super").count();
                    let kept = self.module.len().saturating_sub(depth);
                    return Some([&self.module[..kept], &segments[depth..]].concat());
                }
                Some(name) => {
                    let target = self.bindings.target(&self.module, name)?;
                    if target.is_empty() {
                        return Some(Vec::new());
                    }
                    segments = [target, &segments[1..]].concat();
                }
                None => return None,
            }
        }
        Some(Vec::new())
    }
}
