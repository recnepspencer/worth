use std::collections::BTreeSet;

use super::manifest_capture::corrupt;
use crate::durability::data::{DurabilityError, RelationalCheckpointManifest};
use crate::history::data::CommitId;
use worth_foundational::FoundationalBranchTarget;

pub(super) fn validate(
    manifest: &RelationalCheckpointManifest,
) -> Result<BTreeSet<CommitId>, DurabilityError> {
    let mut commits = BTreeSet::new();
    for (ordinal, entry) in manifest.commits.iter().enumerate() {
        if entry.position.0 != ordinal as u64 + 1
            || entry.commit.commit_id.0 == 0
            || entry
                .commit
                .parents
                .iter()
                .any(|parent| !commits.contains(parent))
            || !commits.insert(entry.commit.commit_id)
        {
            return Err(corrupt(
                "checkpoint commit index is not contiguous and acyclic",
            ));
        }
    }
    let last = manifest.commits.last();
    if manifest.covered_position.0 != last.map_or(0, |entry| entry.position.0)
        || manifest.coverage.up_to_commit.as_ref() != last.map(|entry| &entry.commit)
        || manifest.coverage.up_to_version != last.map(|entry| entry.commit.version_id)
    {
        return Err(corrupt(
            "checkpoint coverage disagrees with its commit index",
        ));
    }
    let mut schemas = BTreeSet::new();
    for schema in &manifest.schemas {
        schema.readmit_registry().map_err(corrupt)?;
        if !schemas.insert(schema.carrier_digest()) {
            return Err(corrupt("duplicate checkpoint schema carrier"));
        }
    }
    let heads = manifest
        .branch_cells
        .iter()
        .filter_map(|cell| match cell.observation.target() {
            FoundationalBranchTarget::Basis(target) => Some(CommitId(target.selected_commit_id())),
            FoundationalBranchTarget::Empty => None,
        })
        .collect::<BTreeSet<_>>();
    let mut roots = BTreeSet::new();
    for root in &manifest.roots {
        if !heads.contains(&root.commit_id)
            || !commits.contains(&root.commit_id)
            || !schemas.contains(&root.schema_carrier_digest)
            || !roots.insert(root.commit_id)
        {
            return Err(corrupt(
                "checkpoint root directory is not the distinct branch heads",
            ));
        }
        validate_leaves(&root.regions)?;
    }
    if roots != heads {
        return Err(corrupt("checkpoint omitted a selected branch root"));
    }
    if manifest
        .mirror_schema
        .is_some_and(|schema| !schemas.contains(&schema))
        || (!manifest.mirror.is_empty() && manifest.mirror_schema.is_none())
    {
        return Err(corrupt("checkpoint mirror has no schema carrier"));
    }
    validate_leaves(&manifest.mirror)?;
    let mut prior_bucket = None;
    for bucket in &manifest.symbol_buckets {
        if prior_bucket.is_some_and(|prior| prior >= bucket.bucket)
            || bucket.pages.is_empty()
            || bucket.pages.iter().collect::<BTreeSet<_>>().len() != bucket.pages.len()
        {
            return Err(corrupt("checkpoint symbol bucket directory is invalid"));
        }
        prior_bucket = Some(bucket.bucket);
    }
    validate_pages(&manifest.symbols)?;
    if manifest.symbol_identity_frontier == 0
        || manifest
            .symbols
            .last()
            .is_some_and(|page| page.last >= u64::from(manifest.symbol_identity_frontier))
    {
        return Err(corrupt(
            "checkpoint symbol identity frontier does not cover its pages",
        ));
    }
    validate_pages(&manifest.lineage)?;
    Ok(heads)
}

fn validate_leaves(
    leaves: &[crate::durability::data::RelationalCheckpointRegion],
) -> Result<(), DurabilityError> {
    let mut previous = None;
    for leaf in leaves {
        if previous.is_some_and(|prior| prior >= leaf.partition_id)
            || leaf
                .entity_slots
                .checked_add(leaf.relation_slots)
                .is_none_or(|slots| leaf.unavailable_records > slots)
        {
            return Err(corrupt(
                "checkpoint region directory has duplicate, unordered or invalid leaves",
            ));
        }
        previous = Some(leaf.partition_id);
    }
    Ok(())
}

fn validate_pages(
    pages: &[crate::durability::data::RelationalCheckpointPage],
) -> Result<(), DurabilityError> {
    let mut previous = None;
    for page in pages {
        if page.first > page.last || previous.is_some_and(|last| page.first <= last) {
            return Err(corrupt("checkpoint pages overlap or are unordered"));
        }
        previous = Some(page.last);
    }
    Ok(())
}
