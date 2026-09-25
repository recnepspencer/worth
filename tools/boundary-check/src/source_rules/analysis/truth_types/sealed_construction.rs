//! Sealed truth types are constructed only by their owners.
//!
//! Private fields keep a literal inside the owner, but a crate-visible
//! constructor can be called anywhere in the crate. The rule reads each owner
//! for its constructors: every associated function without a receiver, and
//! every associated constant, that yields the type, and every variant of a
//! sealed enum. Outside the owner such a constructor is legal only at a caller
//! the configuration declares for it. The rule also rejects, outside the
//! owner, a literal or tuple construction, an impl of a sealed type, and an
//! import or alias that would name a sealed type's constructors by another
//! path. A mint called as a method is resolved by Clippy's
//! `disallowed-methods`; here its declared callers are the only sites that may
//! carry the `expect` admitting it.

mod owners;
mod sites;

use super::super::production_scope::{site_holds, ProductionModule};
use crate::config::{SealedMintConfig, TruthTypeDenialConfig};
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use owners::Declared;
use sites::SiteKind;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn check(
    modules: &[ProductionModule<'_>],
    rule: &TruthTypeDenialConfig,
    disallowed_methods: &BTreeSet<String>,
) -> Vec<Diagnostic> {
    let owners: BTreeMap<&str, &[String]> = rule
        .sealed
        .iter()
        .map(|sealed| (sealed.name.as_str(), sealed.owners.as_slice()))
        .collect();
    let declared = owners::declared(&owners, modules);
    let mut used: Vec<Vec<bool>> = rule
        .mints
        .iter()
        .map(|mint| vec![false; mint.callers.len()])
        .collect();
    let mut diagnostics = Vec::new();
    let finding = |source: &str, line: usize, message: String| {
        Diagnostic::new(
            DiagnosticCode::Bc7005SealedTruthConstruction,
            format!("{}/{source}:{line}", rule.crate_root),
            format!("{message}: {}", rule.guidance),
        )
    };
    for module in modules {
        let sites = sites::read(&owners, &declared.constructors, module);
        for site in sites {
            let admitted = |mint: &SealedMintConfig, clippy: bool| {
                mint.clippy.is_some() == clippy && (clippy || mint.call == site.call)
            };
            let caller = rule.mints.iter().enumerate().find_map(|(index, mint)| {
                let clippy = matches!(site.kind, SiteKind::Suppression { .. });
                admitted(mint, clippy)
                    .then(|| {
                        mint.callers.iter().position(|caller| {
                            module.sources().any(|source| {
                                site_holds(&caller.path, &caller.items, source, &site.item)
                            })
                        })
                    })
                    .flatten()
                    .map(|caller| (index, caller))
            });
            // A file's inner attributes stand in no item.
            let named = if site.item.is_empty() {
                String::from("the module")
            } else {
                format!("`{}`", site.item)
            };
            let message = match (site.kind, caller) {
                (SiteKind::Refused(message), _) => Some(message),
                (kind, Some((mint, caller))) => {
                    used[mint][caller] = true;
                    matches!(kind, SiteKind::Suppression { expect: false }).then(|| format!(
                        "{named} admits a declared mint with `allow`; use `expect`, so Clippy reports it once the call is gone"
                    ))
                }
                (SiteKind::Suppression { .. }, None) => Some(format!(
                    "{named} suppresses Clippy's disallowed methods outside every declared mint caller"
                )),
                (SiteKind::Call, None) => Some(format!(
                    "{named} calls constructor `{}` outside its owner at an undeclared site",
                    site.call
                )),
            };
            if let Some(message) = message {
                diagnostics.push(finding(module.relative_source, site.line, message));
            }
        }
    }
    diagnostics.extend(stale_declarations(
        rule,
        &declared,
        &used,
        disallowed_methods,
    ));
    diagnostics
}

fn stale_declarations(
    rule: &TruthTypeDenialConfig,
    declared: &Declared,
    used: &[Vec<bool>],
    disallowed_methods: &BTreeSet<String>,
) -> Vec<Diagnostic> {
    let stale = |subject: String, message: String| {
        Diagnostic::new(
            DiagnosticCode::Bc7005SealedTruthConstruction,
            format!("{}/{subject}", rule.crate_root),
            message,
        )
    };
    let mut diagnostics = Vec::new();
    for sealed in &rule.sealed {
        if !declared.defined.contains(&sealed.name) {
            diagnostics.push(stale(
                sealed.owners.join(", "),
                format!(
                    "requirement {} type `{}` is not defined in its declared owners",
                    sealed.requirement, sealed.name
                ),
            ));
        }
    }
    for (mint, used) in rule.mints.iter().zip(used) {
        match &mint.clippy {
            Some(path) if !disallowed_methods.contains(path) => diagnostics.push(stale(
                mint.call.clone(),
                format!(
                    "mint `{}` is enforced by Clippy, but clippy.toml does not disallow `{path}`",
                    mint.call
                ),
            )),
            Some(_) => {}
            None => {
                let constructs = mint.call.split_once("::").is_some_and(|(owner, name)| {
                    declared
                        .constructors
                        .get(owner)
                        .is_some_and(|names| names.contains(name))
                });
                if !constructs {
                    diagnostics.push(stale(
                        mint.call.clone(),
                        format!(
                            "declared mint `{}` names no constructor of a sealed type",
                            mint.call
                        ),
                    ));
                }
            }
        }
        for (caller, used) in mint.callers.iter().zip(used) {
            if !used {
                diagnostics.push(stale(
                    caller.path.clone(),
                    format!(
                        "declared caller {:?} no longer mints `{}` ({}); remove it",
                        caller.items, mint.call, mint.reason
                    ),
                ));
            }
        }
    }
    diagnostics
}

/// The sealed types whose owners hold one of `module`'s sources.
fn owned_types<'a>(
    owners: &BTreeMap<&'a str, &[String]>,
    module: &ProductionModule<'_>,
) -> BTreeSet<&'a str> {
    owners
        .iter()
        .filter(|(_, paths)| {
            paths.iter().any(|path| {
                module
                    .sources()
                    .any(|source| site_holds(path, &[], source, ""))
            })
        })
        .map(|(name, _)| *name)
        .collect()
}
