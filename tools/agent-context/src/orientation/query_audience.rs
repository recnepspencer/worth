//! Orientation construction for configured Query audience facade crates.

use super::model::CrateOrientation;
use crate::authority_inputs::{facade_exports, QueryAudienceContractSpec, QueryAudienceFacadeSpec};
use std::path::Path;

pub(crate) fn framework_audience_orientations(
    root: &Path,
    contract: &QueryAudienceContractSpec,
    machine_constitution: &str,
) -> Result<Vec<CrateOrientation>, String> {
    contract
        .audiences
        .iter()
        .map(|audience| {
            let relative_path = Path::new(&contract.workspace)
                .join("crates")
                .join(&audience.package)
                .to_string_lossy()
                .replace('\\', "/");
            let crate_root = root.join(&relative_path);
            if !crate_root.is_dir() {
                return Err(format!(
                    "configured Query audience facade missing at {relative_path}"
                ));
            }
            let facade_exports = facade_exports(&crate_root)?;
            let machine_fences = vec![
                format!(
                    "Framework Query audience facade (`{}`); legal consuming bands: {}.",
                    audience.label,
                    render_bands(&audience.allowed_bands)
                ),
                format!(
                    "May depend only on its configured authority packages: {}; must not depend on other audience facades.",
                    render_packages(&audience.authority_packages)
                ),
                format!(
                    "Leaf re-export surface only; guidance: {}.",
                    audience.guidance
                ),
            ];

            Ok(CrateOrientation {
                crate_name: audience.package.clone(),
                relative_path,
                constitutional_class: "framework/query-audience".to_owned(),
                domain: audience.label.clone(),
                exemplar_role: format!(
                    "Query {} audience facade over {}",
                    audience.label,
                    render_packages(&audience.authority_packages)
                ),
                deferred_routes: Vec::new(),
                public_surface: "facade-only".to_owned(),
                allowed_target_bands: Vec::new(),
                facade_exports,
                owned_modules: Vec::new(),
                machine_fences,
                skeleton_fence: "Framework audience facade: re-export-only; no seed-skeleton allowlist.".to_owned(),
                machine_constitution: machine_constitution.to_owned(),
            })
        })
        .collect()
}

pub(crate) fn framework_certification_orientation(
    root: &Path,
    contract: &QueryAudienceContractSpec,
    machine_constitution: &str,
) -> Result<Option<CrateOrientation>, String> {
    let Some(certification_package) = &contract.certification_package else {
        return Ok(None);
    };
    let relative_path = Path::new(&contract.workspace)
        .join("crates")
        .join(certification_package)
        .to_string_lossy()
        .replace('\\', "/");
    if !root.join(&relative_path).is_dir() {
        return Err(format!(
            "configured Query certification package missing at {relative_path}"
        ));
    }
    let facade_exports = facade_exports(&root.join(&relative_path))?;

    Ok(Some(CrateOrientation {
        crate_name: certification_package.clone(),
        relative_path,
        constitutional_class: "framework/query-certification".to_owned(),
        domain: "certification".to_owned(),
        exemplar_role: format!(
            "Cold Query compiler and hostile certification over {}",
            render_packages(&contract.certification_authority_packages)
        ),
        deferred_routes: Vec::new(),
        public_surface: "facade-only certification contracts".to_owned(),
        allowed_target_bands: Vec::new(),
        facade_exports,
        owned_modules: vec![
            "scenario".to_owned(),
            "evidence".to_owned(),
            "oracle".to_owned(),
        ],
        machine_fences: vec![
            format!(
                "Cold certification facade over {}; ordinary Query packages must not depend on it.",
                render_packages(&contract.certification_authority_packages)
            ),
            "Selected explicitly for compiler, replay, or hostile certification; absent from the ordinary workspace default members.".to_owned(),
            format!(
                "Configured downstream certification owners: {}; all other consumers are denied.",
                render_packages(&contract.certification_consumers)
            ),
            "May expose provider-neutral scenario, evidence, and oracle contracts; must not expose pre-solved Query authority constructors or production runner state.".to_owned(),
        ],
        skeleton_fence: "Framework certification leaf: cert-only semantic scenarios and hostile evidence; no product authority.".to_owned(),
        machine_constitution: machine_constitution.to_owned(),
    }))
}

pub(crate) fn query_machine_fences_for_band(
    band: &str,
    contract: &QueryAudienceContractSpec,
) -> Vec<String> {
    let mut fences = vec![format!(
        "Must not depend on Query engine `{}` directly; consume only through configured audience facades.",
        contract.engine_package
    )];

    let legal: Vec<&QueryAudienceFacadeSpec> = contract
        .audiences
        .iter()
        .filter(|audience| audience.allowed_bands.iter().any(|allowed| allowed == band))
        .collect();

    if legal.is_empty() {
        fences.push(
            "No Query audience facade is legal for this band; derived and other ordinary bands have no Query audience in this milestone."
                .to_owned(),
        );
    } else {
        fences.push(format!(
            "Legal Query audience facades for this band: {}.",
            legal
                .iter()
                .map(|audience| format!("`{}` ({})", audience.package, audience.guidance))
                .collect::<Vec<_>>()
                .join("; ")
        ));
    }

    for audience in &contract.audiences {
        if !audience.allowed_bands.iter().any(|allowed| allowed == band) {
            fences.push(format!(
                "Must not depend on Query audience facade `{}` (allowed bands: {}).",
                audience.package,
                render_bands(&audience.allowed_bands)
            ));
        }
    }

    fences
}

fn render_bands(bands: &[String]) -> String {
    if bands.is_empty() {
        return "none".to_owned();
    }
    bands.join(", ")
}

fn render_packages(packages: &[String]) -> String {
    packages
        .iter()
        .map(|package| format!("`{package}`"))
        .collect::<Vec<_>>()
        .join(", ")
}
