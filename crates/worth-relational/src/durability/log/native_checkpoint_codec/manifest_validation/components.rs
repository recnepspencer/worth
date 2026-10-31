//! Check exact owner-recorded component headers without reading artifact bodies.
use super::*;

#[cfg(test)]
mod tests;
use crate::branch::{
    RelationalBranchBasisDescriptor as Descriptor, RELATIONAL_BRANCH_BASIS_DESCRIPTOR_VERSION,
};

pub(super) fn validate(manifest: &RelationalCheckpointManifest) -> Result<(), DurabilityError> {
    let commits = manifest
        .commits
        .iter()
        .map(|entry| (entry.commit.commit_id, entry))
        .collect::<BTreeMap<_, _>>();
    let schemas = manifest
        .schemas
        .iter()
        .map(|image| (image.carrier_digest(), image.schema_root()))
        .collect::<BTreeMap<_, _>>();
    let mut observations = BTreeSet::new();
    for root in &manifest.roots {
        if root.recorded_components.is_empty() {
            return Err(corrupt("root has no recorded components"));
        }
        let entry = commits
            .get(&root.commit_id)
            .ok_or_else(|| corrupt("component root has no indexed commit"))?;
        let schema = schemas
            .get(&root.schema_carrier_digest)
            .ok_or_else(|| corrupt("component root has no schema"))?;
        let mut prior = None;
        for descriptor in &root.recorded_components {
            let key = validate_descriptor(descriptor, *schema)?;
            require_order(&mut prior, key)?;
            observations.insert(observation_key(
                descriptor.branch_id(),
                descriptor.reference(),
                descriptor.truth_version(),
            )?);
            let FoundationalBranchTarget::Basis(target) = descriptor.reference().target() else {
                return Err(corrupt("canonical root has Empty component"));
            };
            if target.selected_commit_id() != root.commit_id.0
                || target.version_id() != entry.commit.version_id.0
                || target.parent_commit_ids()
                    != entry
                        .commit
                        .parents
                        .iter()
                        .map(|id| id.0)
                        .collect::<Vec<_>>()
                || target.roots() != &entry.root
            {
                return Err(corrupt(
                    "component reference disagrees with exact indexed root",
                ));
            }
        }
    }
    let mut prior = None;
    for empty in &manifest.retained_empty_components {
        let schema = schemas
            .get(&empty.schema_carrier_digest)
            .ok_or_else(|| corrupt("Empty component has no schema"))?;
        require_order(&mut prior, validate_descriptor(&empty.descriptor, *schema)?)?;
        observations.insert(observation_key(
            empty.descriptor.branch_id(),
            empty.descriptor.reference(),
            empty.descriptor.truth_version(),
        )?);
        if !matches!(
            empty.descriptor.reference().target(),
            FoundationalBranchTarget::Empty
        ) || empty.descriptor.visibility_commitment() != [0; 32]
        {
            return Err(corrupt(
                "Empty component has canonical target or visibility",
            ));
        }
    }
    for cell in &manifest.branch_cells {
        let cell = cell
            .in_canonical_frame()
            .map_err(|cause| corrupt(format!("component cell frame: {cause:?}")))?;
        let found = observations.contains(&observation_key(
            &cell.branch_id,
            &cell.observation,
            cell.truth_version,
        )?);
        if !found {
            return Err(corrupt(
                "checkpoint omitted current exact component observation",
            ));
        }
    }
    Ok(())
}
fn validate_descriptor(
    descriptor: &Descriptor,
    schema: [u8; 32],
) -> Result<Vec<u8>, DurabilityError> {
    if descriptor.descriptor_version() != RELATIONAL_BRANCH_BASIS_DESCRIPTOR_VERSION
        || descriptor.in_canonical_frame().ok().as_ref() != Some(descriptor)
        || descriptor.schema_commitment() != schema
    {
        return Err(corrupt("component frame, version or schema mismatch"));
    }
    rmp_serde::to_vec_named(descriptor).map_err(|cause| corrupt(cause.to_string()))
}
fn require_order(prior: &mut Option<Vec<u8>>, key: Vec<u8>) -> Result<(), DurabilityError> {
    if prior.as_ref().is_some_and(|prior| prior >= &key) {
        return Err(corrupt(
            "component observations are duplicated or unordered",
        ));
    }
    *prior = Some(key);
    Ok(())
}

fn observation_key(
    branch: &crate::history::data::BranchId,
    reference: &crate::branch::RelationalBranchReferenceObservation,
    truth: crate::branch::RelationalBranchVersion,
) -> Result<Vec<u8>, DurabilityError> {
    rmp_serde::to_vec_named(&(branch, reference, truth)).map_err(|cause| corrupt(cause.to_string()))
}
