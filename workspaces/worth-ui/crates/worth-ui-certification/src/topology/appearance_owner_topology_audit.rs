use std::path::Path;

use syn::{
    parse::Parser,
    visit::{self, Visit},
    Attribute, Expr, ExprCall, ExprMethodCall, File, FnArg, Item, PatType, Path as SynPath, Type,
    TypeReference,
};

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
    "appearance_posture",
    "appearance_owner_snapshot",
    "operability_standing_snapshot",
    "validation_appearance_snapshot",
    "pointer_presence_appearance_snapshot",
    "pressed_appearance_snapshot",
];
const SEALED_OWNER_SNAPSHOT: &str = "UiAppearanceOwnerSnapshot";
const CLOSE_AUTHORITY_METHOD: &str = "seal_at_turn_close";

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
    let syntax = parse_source(source, path);
    let has_snapshot_parameter = syntax.items.iter().any(|item| {
        if is_test_only_item(item) {
            return false;
        }
        match item {
            Item::Fn(function) if function.sig.ident == "adapt" => function
                .sig
                .inputs
                .iter()
                .any(has_immutable_owner_snapshot_parameter),
            _ => false,
        }
    });
    if !has_snapshot_parameter {
        violations.push(format!(
            "{} appearance adapter must consume `&UiAppearanceOwnerSnapshot`",
            path.display()
        ));
    }
    let mut visitor = ProductionAdapterVisitor::default();
    visitor.visit_file(&syntax);
    if visitor.mutable_owner_reference {
        violations.push(format!(
            "{} appearance adapter reaches mutable state instead of the sealed owner snapshot",
            path.display()
        ));
    }
    for symbol in visitor.mutable_owner_symbols {
        violations.push(format!(
            "{} appearance adapter imports or calls mutable source owner `{symbol}`",
            path.display()
        ));
    }
    for call in visitor.direct_owner_reads {
        violations.push(format!(
            "{} appearance adapter directly reads a mutable owner through `.{call}(`",
            path.display()
        ));
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
        let syntax = parse_source(source.text(), source.absolute_path());
        let mut visitor = ProductionAdapterVisitor::default();
        visitor.visit_file(&syntax);
        production_calls += visitor.close_calls;
        if visitor.close_calls > 0 && normalize(source.relative_path()) != CLOSE_AUTHORITY_SOURCE {
            violations.push(format!(
                "{} calls `seal_at_turn_close` outside observation close",
                source.absolute_path().display()
            ));
        }
    }
    if production_calls != 1 {
        violations.push(format!(
            "expected exactly one non-test production `seal_at_turn_close` call in {CLOSE_AUTHORITY_SOURCE}, found {production_calls}"
        ));
    }
}

fn parse_source(source: &str, path: &Path) -> File {
    syn::parse_file(source).unwrap_or_else(|error| {
        panic!(
            "{} should parse for appearance topology audit: {error}",
            path.display()
        )
    })
}

#[derive(Default)]
struct ProductionAdapterVisitor {
    mutable_owner_reference: bool,
    mutable_owner_symbols: Vec<String>,
    direct_owner_reads: Vec<String>,
    close_calls: usize,
}

impl<'ast> Visit<'ast> for ProductionAdapterVisitor {
    fn visit_item(&mut self, item: &'ast Item) {
        if is_test_only_item(item) {
            return;
        }
        visit::visit_item(self, item);
    }

    fn visit_path(&mut self, path: &'ast SynPath) {
        if let Some(symbol) = mutable_owner_symbol(path) {
            if !self.mutable_owner_symbols.iter().any(|seen| seen == symbol) {
                self.mutable_owner_symbols.push(symbol.to_owned());
            }
        }
        visit::visit_path(self, path);
    }

    fn visit_type_reference(&mut self, reference: &'ast TypeReference) {
        if reference.mutability.is_some() && is_owner_type(reference.elem.as_ref()) {
            self.mutable_owner_reference = true;
        }
        visit::visit_type_reference(self, reference);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        let method = call.method.to_string();
        if DIRECT_OWNER_READS
            .iter()
            .any(|candidate| *candidate == method)
            && !self.direct_owner_reads.iter().any(|seen| seen == &method)
        {
            self.direct_owner_reads.push(method);
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        if let Expr::Path(path) = call.func.as_ref() {
            if path
                .path
                .segments
                .last()
                .is_some_and(|segment| segment.ident == CLOSE_AUTHORITY_METHOD)
            {
                self.close_calls += 1;
            }
        }
        visit::visit_expr_call(self, call);
    }
}

fn mutable_owner_symbol(path: &SynPath) -> Option<&'static str> {
    MUTABLE_OWNER_SYMBOLS
        .iter()
        .copied()
        .find(|symbol| path.segments.iter().any(|segment| segment.ident == *symbol))
}

fn is_owner_type(ty: &Type) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    let Some(ident) = path.path.segments.last().map(|segment| &segment.ident) else {
        return false;
    };
    ident == SEALED_OWNER_SNAPSHOT || mutable_owner_symbol(&path.path).is_some()
}

fn is_test_only_item(item: &Item) -> bool {
    item_attributes(item).iter().any(is_test_cfg)
}

