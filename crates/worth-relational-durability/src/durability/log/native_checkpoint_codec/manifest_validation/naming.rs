//! Read-free per-branch `CHK24` naming, index, and group state validation.
//!
//! Cross-checks every committed branch's [`BranchNamingRoot`] against
//! independently established manifest facts (`branch_cells`, `roots`,
//! `lineage_events`, `commits`, `schemas`, and `group_log`) so replacing an
//! individual branch's naming root with an empty root fails closed with
//! `CorruptCheckpoint` even when other named branches remain intact.

mod chunks;

use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::FoundationalBranchTarget;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_lineage::internal_port::lineage::naming_pages::codec::NamingCodec;
use worth_relational_lineage::internal_port::lineage::naming_pages::tables;

use super::super::manifest_capture::corrupt;
use crate::durability::data::RelationalCheckpointManifest;
use contracts::durability::data::recovery_errors::DurabilityError;
use contracts::history::data::coordinates::{BranchId, CommitId};

pub(super) fn validate(
    manifest: &RelationalCheckpointManifest,
    has_naming_schemes: bool,
) -> Result<(), DurabilityError> {
    manifest.naming.validate().map_err(corrupt)?;
    let tail_commit = manifest
        .commits
        .tail
        .as_ref()
        .map(|entry| entry.commit.commit_id.0);
    if let Some(through) = manifest.naming.group_log_through
        && tail_commit.is_none_or(|tail| through > tail)
    {
        return Err(corrupt(
            "checkpoint naming group log exceeds covered commit tail",
        ));
    }

    let branches_with_own_events: BTreeSet<&BranchId> = manifest
        .lineage_events
        .iter()
        .flat_map(|page| page.routes.iter().map(|route| &route.branch_id))
        .collect();
    let mut branch_forks: BTreeMap<&BranchId, (&BranchId, u64)> = BTreeMap::new();
    for cell in &manifest.branch_cells {
        if let (Some(source), Some(prov)) = (&cell.fork_source_branch_id, &cell.fork_provenance)
            && let FoundationalBranchTarget::Basis(target) = prov.target()
        {
            branch_forks.insert(&cell.branch_id, (source, target.selected_commit_id()));
        }
    }

    let (group_log_branch_min, group_log_branch_max) = validate_group_log_bounds(manifest)?;
    chunks::validate_continuation_chunks(&manifest.naming.group_log, true)?;
    for root in &manifest.naming.branches {
        chunks::validate_continuation_chunks(&root.pages, false)?;
    }

    let mut any_naming_branch = false;
    let mut any_index_branch = false;
    for cell in &manifest.branch_cells {
        let FoundationalBranchTarget::Basis(target) = cell.observation.target() else {
            continue;
        };
        let head = target.selected_commit_id();
        let root = manifest.naming.root(&cell.branch_id).ok_or_else(|| {
            corrupt(format!(
                "checkpoint naming state omitted committed branch {}",
                cell.branch_id.0
            ))
        })?;
        validate_branch_root_headers(
            manifest,
            root,
            head,
            &mut any_naming_branch,
            &mut any_index_branch,
        )?;

        // Per-branch authoritative coverage: neither own lineage events nor
        // inherited fork lineage/groups may be silently replaced by an empty root,
        // even after the parent advances past the fork cutoff.
        let has_own_events = branches_with_own_events.contains(&cell.branch_id);
        let (mut must_have_naming, mut must_have_index, mut must_have_groups) =
            (has_own_events, has_own_events && has_naming_schemes, false);
        let mut cursor = &cell.branch_id;
        let mut eff_cutoff = head;
        let mut visited = BTreeSet::from([cursor]);
        while let Some(&(parent, fork_commit)) = branch_forks.get(cursor) {
            eff_cutoff = eff_cutoff.min(fork_commit);
            if !visited.insert(parent) {
                break;
            }
            if let Some(parent_root) = manifest.naming.root(parent) {
                if parent_root.naming.is_some_and(|h| {
                    h.unavailable || h.earliest_event_commit.is_some_and(|at| at <= eff_cutoff)
                }) {
                    must_have_naming = true;
                }
                if parent_root.index.is_some_and(|h| {
                    h.unavailable || h.earliest_commit.is_some_and(|at| at <= eff_cutoff)
                }) {
                    must_have_index = true;
                }
                if parent_root.groups.as_ref().is_some_and(|h| {
                    h.unavailable || h.earliest_commit.is_some_and(|at| at <= eff_cutoff)
                }) {
                    must_have_groups = true;
                }
            }
            if group_log_branch_min
                .get(&tables::branch_key_digest(parent))
                .is_some_and(|&at| at <= eff_cutoff)
            {
                must_have_groups = true;
            }
            cursor = parent;
        }
        if must_have_naming
            && !root
                .naming
                .is_some_and(|h| h.unavailable || h.latest_event_commit.is_some())
        {
            return Err(corrupt(format!(
                "committed branch {} has lineage events but empty naming basis root",
                cell.branch_id.0
            )));
        }
        if must_have_index
            && !root
                .index
                .is_some_and(|h| h.unavailable || h.head.is_some())
        {
            return Err(corrupt(format!(
                "committed branch {} has persistent naming authority but empty name index root",
                cell.branch_id.0
            )));
        }
        let branch_digest = tables::branch_key_digest(&cell.branch_id);
        let min_group_commit = group_log_branch_min
            .get(&branch_digest)
            .copied()
            .filter(|&c| c <= head);
        let max_group_commit = group_log_branch_max
            .get(&branch_digest)
            .copied()
            .filter(|&c| c <= head);
        if must_have_groups || min_group_commit.is_some() {
            let ok = root.groups.as_ref().is_some_and(|g| {
                g.unavailable
                    || g.latest_commit
                        .is_some_and(|at| max_group_commit.is_none_or(|max| at >= max))
                        && g.earliest_commit
                            .is_some_and(|at| min_group_commit.is_none_or(|min| at <= min))
            });
            if !ok {
                return Err(corrupt(format!(
                    "committed branch {} omitted required group state root",
                    cell.branch_id.0
                )));
            }
        }
    }
    if !manifest.lineage_events.is_empty() {
        if !any_naming_branch {
            return Err(corrupt(
                "checkpoint lineage events lack corresponding branch naming state",
            ));
        }
        if has_naming_schemes && !any_index_branch {
            return Err(corrupt(
                "checkpoint with persistent name schemes omitted branch name index",
            ));
        }
    }
    Ok(())
}

