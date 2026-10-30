pub use std::{fs, path::Path, process::Command};

pub const FORBIDDEN_DEPENDENCIES: [&str; 5] = [
    "worth-query-replay",
    "worth-query-execution",
    "worth-query-installation",
    "worth-query-admission",
    "sha2",
];

pub const FORBIDDEN_RESIDUE: [&str; 6] = [
    "BankEstateOracles",
    "EstateActorContext",
    "EstateCapabilityUse",
    "EstateDecision",
    "EstateDenial",
    "AuthorityMarker",
];

pub const FORBIDDEN_CUTOVER_RESIDUE: [&str; 12] = [
    "WorthQueryRuntimeBuilder",
    "SignalGraph",
    "BridgeConditionalProviderSet",
    "PermissionRegistry",
    "GenericCursor",
    "UndoStack",
    "schedule_temporal_wake",
    "replace_conditional_provider",
    "replace_named_clock",
    "invoke_conditional_operation",
    "delegate_estate_capability(",
    "revoke_estate_capability(",
];

pub fn production_dependency_violations(metadata: &serde_json::Value) -> Vec<String> {
    const PRODUCTION_PACKAGES: [&str; 4] = [
        "bank-domain",
        "bank-server",
        "bank-http-adapter",
        "bank-user-node",
    ];
    const FORBIDDEN: [&str; 9] = [
        "worth-query",
        "worth-query-admission",
        "worth-query-execution",
        "worth-query-installation",
        "worth-query-publication",
        "worth-query-replay",
        "worth-runtime-bridge",
        "worth-signal",
        "worth-relational",
    ];
    let mut violations = Vec::new();
    for package in metadata["packages"].as_array().into_iter().flatten() {
        let package_name = package["name"].as_str().unwrap_or_default();
        if !PRODUCTION_PACKAGES.contains(&package_name) {
            continue;
        }
        for dependency in package["dependencies"].as_array().into_iter().flatten() {
            if dependency["kind"].as_str() == Some("dev") {
                continue;
            }
            let actual_name = dependency["name"].as_str().unwrap_or_default();
            if FORBIDDEN.contains(&actual_name) {
                let local_name = dependency["rename"].as_str().unwrap_or(actual_name);
                violations.push(format!("{package_name} -> {local_name} ({actual_name})"));
            }
        }
    }
    violations.sort();
    violations
}

pub fn production_query_audience_dependencies(
    metadata: &serde_json::Value,
) -> std::collections::BTreeSet<&str> {
    const PRODUCTION_PACKAGES: [&str; 4] = [
        "bank-domain",
        "bank-server",
        "bank-http-adapter",
        "bank-user-node",
    ];
    metadata["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|package| {
            PRODUCTION_PACKAGES.contains(&package["name"].as_str().unwrap_or_default())
        })
        .flat_map(|package| package["dependencies"].as_array().into_iter().flatten())
        .filter(|dependency| dependency["kind"].as_str() != Some("dev"))
        .filter_map(|dependency| dependency["name"].as_str())
        .filter(|name| matches!(*name, "worth-query-decl" | "worth-query-host"))
        .collect()
}

pub fn inspect_rust_sources(root: &Path) {
    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let source = fs::read_to_string(&path).unwrap();
                for residue in FORBIDDEN_RESIDUE {
                    assert!(
                        !source.contains(residue),
                        "forbidden residue {residue} remains in {}",
                        path.display()
                    );
                }
                for residue in FORBIDDEN_CUTOVER_RESIDUE {
                    assert!(
                        !source.contains(residue),
                        "forbidden Phase 10 residue {residue} remains in {}",
                        path.display()
                    );
                }
            }
        }
    }
}

pub fn manifest_dir() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../bank-estate-certification"
    ))
}
