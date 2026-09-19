use super::governed_crate::{
    discover_born_crates, discover_context_crates, parse_governed_crate_identity, DiscoveredCrate,
};
use super::model::CrateOrientation;
use super::query_audience::{
    framework_audience_orientations, framework_certification_orientation,
    query_machine_fences_for_band,
};
use super::source_surface::{collect_owned_modules, ensure_facade_only_public_surface};
use crate::authority_inputs::{
    load_orientation_contract, CommittedFacadeSnapshot, OrientationContract,
};
use std::path::Path;

pub(crate) fn load_orientations(
    root: &Path,
    config_path: &Path,
) -> Result<Vec<CrateOrientation>, String> {
    let contract = load_orientation_contract(config_path)?;
    let facade_snapshot = CommittedFacadeSnapshot::load(root)?;
    let discovered = discover_born_crates(root, &contract.subworkspace_paths)?;
    let mut orientations = discovered
        .into_iter()
        .map(|born| project_governed_orientation(root, born, &contract, &facade_snapshot))
        .collect::<Result<Vec<_>, String>>()?;
    for workspace in &contract.context_workspaces {
        orientations.extend(
            discover_context_crates(root, workspace)?
                .into_iter()
                .map(|born| project_context_orientation(root, born, workspace, &contract))
                .collect::<Result<Vec<_>, String>>()?,
        );
    }
    orientations.extend(framework_audience_orientations(
        root,
        &contract.query_audience,
        &contract.machine_constitution,
        &facade_snapshot,
    )?);
    if let Some(certification) = framework_certification_orientation(
        root,
        &contract.query_audience,
        &contract.machine_constitution,
        &facade_snapshot,
    )? {
        orientations.push(certification);
    }
    orientations.sort_by(|left, right| left.crate_name.cmp(&right.crate_name));
    Ok(orientations)
}

fn project_context_orientation(
    root: &Path,
    discovered: DiscoveredCrate,
    workspace: &crate::authority_inputs::ContextWorkspaceSpec,
    contract: &OrientationContract,
) -> Result<CrateOrientation, String> {
    let crate_root = root.join(&discovered.relative_path);
    let domain = discovered
        .package
        .strip_prefix(&workspace.package_prefix)
        .and_then(|suffix| suffix.strip_prefix('-'))
        .unwrap_or("facade")
        .to_owned();
    Ok(CrateOrientation {
        crate_name: discovered.package,
        relative_path: discovered.relative_path,
        constitutional_class: "worth/ui".to_owned(),
        domain,
        exemplar_role: "WORTH UI workspace-owned implementation surface.".to_owned(),
        deferred_routes: Vec::new(),
        public_surface: "workspace-owned; package targets remain the explicit export or composition owners".to_owned(),
        allowed_target_bands: vec!["WORTH UI manifest-declared dependencies".to_owned()],
        facade_exports: Vec::new(),
        owned_modules: collect_owned_modules(&crate_root.join("src"))?,
        machine_fences: vec![
            "Must not depend on worthy-* crates.".to_owned(),
            format!(
                "Replay dependencies are admitted only for configured certification packages: {}.",
                workspace.certification_packages.join(", ")
            ),
            "Production dependencies on the direct Query engine remain confined by the configured Worth UI Query edge; certification-only test dependencies are outside that production fence."
                .to_owned(),
        ],
        skeleton_fence: "No Road 1 seed skeleton applies; WORTH UI topology is workspace-owned and mechanically discovered.".to_owned(),
        machine_constitution: contract.machine_constitution.clone(),
    })
}

fn project_governed_orientation(
    root: &Path,
    born: DiscoveredCrate,
    contract: &OrientationContract,
    facade_snapshot: &CommittedFacadeSnapshot,
) -> Result<CrateOrientation, String> {
    let crate_root = root.join(&born.relative_path);
    let identity = parse_governed_crate_identity(&born.package)?;
    let exemplar = contract.route_proof.get(&born.package);
    let facade_exports = facade_snapshot.exports_for(&born.package)?;
    let owned_modules = collect_owned_modules(&crate_root.join("src"))?;
    ensure_facade_only_public_surface(&crate_root.join("src/lib.rs"))?;

    let mut machine_fences = Vec::new();
    if identity.tier == "worth" {
        machine_fences.push("Must not depend on worthy-* crates.".to_owned());
    }
    machine_fences.extend(query_machine_fences_for_band(
        &identity.band,
        &contract.query_audience,
    ));
    if identity.band != "cert" {
        machine_fences.push(format!(
            "Must not depend on replay surface families such as {}.",
            contract.replay_surface_summary
        ));
    }

    Ok(CrateOrientation {
        crate_name: born.package,
        relative_path: born.relative_path.clone(),
        constitutional_class: format!("{}/{}", identity.tier, identity.band),
        domain: identity.domain,
        exemplar_role: exemplar
            .map(|value| value.specimen.clone())
            .unwrap_or_else(|| "No exemplar route assigned yet.".to_owned()),
        deferred_routes: exemplar
            .map(|value| value.deferred_routes.clone())
            .unwrap_or_default(),
        public_surface: "facade-only".to_owned(),
        allowed_target_bands: contract
            .band_rules
            .get(&identity.band)
            .cloned()
            .unwrap_or_default(),
        facade_exports,
        owned_modules,
        machine_fences,
        skeleton_fence: skeleton_fence(&born.relative_path, contract),
        machine_constitution: contract.machine_constitution.clone(),
    })
}

fn skeleton_fence(relative_path: &str, contract: &OrientationContract) -> String {
    if contract
        .seed_skeleton_paths
        .iter()
        .any(|path| path == relative_path)
    {
        "Seed skeleton is machine-fenced by boundary-check; undeclared files and mixed-class modules are denied."
            .to_owned()
    } else {
        "No seed-specific skeleton allowlist is declared for this born crate; general Road 1 boundary law still applies."
            .to_owned()
    }
}