fn validate_group_log_bounds(
    manifest: &RelationalCheckpointManifest,
) -> Result<(BTreeMap<[u8; 32], u64>, BTreeMap<[u8; 32], u64>), DurabilityError> {
    let mut group_log_branch_min: BTreeMap<[u8; 32], u64> = BTreeMap::new();
    let mut group_log_branch_max: BTreeMap<[u8; 32], u64> = BTreeMap::new();
    let mut expected_ordinal: u64 = 1;
    let mut prev_tail: Option<(u64, [u8; 32])> = None;
    let mut max_group_log_commit: Option<u64> = None;
    for page in &manifest.naming.group_log {
        if page.first.first() != Some(&tables::GROUP_LOG) {
            continue;
        }
        let ((ord_first, commit_first), branch_first) =
            <((u64, CommitId), [u8; 32])>::from_bytes(&page.first[1..])
                .map_err(|_| corrupt("malformed checkpoint group log first key"))?;
        let ((ord_last, commit_last), branch_last) =
            <((u64, CommitId), [u8; 32])>::from_bytes(&page.last[1..])
                .map_err(|_| corrupt("malformed checkpoint group log last key"))?;
        let span = ord_last
            .checked_sub(ord_first)
            .and_then(|d| d.checked_add(1));
        if ord_first != expected_ordinal
            || span != Some(u64::from(page.entries))
            || commit_first > commit_last
            || (ord_first == ord_last
                && (commit_first != commit_last || branch_first != branch_last))
            || prev_tail
                .as_ref()
                .is_some_and(|(pc, pb)| (commit_first.0, &branch_first) <= (*pc, pb))
        {
            return Err(corrupt(
                "checkpoint primary GROUP_LOG page roster has missing, gapped, or unordered pages",
            ));
        }
        if manifest
            .naming
            .group_log_through
            .is_none_or(|through| commit_last.0 > through)
        {
            return Err(corrupt(
                "checkpoint group log entry exceeds group_log_through",
            ));
        }
        expected_ordinal = ord_last
            .checked_add(1)
            .ok_or_else(|| corrupt("checkpoint group log ordinal overflow"))?;
        prev_tail = Some((commit_last.0, branch_last));
        max_group_log_commit = Some(commit_last.0);
        for (commit, branch) in [(commit_first, branch_first), (commit_last, branch_last)] {
            group_log_branch_min
                .entry(branch)
                .and_modify(|at| *at = (*at).min(commit.0))
                .or_insert(commit.0);
            group_log_branch_max
                .entry(branch)
                .and_modify(|at| *at = (*at).max(commit.0))
                .or_insert(commit.0);
        }
    }
    if manifest.naming.group_log_through != max_group_log_commit {
        return Err(corrupt(
            "checkpoint group_log_through disagrees with primary GROUP_LOG page descriptors",
        ));
    }
    if max_group_log_commit.is_none() && !manifest.naming.group_log.is_empty() {
        return Err(corrupt(
            "checkpoint group log carries continuation chunks without any GROUP_LOG page",
        ));
    }
    Ok((group_log_branch_min, group_log_branch_max))
}

