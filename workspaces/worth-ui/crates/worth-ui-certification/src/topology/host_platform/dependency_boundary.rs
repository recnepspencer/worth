use std::path::Path;

const RETIRED_PACKAGES: [&str; 4] = ["egui", "eframe", "egui-wgpu", "egui_extras"];

pub(super) fn audit_manifest(path: &Path, text: &str, failures: &mut Vec<String>) {
    let Ok(document) = text.parse::<toml::Table>().map(toml::Value::Table) else {
        failures.push(format!("{} is not valid TOML", path.display()));
        return;
    };
    visit_manifest_value(path, &document, false, failures);
}

fn visit_manifest_value(
    path: &Path,
    value: &toml::Value,
    dependency_section: bool,
    failures: &mut Vec<String>,
) {
    let Some(table) = value.as_table() else {
        return;
    };
    for (key, child) in table {
        let is_dependency_section = matches!(
            key.as_str(),
            "dependencies" | "dev-dependencies" | "build-dependencies"
        );
        if dependency_section && retired_package(key) {
            failures.push(format!(
                "{} declares retired dependency `{key}`",
                path.display()
            ));
        }
        if dependency_section {
            let package = child
                .as_table()
                .and_then(|entry| entry.get("package"))
                .and_then(toml::Value::as_str);
            if package.is_some_and(retired_package) {
                failures.push(format!(
                    "{} declares retired package alias `{key}`",
                    path.display()
                ));
            }
        }
        visit_manifest_value(path, child, is_dependency_section, failures);
    }
}

pub(super) fn audit_lockfile(path: &Path, text: &str, failures: &mut Vec<String>) {
    let Ok(document) = text.parse::<toml::Table>().map(toml::Value::Table) else {
        failures.push(format!("{} is not valid lockfile TOML", path.display()));
        return;
    };
    let packages = document
        .get("package")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten();
    for package in packages {
        let Some(name) = package.get("name").and_then(toml::Value::as_str) else {
            continue;
        };
        if retired_package(name) {
            failures.push(format!(
                "{} resolves retired package `{name}`",
                path.display()
            ));
        }
    }
}

fn retired_package(package: &str) -> bool {
    RETIRED_PACKAGES.contains(&package)
}

pub(super) fn mutation_controls() -> Vec<String> {
    let mut failures = Vec::new();
    audit_manifest(
        Path::new("mutant-feature/Cargo.toml"),
        "[target.'cfg(windows)'.dependencies]\neframe = '1'\n",
        &mut failures,
    );
    audit_manifest(
        Path::new("mutant-alias/Cargo.toml"),
        "[workspace.dependencies]\nrenderer = { package = 'egui-wgpu', version = '1' }\n",
        &mut failures,
    );
    audit_lockfile(
        Path::new("mutant-lock/Cargo.lock"),
        "[[package]]\nname = 'egui'\nversion = '1.0.0'\n",
        &mut failures,
    );
    failures
}
