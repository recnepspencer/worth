use std::collections::{BTreeMap, BTreeSet};
use worth_relational_contracts::internal_port as contracts;

use super::manifest_capture::corrupt;
use crate::durability::data::RelationalCheckpointManifest;
use contracts::durability::data::recovery_errors::DurabilityError;
use contracts::history::data::coordinates::CommitId;
use worth_foundational::FoundationalBranchTarget;

pub mod components;

#[cfg(test)]
#[path = "manifest_integrity_tests.rs"]
pub mod tests;

#[derive(Debug)]
pub(super) struct ManifestDirectoryFacts {
    pub(super) heads: BTreeSet<CommitId>,
    pub(super) history_members:
        std::sync::Arc<worth_relational_storage::internal_port::storage::data::history_member_directory::RelationalHistoryMemberDirectory>,
}

pub(super) fn validate(
    lane: &contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    manifest: &RelationalCheckpointManifest,
) -> Result<ManifestDirectoryFacts, DurabilityError> {
    if manifest.index_generation_frontier == 0 {
        return Err(corrupt("checkpoint index generation frontier is zero"));
    }
    // G7: the commit index is validated from its descriptor alone; pages
    // are checked against it when a lookup first faults them.
    manifest.commits.validate().map_err(corrupt)?;
    let tail = manifest.commits.tail.as_ref();
    if manifest.covered_position.0 != manifest.commits.count
        || manifest.coverage.up_to_commit.as_ref() != tail.map(|entry| &entry.commit)
        || manifest.coverage.up_to_version != tail.map(|entry| entry.commit.version_id)
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
        if manifest.commits.head(root.commit_id).is_none()
            || !schemas.contains(&root.schema_carrier_digest)
            || !roots.insert(root.commit_id)
        {
            return Err(corrupt(
                "checkpoint root directory has duplicate or unbacked roots",
            ));
        }
        validate_leaves(&root.regions)?;
    }
    if !heads.is_subset(&roots) || manifest.commits.heads.len() != roots.len() {
        return Err(corrupt("checkpoint omitted a selected branch root"));
    }
    components::validate(manifest)?;
    if manifest
        .mirror_schema
        .is_some_and(|schema| !schemas.contains(&schema))
        || (!manifest.mirror.is_empty() && manifest.mirror_schema.is_none())
    {
        return Err(corrupt("checkpoint mirror has no schema carrier"));
    }
    validate_leaves(&manifest.mirror)?;
    validate_shared_leaves(manifest)?;
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
            .iter()
            .any(|page| page.last >= u64::from(manifest.symbol_identity_frontier))
    {
        return Err(corrupt(
            "checkpoint symbol identity frontier does not cover its pages",
        ));
    }
    validate_pages(&manifest.lineage)?;
    super::lineage_event_directory::validate(manifest)?;
    validate_record_identity(&manifest.record_identity)?;
    let history_members = manifest
        .readmit_history_members(lane)
        .map_err(|cause| corrupt(format!("history member dependencies: {cause:?}")))?;
    Ok(ManifestDirectoryFacts {
        heads,
        history_members,
    })
}

fn validate_shared_leaves(manifest: &RelationalCheckpointManifest) -> Result<(), DurabilityError> {
    let mut members = BTreeMap::new();
    for (schema, leaves) in manifest
        .roots
        .iter()
        .map(|root| (root.schema_carrier_digest, &root.regions))
        .chain(
            manifest
                .mirror_schema
                .map(|schema| (schema, &manifest.mirror)),
        )
    {
        for leaf in leaves {
            let key = (leaf.partition_id, schema, leaf.head, leaf.history);
            if members.insert(key, leaf).is_some_and(|prior| prior != leaf) {
                return Err(corrupt(
                    "shared region members disagree on their leaf facts",
                ));
            }
        }
    }
    Ok(())
}

fn validate_leaves(
    leaves: &[worth_relational_storage::internal_port::storage::data::checkpoint_region::RelationalCheckpointRegion],
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
        if (leaf.entity_slots == 0 && leaf.chunks.entity_domain != 0)
            || (leaf.relation_slots == 0 && leaf.chunks.relation_domain != 0)
        {
            return Err(corrupt(
                "checkpoint region chunk placement claims slots in an empty arena",
            ));
        }
        leaf.validate_directory()
            .map_err(|detail| corrupt(format!("checkpoint region {detail}")))?;
        leaf.chunks.admit().map_err(|refusal| {
            corrupt(format!(
                "checkpoint region chunk placement is invalid: {refusal:?}"
            ))
        })?;
        previous = Some(leaf.partition_id);
    }
    Ok(())
}

pub(super) fn validate_pages(
    pages: &[crate::durability::data::RelationalCheckpointPage],
) -> Result<(), DurabilityError> {
    let mut previous = None;
    let mut range = None;
    let mut occupied = [0_u64; 4];
    let mut members = BTreeSet::new();
    for page in pages {
        let key = (page.range(), page.first);
        if !page.valid()
            || previous.is_some_and(|prior| key <= prior)
            || !members.insert(page.member)
        {
            return Err(corrupt(
                "checkpoint page membership is invalid or unordered",
            ));
        }
        if range != Some(page.range()) {
            occupied = [0; 4];
            range = Some(page.range());
        }
        for (prior, incoming) in occupied.iter_mut().zip(page.membership) {
            if *prior & incoming != 0 {
                return Err(corrupt("checkpoint pages duplicate a published identity"));
            }
            *prior |= incoming;
        }
        previous = Some(key);
    }
    Ok(())
}

pub(super) fn validate_record_identity(
    identity: &worth_relational_storage::internal_port::storage::data::record_identity_pages::PagedRecordIdentity,
) -> Result<(), DurabilityError> {
    let limit = worth_relational_storage::internal_port::storage::data::record_identity_pages::IDENTITY_PAGE_BYTES as u64;
    let mut previous = None;
    for page in &identity.pages {
        if previous.is_some_and(|prior| prior >= page) || page.length > limit {
            return Err(corrupt(
                "checkpoint record identity page roster is duplicate, unsorted or oversized",
            ));
        }
        previous = Some(page);
    }
    let roots = &identity.roots;
    let tables = [
        roots.generation_high_water,
        roots.reusable_slots,
        roots.append_frontiers,
        roots.pending_reservations,
    ];
    let mut total = 0_u64;
    let mut any_root = false;
    for root in tables.into_iter().flatten() {
        any_root = true;
        if root.entry_count == 0 || identity.pages.binary_search(&root.root_page).is_err() {
            return Err(corrupt(
                "checkpoint record identity root is empty or absent from page roster",
            ));
        }
        total = total
            .checked_add(root.entry_count)
            .ok_or_else(|| corrupt("checkpoint record identity entry count overflow"))?;
    }
    if identity.pages.is_empty() == any_root
        || roots.append_frontiers.is_some() != roots.partition_high_water.is_some()
    {
        return Err(corrupt(
            "checkpoint record identity roots disagree with page roster or partition high water",
        ));
    }
    let _ = total;
    Ok(())
}
