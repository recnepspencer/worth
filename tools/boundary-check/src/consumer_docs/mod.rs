//! Consumer documentation classification, compilation ownership and link contracts.
mod absence_claims;
mod compiled_regions;
mod configuration;
mod debt;
mod doctest_coverage;
mod fences;
mod generated_regions;
pub(crate) mod guidance_write;
mod links;
mod native_test_cfg;
mod navigation;
mod region_compilation;
mod source_facts;
#[cfg(test)]
mod tests;

use crate::diagnostics::{Diagnostic, DiagnosticCode};
use fences::{Fence, Marker};
use std::collections::BTreeMap;
use std::path::Path;

pub(crate) fn validate(root: &Path) -> Vec<Diagnostic> {
    let config = match configuration::load(root) {
        Ok(config) => config,
        Err(error) => return vec![invalid(configuration::CONFIG, error)],
    };
    let mut diagnostics = Vec::new();
    let mut observed = BTreeMap::new();
    let mut targets = links::Targets::default();
    let mut compiled = compiled_regions::FixtureSources::default();
    for document in &config.documents {
        let text = match crate::source_documents::read_to_string(&root.join(document)) {
            Ok(text) => text,
            Err(error) => {
                diagnostics.push(invalid(document, error.to_string()));
                continue;
            }
        };
        let observation = fences::observe(&text);
        observed.insert(
            document.clone(),
            observation
                .fences
                .iter()
                .filter(|fence| fence.marker.is_none())
                .count(),
        );
        for (line, reason) in observation.invalid_markers {
            diagnostics.push(invalid(&format!("{document}:{line}"), reason));
        }
        for fence in observation.fences {
            if let Err(reason) = classify(root, &config, document, &fence, &mut compiled) {
                diagnostics.push(invalid(&format!("{document}:{}", fence.line), reason));
            }
        }
        diagnostics.extend(targets.validate(root, document, &text));
    }
    diagnostics.extend(debt::compare(root, &observed));
    diagnostics.extend(guidance_diagnostics(root, &config));
    diagnostics
}

fn classify(
    root: &Path,
    config: &configuration::Configuration,
    document: &str,
    fence: &Fence,
    compiled: &mut compiled_regions::FixtureSources,
) -> Result<(), String> {
    match &fence.marker {
        None => Ok(()),
        Some(Marker::Invalid(reason)) => Err(reason.clone()),
        Some(Marker::Illustrative) => {
            if fence.info != "rust,ignore"
                || fence.code.lines().next() != Some("// Illustrative: not compiled.")
            {
                return Err("illustrative fence requires rust,ignore and first line `// Illustrative: not compiled.`".into());
            }
            Ok(())
        }
        Some(Marker::Compiled(name)) => {
            if fence.info != "rust" {
                return Err("compiled fence requires rust without ignore or other flags".into());
            }
            compiled.validate(root, config, name)
        }
        Some(Marker::Doctest(name)) => {
            if !matches!(fence.info.as_str(), "rust" | "rust,no_run") {
                return Err("doctest fence requires rust or rust,no_run".into());
            }
            doctest_coverage::validate(root, config, document, name)
        }
    }
}

fn invalid(subject: &str, reason: String) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::Bc8008ConsumerDocMarker, subject, reason)
}

fn guidance_diagnostics(root: &Path, config: &configuration::Configuration) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if config.derive_skills {
        if let Err(error) = crate::derived_skills::check(root) {
            diagnostics.push(invalid(".grok/skills", error));
        }
    }
    match generated_regions::planned(root, config) {
        Ok(planned) => {
            for (path, expected) in planned {
                match crate::source_documents::read_to_string(&path) {
                    Ok(actual) if actual == expected => {}
                    _ => diagnostics.push(invalid(
                        &path.display().to_string(),
                        "source facts drift; run --write-guidance".into(),
                    )),
                }
            }
        }
        Err(error) => diagnostics.push(invalid(configuration::CONFIG, error)),
    }
    for result in [
        absence_claims::validate(root, config),
        navigation::validate(root, config),
    ] {
        if let Err(error) = result {
            diagnostics.push(invalid(configuration::CONFIG, error));
        }
    }
    diagnostics
}

/// Complete validation/planning precedes writes; no debt or source is generated.
pub(crate) fn write_guidance(root: &Path) -> Result<(), String> {
    let config = configuration::load(root)?;
    let regions = generated_regions::planned(root, &config)?;
    let skills = if config.derive_skills {
        Some(crate::derived_skills::planned(root)?)
    } else {
        None
    };
    absence_claims::validate(root, &config)?;
    navigation::validate(root, &config)?;
    guidance_write::Plan::new(root, regions, skills)?.apply()
}
