//! Strict fail-closed validation of `Merge` lineage postures (`KeepParent` and
//! `MintNew`) against staged `WorkingState` effects.

use std::collections::{BTreeMap, BTreeSet};

use crate::identity::data::VersionId;
use crate::lineage::data::{
    InstalledPersistentNameSchemeRegistry, LineageBasisView, LineageMergeKeeper, LineageNameDenial,
    LineagePostureKind, LineageRecordTarget, PersistentNameBirthRequest, PersistentNameIdentity,
};
use crate::runtime::WorkingState;

use super::posture_validation::{
    mismatch, require_live_holder_and_lineage, validate_birth_holder_and_subject,
    ValidatedMergeTarget, ValidatedPosture,
};
use super::staged_effects::{existed_at_basis, is_live_after_commit, is_retired_in_commit};

#[allow(clippy::too_many_arguments)]
pub(super) fn validate_merge_claim(
    staged: &WorkingState,
    version_id: VersionId,
    scheme_registry: &InstalledPersistentNameSchemeRegistry,
    basis: &dyn LineageBasisView,
    created_targets: &BTreeSet<LineageRecordTarget>,
    parents: &[(LineageRecordTarget, PersistentNameIdentity)],
    target: LineageRecordTarget,
    keeper: &LineageMergeKeeper,
    claimed_created_targets: &mut BTreeSet<LineageRecordTarget>,
    claimed_basis_targets: &mut BTreeSet<LineageRecordTarget>,
    retired_names_in_commit: &mut BTreeSet<PersistentNameIdentity>,
    kept_names_in_commit: &mut BTreeSet<PersistentNameIdentity>,
    births_to_mint: &mut BTreeMap<LineageRecordTarget, PersistentNameBirthRequest>,
) -> Result<ValidatedPosture, LineageNameDenial> {
    if parents.is_empty() {
        return Err(mismatch(
            LineagePostureKind::Merge,
            target,
            "Merge posture requires at least two parent records",
        ));
    }
    if let LineageMergeKeeper::KeepParent(keeper_id) = keeper
        && !parents.iter().any(|(_, id)| id == keeper_id)
    {
        return Err(LineageNameDenial::InvalidMergeKeeper {
            keeper: *keeper_id,
            parents: parents.iter().map(|(_, id)| *id).collect(),
        });
    }
    if parents.len() < 2 {
        return Err(mismatch(
            LineagePostureKind::Merge,
            target,
            "Merge posture requires at least two distinct parents",
        ));
    }

    let mut seen_parent_targets = BTreeSet::new();
    let mut seen_parent_names = BTreeSet::new();
    let mut source_lineages = Vec::with_capacity(parents.len());
    let mut parent_lineage_by_name = BTreeMap::new();
    let mut parent_target_by_name = BTreeMap::new();

    for &(parent_target, parent_name) in parents {
        if parent_target.subject() != target.subject() {
            return Err(mismatch(
                LineagePostureKind::Merge,
                parent_target,
                "Merge parent subject kind must match target subject kind",
            ));
        }
        if !seen_parent_targets.insert(parent_target) || !seen_parent_names.insert(parent_name) {
            return Err(mismatch(
                LineagePostureKind::Merge,
                parent_target,
                "Merge parents must list distinct targets and distinct persistent names",
            ));
        }
        if !existed_at_basis(staged, parent_target, version_id) {
            return Err(mismatch(
                LineagePostureKind::Merge,
                parent_target,
                "Merge parent must exist at commit basis",
            ));
        }
        if !claimed_basis_targets.insert(parent_target) {
            return Err(mismatch(
                LineagePostureKind::Merge,
                parent_target,
                "Merge parent is already claimed by another posture in this commit",
            ));
        }
        let lineage_id = require_live_holder_and_lineage(
            LineagePostureKind::Merge,
            staged,
            basis,
            parent_target,
            parent_name,
        )?;
        source_lineages.push(lineage_id);
        parent_lineage_by_name.insert(parent_name, lineage_id);
        parent_target_by_name.insert(parent_name, parent_target);
    }
    source_lineages.sort_unstable();

    let validated_target = match keeper {
        LineageMergeKeeper::MintNew { birth } => {
            for &(parent_target, parent_name) in parents {
                if !is_retired_in_commit(staged, parent_target, version_id) {
                    return Err(mismatch(
                        LineagePostureKind::Merge,
                        parent_target,
                        "Merge(MintNew) requires every listed parent to be retired in this commit",
                    ));
                }
                retired_names_in_commit.insert(parent_name);
            }
            if !created_targets.contains(&target) {
                return Err(mismatch(
                    LineagePostureKind::Merge,
                    target,
                    "Merge(MintNew) target must be newly created and live in this commit",
                ));
            }
            if !claimed_created_targets.insert(target) {
                return Err(mismatch(
                    LineagePostureKind::Merge,
                    target,
                    "Merge(MintNew) target is already claimed by another posture in this commit",
                ));
            }
            validate_birth_holder_and_subject(
                LineagePostureKind::Merge,
                target,
                birth,
                scheme_registry,
            )?;
            let mut sorted_parent_names: Vec<PersistentNameIdentity> =
                seen_parent_names.into_iter().collect();
            sorted_parent_names.sort_unstable();
            let mut normalized_birth = birth.clone();
            if normalized_birth.parent_identities.is_empty() {
                normalized_birth.parent_identities = sorted_parent_names;
            } else {
                let mut supplied = normalized_birth.parent_identities.clone();
                supplied.sort_unstable();
                if supplied != sorted_parent_names {
                    return Err(mismatch(
                        LineagePostureKind::Merge,
                        target,
                        "Merge(MintNew) birth parent_identities must match all declared merge parents",
                    ));
                }
                normalized_birth.parent_identities = sorted_parent_names;
            }
            births_to_mint.insert(target, normalized_birth);
            ValidatedMergeTarget::MintedNew { target }
        }
        LineageMergeKeeper::KeepParent(keeper_id) => {
            let keeper_target = parent_target_by_name[keeper_id];
            let keeper_lineage = parent_lineage_by_name[keeper_id];
            for &(parent_target, parent_name) in parents {
                if parent_name == *keeper_id {
                    continue;
                }
                if !is_retired_in_commit(staged, parent_target, version_id) {
                    return Err(mismatch(
                        LineagePostureKind::Merge,
                        parent_target,
                        "Merge(KeepParent) requires every non-keeper parent to be retired in this commit",
                    ));
                }
                retired_names_in_commit.insert(parent_name);
            }
            if !kept_names_in_commit.insert(*keeper_id) {
                return Err(mismatch(
                    LineagePostureKind::Merge,
                    target,
                    "Merge keeper name is already kept by another merge in this commit",
                ));
            }
            if target == keeper_target
                && is_live_after_commit(staged, keeper_target)
                && !created_targets.contains(&target)
            {
                ValidatedMergeTarget::InPlaceKeeper {
                    lineage_id: keeper_lineage,
                    kept_name: *keeper_id,
                }
            } else if is_retired_in_commit(staged, keeper_target, version_id)
                && created_targets.contains(&target)
                && claimed_created_targets.insert(target)
            {
                ValidatedMergeTarget::RecreatedKeeper {
                    target,
                    kept_name: *keeper_id,
                }
            } else {
                return Err(mismatch(
                    LineagePostureKind::Merge,
                    target,
                    "Merge(KeepParent) requires either preserving the keeper in place or retiring it and creating a replacement target",
                ));
            }
        }
    };

    Ok(ValidatedPosture::Merge {
        subject: target.subject(),
        source_lineages,
        target: validated_target,
    })
}
