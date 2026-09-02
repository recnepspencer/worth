use std::path::Path;

use syn::{FnArg, Item, PatType, Type};

use super::WorkspaceSourceInventory;

const APPEARANCE_ADAPTERS: [&str; 6] = [
    "crates/worth-ui-runtime/src/runtime/appearance/state/operability.rs",
    "crates/worth-ui-runtime/src/runtime/appearance/state/focus.rs",
    "crates/worth-ui-runtime/src/runtime/appearance/state/validation.rs",
    "crates/worth-ui-runtime/src/runtime/appearance/state/selection.rs",
    "crates/worth-ui-runtime/src/runtime/appearance/state/hover.rs",
    "crates/worth-ui-runtime/src/runtime/appearance/state/pressed.rs",
];
const RUNTIME_SOURCE_ROOT: &str = "crates/worth-ui-runtime/src";
const CLOSE_AUTHORITY_SOURCE: &str =
    "crates/worth-ui-runtime/src/runtime/observation/turn/appearance_close.rs";
const MUTABLE_OWNER_SYMBOLS: [&str; 6] = [
    "UiFocusRuntimeState",
    "UiSelectionRuntimeState",
    "UiIntentAdmissionState",
    "UiIntentApplicationFactState",
    "UiInteractionRuntimeState",
    "UiPointerPresenceOwner",
];
const DIRECT_OWNER_READS: [&str; 6] = [
    ".appearance_posture(",
    ".appearance_owner_snapshot(",
    ".operability_standing_snapshot(",
    ".validation_appearance_snapshot(",
    ".pointer_presence_appearance_snapshot(",
    ".pressed_appearance_snapshot(",
];

pub fn audit_appearance_owner_export_topology(inventory: &WorkspaceSourceInventory) -> Vec<String> {
    let mut violations = Vec::new();
    for relative_path in APPEARANCE_ADAPTERS {
        let Some(source) = inventory.source(relative_path) else {
            violations.push(format!(
                "missing appearance adapter source `{relative_path}`"
            ));
            continue;
        };
        audit_adapter(source.text(), source.absolute_path(), &mut violations);
    }
    audit_close_authority(inventory, &mut violations);
    violations.sort();
    violations.dedup();
    violations
}

fn audit_adapter(source: &str, path: &Path, violations: &mut Vec<String>) {
    let syntax = syn::parse_file(source).unwrap_or_else(|error| {
        panic!(
            "{} should parse for appearance topology audit: {error}",
            path.display()
        )
    });
    let has_snapshot_parameter = syntax.items.iter().any(|item| match item {
        Item::Fn(function) if function.sig.ident == "adapt" => function
            .sig
            .inputs
            .iter()
            .any(has_immutable_owner_snapshot_parameter),
        _ => false,
    });
    if !has_snapshot_parameter {
        violations.push(format!(
            "{} appearance adapter must consume `&UiAppearanceOwnerSnapshot`",
            path.display()
        ));
    }
    if source.contains("&mut") {
        violations.push(format!(
            "{} appearance adapter reaches mutable state instead of the sealed owner snapshot",
            path.display()
        ));
    }
    for symbol in MUTABLE_OWNER_SYMBOLS {
        if source.contains(symbol) {
            violations.push(format!(
                "{} appearance adapter imports or calls mutable source owner `{symbol}`",
                path.display()
            ));
        }
    }
    for call in DIRECT_OWNER_READS {
        if source.contains(call) {
            violations.push(format!(
                "{} appearance adapter directly reads a mutable owner through `{call}`",
                path.display()
            ));
        }
    }
}

fn has_immutable_owner_snapshot_parameter(argument: &FnArg) -> bool {
    let FnArg::Typed(PatType { ty, .. }) = argument else {
        return false;
    };
    let Type::Reference(reference) = ty.as_ref() else {
        return false;
    };
    if reference.mutability.is_some() {
        return false;
    }
    let Type::Path(path) = reference.elem.as_ref() else {
        return false;
    };
    path.path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "UiAppearanceOwnerSnapshot")
}

fn audit_close_authority(inventory: &WorkspaceSourceInventory, violations: &mut Vec<String>) {
    let mut production_calls = 0;
    for source in inventory.rust_files_under(RUNTIME_SOURCE_ROOT) {
        if is_test_source(source.relative_path()) {
            continue;
        }
        for line in source.text().lines() {
            if !line.contains("seal_at_turn_close(") || line.contains("fn seal_at_turn_close(") {
                continue;
            }
            production_calls += 1;
            if normalize(source.relative_path()) != CLOSE_AUTHORITY_SOURCE {
                violations.push(format!(
                    "{} calls `seal_at_turn_close` outside observation close",
                    source.absolute_path().display()
                ));
            }
        }
    }
    if production_calls != 1 {
        violations.push(format!(
            "expected exactly one non-test production `seal_at_turn_close` call in {CLOSE_AUTHORITY_SOURCE}, found {production_calls}"
        ));
    }
}

fn is_test_source(path: &Path) -> bool {
    let path = normalize(path);
    path.contains("/tests/") || path.ends_with("_tests.rs") || path.ends_with("/tests.rs")
}

fn normalize(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
