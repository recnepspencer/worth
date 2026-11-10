//! Per-binding source guard for explicit domain-atom contracts.
mod attributes;
mod canonical_claims;
mod claims;
mod closure;
mod config;
mod macro_origin;
mod modules;
mod type_resolution;
use crate::{
    config::{diagnostic, Code, Diagnostic, Files, CONFIG},
    inputs::Inputs,
};
use quote::ToTokens;
use std::collections::{BTreeMap, BTreeSet};

pub use config::workspace_manifests;
use config::{registrations, Posture};
const REGISTRY: &str = "domain-values.toml";

pub fn check(input: &Inputs) -> Vec<Diagnostic> {
    check_registry(input, &format!("{CONFIG}{REGISTRY}"))
}

pub fn check_registry(input: &Inputs, path: &str) -> Vec<Diagnostic> {
    let roots = roots(input);
    match roots.and_then(|roots| check_sources(&input.files, roots, path)) {
        Ok(()) => Vec::new(),
        Err(errors) => errors
            .into_iter()
            .map(|error| diagnostic(Code::DomainValue, path, error))
            .collect(),
    }
}

fn roots(input: &Inputs) -> Result<BTreeMap<String, String>, Vec<String>> {
    let mut roots = BTreeMap::new();
    for graph in input.graphs.values() {
        for package in &graph.packages {
            // --no-deps omits facade packages. Only actual direct dependencies
            // authorize the existing public macro spelling; never type roots.
            if graph.resolve.is_none() {
                for dependency in &package.dependencies {
                    if dependency.name == "worth-query-decl" {
                        roots
                            .entry("worth_query_decl".into())
                            .or_insert_with(|| "external/decl/lib.rs".into());
                    }
                }
            }
            for target in &package.targets {
                if target.kind.iter().any(|kind| kind == "lib")
                    && (input.files.contains_key(&target.src_path)
                        || ["worth_query_decl", "worth_query_declaration"]
                            .contains(&target.name.as_str()))
                {
                    if roots
                        .insert(target.name.clone(), target.src_path.clone())
                        .is_some_and(|previous| previous != target.src_path)
                    {
                        return Err(vec![format!(
                            "ambiguous Cargo library root {}",
                            target.name
                        )]);
                    }
                }
            }
        }
    }
    Ok(roots)
}

fn check_sources(
    files: &Files,
    roots: BTreeMap<String, String>,
    registry: &str,
) -> Result<(), Vec<String>> {
    let registrations = registrations(files, registry).map_err(|error| vec![error])?;
    let sources = modules::Sources { files, roots };
    let mut claims = Vec::new();
    let mut errors = Vec::new();
    for (source, text) in files {
        if source.ends_with(".rs")
            && [
                "binary",
                "domain_value",
                "DOMAIN_VALUE",
                "domain",
                "ApplicationCanonicalValueContract",
                "canonical_contract",
            ]
            .iter()
            .any(|marker| text.contains(marker))
        {
            match claims::discover(source, text) {
                Ok(found) => claims.extend(found),
                Err(error) => errors.push(error),
            }
        }
    }
    let mut used = BTreeSet::new();
    for claim in claims {
        let binding = claim
            .modules
            .iter()
            .cloned()
            .chain([claim.binding.clone()])
            .collect::<Vec<_>>()
            .join("::");
        let matches: Vec<_> = registrations
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.source == claim.source && entry.binding == binding)
            .collect();
        if matches.len() != 1 {
            errors.push(format!(
                "{}::{binding}: explicit domain claim needs exactly one binding registration (unregistered value path {})",
                claim.source, claim.value.to_token_stream()
            ));
            continue;
        }
        let (index, registration) = matches[0];
        if !used.insert(index) {
            errors.push(format!(
                "{}::{binding}: ambiguous duplicate domain claim",
                claim.source
            ));
            continue;
        }
        let result = sources.find_source(&claim.source).and_then(|mut context| {
            context.extend(claim.modules);
            sources.certify_context(&context)?;
            if let Some(origin) = &claim.origin {
                sources.macro_origin(&context, origin, 0)?;
            }
            if matches!(registration.posture, Posture::Diagnostic) {
                Ok(Vec::new())
            } else {
                closure::check(&sources, &context, &claim.value, &registration.forbidden)
            }
        });
        if let Err(error) = result {
            errors.push(format!("{}::{binding}: {error}", claim.source));
        }
    }
    for (index, registration) in registrations.iter().enumerate() {
        if !used.contains(&index) {
            errors.push(format!(
                "{}::{}: stale registration has no actual domain claim",
                registration.source, registration.binding
            ));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests;