fn validate_branch_root_headers(
    manifest: &RelationalCheckpointManifest,
    root: &worth_relational_lineage::internal_port::lineage::naming_pages::BranchNamingRoot,
    head: u64,
    any_naming_branch: &mut bool,
    any_index_branch: &mut bool,
) -> Result<(), DurabilityError> {
    if root.naming.is_none()
        && root.index.is_none()
        && root.groups.is_none()
        && !root.pages.is_empty()
    {
        return Err(corrupt(
            "branch naming root carries pages without any naming, index, or group header",
        ));
    }
    let has_domain = |domain: u8| {
        root.pages
            .iter()
            .any(|p| tables::table_domain(p.first[0]) == Some(domain))
    };
    let has_naming_pages = has_domain(1);
    let has_index_pages = has_domain(2);
    let has_group_pages = has_domain(3);
    if let Some(header) = root.naming {
        if !header.unavailable {
            if header.earliest_event_commit.is_some() != header.latest_event_commit.is_some()
                || header
                    .earliest_event_commit
                    .zip(header.latest_event_commit)
                    .is_some_and(|(e, l)| e > l)
                || header.latest_event_commit.is_some_and(|at| at > head)
            {
                return Err(corrupt("branch naming basis exceeds branch head commit"));
            }
            if header.latest_event_commit.is_some() && !has_naming_pages {
                return Err(corrupt(
                    "branch naming basis claims events without any naming pages",
                ));
            }
            *any_naming_branch |= header.latest_event_commit.is_some();
        } else {
            *any_naming_branch = true;
        }
    } else if has_naming_pages {
        return Err(corrupt("branch omitted naming header despite naming pages"));
    }
    if let Some(header) = root.index {
        if !header.unavailable {
            if header.earliest_commit.is_some() != header.head.is_some()
                || header
                    .earliest_commit
                    .zip(header.head)
                    .is_some_and(|(e, l)| e > l)
                || header.head.is_some_and(|at| at > head)
                || header.retention_floor.is_some_and(|at| at > head)
            {
                return Err(corrupt("branch name index exceeds branch head commit"));
            }
            if header.head.is_some() && !has_index_pages {
                return Err(corrupt(
                    "branch name index claims commits without any index pages",
                ));
            }
            *any_index_branch |= header.head.is_some();
        } else {
            *any_index_branch = true;
        }
    } else if has_index_pages {
        return Err(corrupt("branch omitted index header despite index pages"));
    }
    if let Some(header) = &root.groups {
        if !header.unavailable {
            if header.earliest_commit.is_some() != header.latest_commit.is_some()
                || header
                    .earliest_commit
                    .zip(header.latest_commit)
                    .is_some_and(|(e, l)| e > l)
                || header.latest_commit.is_some_and(|at| at > head)
            {
                return Err(corrupt("branch group state exceeds branch head commit"));
            }
            if header.latest_commit.is_some() && !has_group_pages {
                return Err(corrupt(
                    "branch group state claims commits without any group pages",
                ));
            }
            if !header.schemes.is_empty()
                && (header.latest_commit.is_none()
                    || !has_group_pages
                    || manifest.naming.group_log.is_empty()
                    || manifest.naming.group_log_through < header.latest_commit)
            {
                return Err(corrupt(
                    "branch group schemes disagree with checkpoint group log",
                ));
            }
        }
    } else if has_group_pages {
        return Err(corrupt("branch omitted group header despite group pages"));
    }
    Ok(())
}
