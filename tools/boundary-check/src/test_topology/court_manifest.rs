//! A court declares one cost class and has no production compilation target.
use super::failure;
use crate::{diagnostics::Diagnostic, source_rules::GovernedCrate};

const CLASSES: &[&str] = &["fast", "process", "exhaustive", "scale", "compile_fail"];

pub(super) fn class(
    owner: &GovernedCrate,
    value: &toml::Value,
    stem: &str,
) -> Result<String, Diagnostic> {
    let court = value
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("worth"))
        .and_then(|w| w.get("court"))
        .ok_or_else(|| {
            failure(
                &owner.relative_crate_root,
                "court needs package.metadata.worth.court",
            )
        })?;
    let class = court
        .get("class")
        .and_then(|v| v.as_str())
        .filter(|v| CLASSES.contains(v))
        .ok_or_else(|| {
            failure(
                &owner.relative_crate_root,
                "court must declare exactly one class",
            )
        })?;
    let boundary = court
        .get("boundary")
        .and_then(|v| v.as_str())
        .filter(|v| segment(v))
        .ok_or_else(|| failure(&owner.relative_crate_root, "court needs its entry boundary"))?;
    let family = court.get("family").and_then(|v| v.as_str());
    if family.is_some_and(|name| !segment(name)) {
        return Err(failure(
            &owner.relative_crate_root,
            "court family must name its measured split axis",
        ));
    }
    let name = if class == "compile_fail" {
        format!("{stem}-court-compile-fail")
    } else {
        format!(
            "{stem}-court-{boundary}{}-{class}",
            family.map_or(String::new(), |f| format!("-{f}"))
        )
    };
    if owner.package != name {
        return Err(failure(
            &owner.relative_crate_root,
            format!("misnamed court: expected `{name}`"),
        ));
    }
    Ok(class.into())
}

fn segment(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

pub(super) fn product_code(owner: &GovernedCrate, value: &toml::Value) -> Vec<Diagnostic> {
    let has_target = ["lib", "bin", "example", "bench"]
        .iter()
        .any(|key| value.get(key).is_some());
    let has_build = value
        .get("package")
        .and_then(|p| p.get("build"))
        .is_some_and(|v| v.as_bool() != Some(false));
    let mut diagnostics = Vec::new();
    if has_target
        || has_build
        || owner.crate_root.join("src").exists()
        || owner.crate_root.join("build.rs").exists()
    {
        diagnostics.push(failure(
            &owner.relative_crate_root,
            "court contains product code or a production target",
        ));
    }
    let mut directories = vec![owner.crate_root.clone()];
    while let Some(directory) = directories.pop() {
        let entries = match std::fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) => {
                diagnostics.push(failure(&owner.relative_crate_root, error.to_string()));
                continue;
            }
        };
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    diagnostics.push(failure(&owner.relative_crate_root, error.to_string()));
                    continue;
                }
            };
            let path = entry.path();
            if path.is_dir() {
                if path != owner.crate_root.join("target") && path != owner.crate_root.join("tests")
                {
                    directories.push(path);
                }
            } else if path.extension().is_some_and(|e| e == "rs") {
                diagnostics.push(failure(
                    &owner.relative_crate_root,
                    format!(
                        "court contains product code outside tests/: {}",
                        path.display()
                    ),
                ));
            }
        }
    }
    diagnostics
}
