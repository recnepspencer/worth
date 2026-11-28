//! Read-free `delta_base` fork-provenance, headerless-root, and alias-authority
//! validation across every `BranchNamingRoot` in a `CHK24` checkpoint manifest.

use worth_relational_contracts::internal_port::durability::data::recovery_errors::DurabilityError;
use worth_relational_lineage::internal_port::lineage::naming_pages::{tables, BranchNamingRoot};

use super::super::super::manifest_capture::corrupt;
use super::ancestry::AncestryIndex;
use super::chain_has_domain;
use crate::durability::data::RelationalCheckpointManifest;

fn has_own_domain(root: &BranchNamingRoot, domain: u8) -> bool {
    root.pages
        .iter()
        .any(|p| tables::table_domain(p.first[0]) == Some(domain))
}

fn validate_alias_header_authority(
    root: &BranchNamingRoot,
    base: &BranchNamingRoot,
    has_naming_schemes: bool,
) -> Result<(), DurabilityError> {
    if root.has_active_domain(1) && !has_own_domain(root, 1) {
        if !base.has_active_domain(1) || base.naming != root.naming {
            return Err(corrupt(
                "branch naming delta alias header disagrees with delta_base",
            ));
        }
    }
    if root.has_active_domain(2) && base.has_active_domain(2) {
        let covers_retention = base
            .index
            .zip(root.index)
            .is_some_and(|(b, r)| b.retention_floor <= r.retention_floor);
        if !covers_retention {
            return Err(corrupt(
                "branch index delta_base compacted past child retention_floor",
            ));
        }
    }
    if root.has_active_domain(2) && !has_own_domain(root, 2) {
        let same_earliest = base
            .index
            .zip(root.index)
            .is_some_and(|(b, r)| b.earliest_commit == r.earliest_commit);
        // A role-only kept-group transition can advance `index.head` without
        // mutating any index row; when lineage events also diverge under active
        // naming schemes, a zero-page index alias cannot disagree on `head`.
        let head_mismatch_with_event_diff = has_naming_schemes
            && base.index.zip(root.index).is_some_and(|(b, r)| {
                if b.head == r.head {
                    return false;
                }
                if base.naming.is_some() || root.naming.is_some() {
                    return base.naming != root.naming;
                }
                let (later, head) = if b.head > r.head { (base, b.head) } else { (root, r.head) };
                later.groups.as_ref().and_then(|groups| groups.latest_commit) != head
            });
        if !base.has_active_domain(2) || !same_earliest || head_mismatch_with_event_diff {
            return Err(corrupt(
                "branch index delta alias header disagrees with delta_base",
            ));
        }
    }
    if root.has_active_domain(tables::GROUP_STATE_DOMAIN)
        && !has_own_domain(root, tables::GROUP_STATE_DOMAIN)
    {
        let same_groups = base
            .groups
            .as_ref()
            .zip(root.groups.as_ref())
            .is_some_and(|(b, r)| b.schemes == r.schemes && b.earliest_commit == r.earliest_commit);
        if !base.has_active_domain(tables::GROUP_STATE_DOMAIN) || !same_groups {
            return Err(corrupt(
                "branch group delta alias header disagrees with delta_base",
            ));
        }
    }
    Ok(())
}

/// Validates every root in `manifest.naming.branches` (including non-cell roots)
/// for headerless pages/deltas, header-vs-page presence, fork provenance of
/// `delta_base`, and alias-authority header consistency.
pub(super) fn validate_all_roots_and_delta_provenance(
    manifest: &RelationalCheckpointManifest,
    ancestry: &AncestryIndex<'_>,
    has_naming_schemes: bool,
) -> Result<(), DurabilityError> {
    for root in &manifest.naming.branches {
        if root.naming.is_none()
            && root.index.is_none()
            && root.groups.is_none()
            && (!root.pages.is_empty() || root.delta_base.is_some())
        {
            return Err(corrupt(
                "branch naming root carries pages or delta_base without any naming, index, or group header",
            ));
        }
        if root.naming.is_none() && has_own_domain(root, 1) {
            return Err(corrupt("branch omitted naming header despite naming pages"));
        }
        if root.index.is_none() && has_own_domain(root, 2) {
            return Err(corrupt("branch omitted index header despite index pages"));
        }
        if root.groups.is_none()
            && (has_own_domain(root, tables::GROUP_STATE_DOMAIN)
                || has_own_domain(root, tables::GROUP_MEMBER_DOMAIN))
        {
            return Err(corrupt("branch omitted group header despite group pages"));
        }
        if has_own_domain(root, tables::GROUP_MEMBER_DOMAIN)
            && !chain_has_domain(manifest, root, tables::GROUP_STATE_DOMAIN)
        {
            return Err(corrupt(
                "branch group member pages lack any group state pages",
            ));
        }
        if let Some(base_branch) = root.delta_base.as_ref() {
            if !ancestry.is_committed(&root.branch) || !ancestry.is_committed(base_branch) {
                return Err(corrupt(
                    "branch naming root delta_base requires committed branch cells",
                ));
            }
            if !ancestry.shares_delta_provenance(&root.branch, base_branch) {
                return Err(corrupt(
                    "branch naming root delta_base lacks shared fork provenance",
                ));
            }
            let base_root = manifest
                .naming
                .root(base_branch)
                .ok_or_else(|| corrupt("branch naming root delta_base references missing root"))?;
            validate_alias_header_authority(root, base_root, has_naming_schemes)?;
        }
    }
    Ok(())
}