fn is_test_cfg(attribute: &Attribute) -> bool {
    if !attribute.path().is_ident("cfg") {
        return false;
    }
    let syn::Meta::List(list) = &attribute.meta else {
        return false;
    };
    let parser = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
    parser
        .parse2(list.tokens.clone())
        .is_ok_and(|metas| metas.iter().any(cfg_meta_is_test_only))
}

fn cfg_meta_is_test_only(meta: &syn::Meta) -> bool {
    match meta {
        syn::Meta::Path(path) => path.is_ident("test"),
        syn::Meta::List(list) if list.path.is_ident("all") => {
            let parser = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
            parser
                .parse2(list.tokens.clone())
                .is_ok_and(|metas| metas.iter().any(cfg_meta_is_test_only))
        }
        syn::Meta::List(list) if list.path.is_ident("any") => {
            let parser = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
            parser
                .parse2(list.tokens.clone())
                .is_ok_and(|metas| !metas.is_empty() && metas.iter().all(cfg_meta_is_test_only))
        }
        _ => false,
    }
}

fn item_attributes(item: &Item) -> &[Attribute] {
    match item {
        Item::Const(item) => &item.attrs,
        Item::Enum(item) => &item.attrs,
        Item::ExternCrate(item) => &item.attrs,
        Item::Fn(item) => &item.attrs,
        Item::ForeignMod(item) => &item.attrs,
        Item::Impl(item) => &item.attrs,
        Item::Macro(item) => &item.attrs,
        Item::Mod(item) => &item.attrs,
        Item::Static(item) => &item.attrs,
        Item::Struct(item) => &item.attrs,
        Item::Trait(item) => &item.attrs,
        Item::TraitAlias(item) => &item.attrs,
        Item::Type(item) => &item.attrs,
        Item::Union(item) => &item.attrs,
        Item::Use(item) => &item.attrs,
        Item::Verbatim(_) => &[],
        _ => &[],
    }
}

fn is_test_source(path: &Path) -> bool {
    let path = normalize(path);
    path.contains("/tests/") || path.ends_with("_tests.rs") || path.ends_with("/tests.rs")
}

fn normalize(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_audit_ignores_comments_strings_and_cfg_test_items() {
        let source = r#"
            // &mut UiFocusRuntimeState .appearance_posture( seal_at_turn_close(
            fn adapt(snapshot: &UiAppearanceOwnerSnapshot) {
                let _text = "UiFocusRuntimeState .appearance_posture( seal_at_turn_close(";
                let _ = snapshot;
            }
            #[cfg(test)]
            mod tests {
                fn fake() {
                    let _owner: &mut UiFocusRuntimeState = todo!();
                    let _ = UiFocusRuntimeState::appearance_posture();
                    let _ = UiAppearanceOwnerSnapshot::seal_at_turn_close();
                }
            }
        "#;
        let mut violations = Vec::new();
        audit_adapter(source, Path::new("fixture.rs"), &mut violations);
        assert!(violations.is_empty(), "{}", violations.join("\n"));
    }

    #[test]
    fn adapter_audit_reports_production_owner_misuse_from_ast() {
        let source = r#"
            fn adapt(snapshot: &mut UiAppearanceOwnerSnapshot) {
                let _ = UiFocusRuntimeState::new();
                let _ = snapshot.appearance_posture();
            }
        "#;
        let mut violations = Vec::new();
        audit_adapter(source, Path::new("fixture.rs"), &mut violations);
        assert!(violations
            .iter()
            .any(|violation| { violation.contains("must consume `&UiAppearanceOwnerSnapshot`") }));
        assert!(violations
            .iter()
            .any(|violation| violation.contains("reaches mutable state")));
        assert!(violations
            .iter()
            .any(|violation| violation.contains("UiFocusRuntimeState")));
        assert!(violations
            .iter()
            .any(|violation| violation.contains(".appearance_posture(")));
    }

    #[test]
    fn adapter_audit_does_not_hide_composite_cfg_production_items() {
        let source = r#"
            fn adapt(snapshot: &UiAppearanceOwnerSnapshot) {
                let _ = snapshot;
            }
            #[cfg(any(test, feature = "fixture"))]
            fn production_when_fixture_is_enabled() {
                let _owner: &mut UiFocusRuntimeState = todo!();
            }
        "#;
        let mut violations = Vec::new();
        audit_adapter(source, Path::new("fixture.rs"), &mut violations);
        assert!(violations
            .iter()
            .any(|violation| violation.contains("reaches mutable state")));
    }

    #[test]
    fn close_authority_audit_counts_only_production_call_expressions() {
        let source = r#"
            fn production() {
                let _ = UiAppearanceOwnerSnapshot::seal_at_turn_close();
                let _text = "UiAppearanceOwnerSnapshot::seal_at_turn_close(";
            }
            #[cfg(test)]
            fn test_only() {
                let _ = UiAppearanceOwnerSnapshot::seal_at_turn_close();
            }
        "#;
        let syntax = parse_source(source, Path::new("fixture.rs"));
        let mut visitor = ProductionAdapterVisitor::default();
        visitor.visit_file(&syntax);
        assert_eq!(visitor.close_calls, 1);
    }
}
