use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::workspace_source_inventory::WorkspaceSourceInventory;
use syn::{Item, UseTree, Visibility};

mod role_purity_inventory;

const FORBIDDEN_PUBLIC_MODULE_NAMES: [&str; 7] = [
    "internal", "common", "helpers", "utils", "data", "manager", "debug",
];

pub fn audit_inspection_public_module_names(inventory: &WorkspaceSourceInventory) -> Vec<String> {
    let mut violations = Vec::new();

    for declaration in collect_public_module_declarations(inventory) {
        if FORBIDDEN_PUBLIC_MODULE_NAMES.contains(&declaration.module_name.as_str()) {
            violations.push(format!(
                "{} exposes forbidden public module `{}`",
                declaration.declaring_file.display(),
                declaration.module_name
            ));
        }
    }

    violations.sort();
    violations.dedup();
    violations
}

pub fn audit_inspection_public_module_role_purity(
    inventory: &WorkspaceSourceInventory,
) -> Vec<String> {
    let inspection_root = inventory.absolute_path("crates/worth-ui-inspection/src");
    let expected_exports = role_purity_inventory::expected_exports();
    let mut violations = Vec::new();

    for (relative_path, expected_names) in expected_exports {
        let path = inspection_root.join(relative_path);
        let actual_names = collect_public_export_names(inventory, &path);
        if actual_names != expected_names {
            violations.push(format!(
                "{} exports {:?}; expected {:?} for its single public responsibility",
                path.display(),
                actual_names,
                expected_names
            ));
        }
    }

    for declaration in collect_public_module_declarations(inventory) {
        violations.push(format!(
            "{} introduces public child module `{}`; inspection topology must stay on the curated root re-export surface instead of growing nested public module trees by default",
            declaration.declaring_file.display(),
            declaration.module_name
        ));
    }

    violations.sort();
    violations.dedup();
    violations
}

pub fn audit_inspection_future_artifact_seed_topology(
    inventory: &WorkspaceSourceInventory,
) -> Vec<String> {
    let inspection_root = inventory.absolute_path("crates/worth-ui-inspection/src");
    let receipt_mod = inspection_root.join("receipt/mod.rs");
    let expected_seed_modules = [
        ("evidence", inspection_root.join("receipt/evidence/mod.rs")),
        ("replay", inspection_root.join("receipt/replay/mod.rs")),
        ("snapshot", inspection_root.join("receipt/snapshot/mod.rs")),
    ];
    let evidence_mod = inspection_root.join("receipt/evidence/mod.rs");
    let expected_evidence_seed_modules = [
        (
            "measurement",
            inspection_root.join("receipt/evidence/measurement/mod.rs"),
        ),
        (
            "mounting",
            inspection_root.join("receipt/evidence/mounting/mod.rs"),
        ),
        (
            "inspector",
            inspection_root.join("receipt/evidence/inspector/mod.rs"),
        ),
    ];
    let mut violations = missing_seed_module_violations(
        inventory,
        &expected_seed_modules,
        "inspection artifacts lack an honest internal home",
    );
    violations.extend(private_seed_declaration_violations(
        inventory,
        &receipt_mod,
        &expected_seed_modules,
        "inspection landing zone",
    ));
    violations.extend(missing_seed_module_violations(
        inventory,
        &expected_evidence_seed_modules,
        "evidence lacks one obvious typed substrate home",
    ));
    violations.extend(private_seed_declaration_violations(
        inventory,
        &evidence_mod,
        &expected_evidence_seed_modules,
        "evidence landing zone",
    ));

    violations.sort();
    violations.dedup();
    violations
}

fn missing_seed_module_violations(
    inventory: &WorkspaceSourceInventory,
    expected: &[(&str, PathBuf)],
    responsibility: &str,
) -> Vec<String> {
    expected
        .iter()
        .filter(|(_, module_path)| inventory.source(module_path).is_none())
        .map(|(module_name, module_path)| {
            format!(
                "{} is missing; future {module_name} {responsibility}",
                module_path.display()
            )
        })
        .collect()
}

fn private_seed_declaration_violations(
    inventory: &WorkspaceSourceInventory,
    declaring_module: &Path,
    expected: &[(&str, PathBuf)],
    responsibility: &str,
) -> Vec<String> {
    let parsed = parse_rust_file(inventory, declaring_module);
    expected
        .iter()
        .filter(|(module_name, _)| {
            !parsed.items.iter().any(|item| match item {
                Item::Mod(item_mod) => {
                    item_mod.ident == **module_name
                        && !matches!(item_mod.vis, Visibility::Public(_))
                }
                _ => false,
            })
        })
        .map(|(module_name, _)| {
            format!(
                "{} must declare a private `{module_name}` child module as the future {module_name} {responsibility}",
                declaring_module.display()
            )
        })
        .collect()
}

fn collect_public_export_names(
    inventory: &WorkspaceSourceInventory,
    path: &Path,
) -> BTreeSet<String> {
    let parsed = parse_rust_file(inventory, path);
    let mut names = BTreeSet::new();

    for item in parsed.items {
        match item {
            Item::Use(item_use) if matches!(item_use.vis, Visibility::Public(_)) => {
                collect_public_use_names(&item_use.tree, &mut names);
            }
            Item::Struct(item_struct) if matches!(item_struct.vis, Visibility::Public(_)) => {
                names.insert(item_struct.ident.to_string());
            }
            Item::Enum(item_enum) if matches!(item_enum.vis, Visibility::Public(_)) => {
                names.insert(item_enum.ident.to_string());
            }
            Item::Fn(item_fn) if matches!(item_fn.vis, Visibility::Public(_)) => {
                names.insert(item_fn.sig.ident.to_string());
            }
            Item::Const(item_const) if matches!(item_const.vis, Visibility::Public(_)) => {
                names.insert(item_const.ident.to_string());
            }
            _ => {}
        }
    }

    names
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PublicModuleDeclaration {
    declaring_file: PathBuf,
    module_name: String,
}

fn collect_public_module_declarations(
    inventory: &WorkspaceSourceInventory,
) -> Vec<PublicModuleDeclaration> {
    let mut declarations = Vec::new();

    for source in inventory.rust_files_under("crates/worth-ui-inspection/src") {
        let path = source.absolute_path();
        let parsed = parse_rust_file(inventory, path);
        for item in parsed.items {
            if let Item::Mod(item_mod) = item
                && matches!(item_mod.vis, Visibility::Public(_))
            {
                declarations.push(PublicModuleDeclaration {
                    declaring_file: path.to_path_buf(),
                    module_name: item_mod.ident.to_string(),
                });
            }
        }
    }

    declarations.sort_by(|left, right| {
        left.declaring_file
            .cmp(&right.declaring_file)
            .then(left.module_name.cmp(&right.module_name))
    });
    declarations.dedup();
    declarations
}

fn collect_public_use_names(tree: &UseTree, output: &mut BTreeSet<String>) {
    match tree {
        UseTree::Name(name) => {
            output.insert(name.ident.to_string());
        }
        UseTree::Rename(rename) => {
            output.insert(rename.rename.to_string());
        }
        UseTree::Group(group) => {
            for item in &group.items {
                collect_public_use_names(item, output);
            }
        }
        UseTree::Path(path) => collect_public_use_names(&path.tree, output),
        UseTree::Glob(_) => {}
    }
}

fn parse_rust_file(inventory: &WorkspaceSourceInventory, path: &Path) -> syn::File {
    let text = inventory.text(path);
    syn::parse_file(text).unwrap_or_else(|error| {
        panic!("{} should parse as Rust source: {error}", path.display());
    })
}
