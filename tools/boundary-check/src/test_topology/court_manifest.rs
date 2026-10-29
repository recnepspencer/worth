//! A court names a dependency surface and has no production compilation target.
use super::failure;
use crate::{diagnostics::Diagnostic, source_rules::GovernedCrate};

pub(super) fn surface(
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
    if court.get("class").is_some() {
        return Err(failure(
            &owner.relative_crate_root,
            "court categories belong in module paths, not a manifest class",
        ));
    }
    if value
        .get("package")
        .and_then(|p| p.get("autotests"))
        .and_then(|v| v.as_bool())
        != Some(false)
    {
        return Err(failure(
            &owner.relative_crate_root,
            "court must set autotests = false",
        ));
    }
    let surface = court
        .get("surface")
        .and_then(|v| v.as_str())
        .filter(|v| segment(v))
        .ok_or_else(|| {
            failure(
                &owner.relative_crate_root,
                "court needs its dependency surface",
            )
        })?;
    let name = format!("{stem}-court-{surface}");
    let class_suffix = ["-fast", "-exhaustive", "-scale", "-compile-fail"]
        .iter()
        .any(|suffix| name.ends_with(suffix));
    if owner.package != name || class_suffix {
        return Err(failure(
            &owner.relative_crate_root,
            format!("misnamed court: expected `{name}`"),
        ));
    }
    Ok(surface.into())
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
