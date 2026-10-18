//! The ordinary audience must not regain the displaced manual mutation entrance.
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use std::path::Path;
use syn::{Item, ReturnType, Type, UseTree, Visibility};

pub(super) fn validate(root: &Path, workspace: &str) -> Result<Vec<Diagnostic>, String> {
    let owner = format!("{workspace}/crates/worth-query-publication/src/application_entry");
    let mut diagnostics = Vec::new();
    let exports = format!("{owner}/mod.rs");
    let syntax = read(root, &exports)?;
    for item in &syntax.items {
        if let Item::Use(export) = item {
            if matches!(export.vis, Visibility::Public(_)) && exports_native_request(&export.tree) {
                diagnostics.push(denial(
                    &exports,
                    "manual mutation requests belong only under application_entry::advanced",
                ));
            }
        }
    }
    let request = format!("{owner}/request.rs");
    let request_syntax = read(root, &request)?;
    for item in &request_syntax.items {
        let Item::Impl(implementation) = item else {
            continue;
        };
        if type_name(&implementation.self_ty) != Some("WorthQueryApplicationRequest") {
            continue;
        }
        for item in &implementation.items {
            let syn::ImplItem::Fn(method) = item else {
                continue;
            };
            if method.sig.ident != "mutate" {
                continue;
            }
            let ordinary = matches!(&method.sig.output, ReturnType::Type(_, ty)
                if type_name(ty) == Some("WorthQueryOperationBuilder"));
            if !ordinary {
                diagnostics.push(denial(
                    &request,
                    "ordinary mutate must return the owning operation builder",
                ));
            }
        }
    }
    validate_ordinary_methods(root, &owner, &mut diagnostics)?;
    Ok(diagnostics)
}

fn validate_ordinary_methods(
    root: &Path,
    owner: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(), String> {
    let mut directories = vec![root.join(owner)];
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(&directory).map_err(|cause| cause.to_string())? {
            let path = entry.map_err(|cause| cause.to_string())?.path();
            if path.is_dir() {
                directories.push(path);
                continue;
            }
            if path.extension().is_none_or(|extension| extension != "rs") {
                continue;
            }
            let relative = path
                .strip_prefix(root)
                .map_err(|cause| cause.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            let syntax = read(root, &relative)?;
            for item in &syntax.items {
                let Item::Impl(implementation) = item else {
                    continue;
                };
                if !matches!(
                    type_name(&implementation.self_ty),
                    Some(
                        "WorthQueryOperationBuilder"
                            | "WorthQueryPendingApplicationLane"
                            | "WorthQueryBoundWorkflowOperation"
                    )
                ) {
                    continue;
                }
                for item in &implementation.items {
                    let syn::ImplItem::Fn(method) = item else {
                        continue;
                    };
                    if matches!(method.vis, Visibility::Public(_))
                        && manual_transition(&method.sig.ident.to_string())
                        && !(type_name(&implementation.self_ty)
                            == Some("WorthQueryBoundWorkflowOperation")
                            && method.sig.ident == "execute_in_program")
                    {
                        diagnostics.push(denial(&relative,
                            "ordinary action construction cannot expose manual fixed/source-only transitions"));
                    }
                }
            }
        }
    }
    Ok(())
}

fn manual_transition(name: &str) -> bool {
    matches!(
        name,
        "execute"
            | "execute_report"
            | "execute_retained"
            | "execute_retained_in_program"
            | "execute_in_program"
            | "execute_in_program_report"
            | "execute_capability_in_program"
            | "prepare_in_program"
            | "prepare_in_program_report"
            | "prepare_program_migration"
            | "execute_performed"
            | "execute_performed_report"
            | "recover_unpublished_in_program"
            | "resolve_idempotency_in_program"
            | "recover_unpublished_required_in_program"
            | "promote_recovered_required_outputs"
            | "retry_required_output_retention"
    )
}

fn read(root: &Path, relative: &str) -> Result<std::rc::Rc<syn::File>, String> {
    let path = root.join(relative);
    let source = crate::source_documents::read_to_string(&path)
        .map_err(|cause| format!("read {}: {cause}", path.display()))?;
    crate::source_syntax::parse_named(&path, &source)
        .map_err(|cause| format!("parse {}: {cause}", path.display()))
}

fn exports_native_request(tree: &UseTree) -> bool {
    match tree {
        UseTree::Path(path) => exports_native_request(&path.tree),
        UseTree::Group(group) => group.items.iter().any(exports_native_request),
        UseTree::Name(name) => native_name(&name.ident.to_string()),
        UseTree::Rename(rename) => {
            native_name(&rename.ident.to_string()) || native_name(&rename.rename.to_string())
        }
        UseTree::Glob(_) => false,
    }
}

fn native_name(name: &str) -> bool {
    matches!(
        name,
        "WorthQueryApplicationMutationRequest"
            | "WorthQueryApplicationMutationRequestWithIdempotency"
    )
}

fn type_name(ty: &Type) -> Option<&str> {
    let Type::Path(path) = ty else { return None };
    let ident = &path.path.segments.last()?.ident;
    if ident == "WorthQueryApplicationRequest" {
        Some("WorthQueryApplicationRequest")
    } else if ident == "WorthQueryOperationBuilder" {
        Some("WorthQueryOperationBuilder")
    } else if ident == "WorthQueryPendingApplicationLane" {
        Some("WorthQueryPendingApplicationLane")
    } else if ident == "WorthQueryBoundWorkflowOperation" {
        Some("WorthQueryBoundWorkflowOperation")
    } else {
        None
    }
}

fn denial(subject: &str, message: &str) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::Bc3003QueryAudienceFacadeContract,
        subject,
        message.to_owned(),
    )
}
