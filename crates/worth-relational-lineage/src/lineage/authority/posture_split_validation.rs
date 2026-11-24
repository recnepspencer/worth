//! Strict fail-closed validation of `Split` lineage postures against staged
//! `WorkingState` effects.

use std::collections::{BTreeMap, BTreeSet};

use crate::contracts::identity::data::primitives::VersionId;
use crate::contracts::lineage::data::naming::{
    InstalledPersistentNameSchemeRegistry, PersistentNameBirthRequest, PersistentNameIdentity,
};
use crate::contracts::lineage::data::naming_denial::{LineageNameDenial, LineagePostureKind};
use crate::contracts::lineage::data::postures::{
    LineageBasisView, LineageRecordTarget, LineageSplitChild,
};
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

use super::posture_validation::{
    mismatch, require_live_holder_and_lineage, validate_birth_holder_and_subject, ValidatedPosture,
};
use super::staged_effects::{existed_at_basis, is_retired_in_commit};

#[allow(clippy::too_many_arguments)]
pub(super) fn validate_split_claim(
    staged: &WorkingState,
    version_id: VersionId,
    scheme_registry: &InstalledPersistentNameSchemeRegistry,
    basis: &dyn LineageBasisView,
    created_targets: &BTreeSet<LineageRecordTarget>,
    parent_target: LineageRecordTarget,
    parent_name: PersistentNameIdentity,
    children: &[LineageSplitChild],
    claimed_created_targets: &mut BTreeSet<LineageRecordTarget>,
    claimed_basis_targets: &mut BTreeSet<LineageRecordTarget>,
    retired_names_in_commit: &mut BTreeSet<PersistentNameIdentity>,
    births_to_mint: &mut BTreeMap<LineageRecordTarget, PersistentNameBirthRequest>,
) -> Result<ValidatedPosture, LineageNameDenial> {
    if children.is_empty() {
        return Err(mismatch(
            LineagePostureKind::Split,
            parent_target,
            "Split posture requires at least one created successor",
        ));
    }
    if !existed_at_basis(staged, parent_target, version_id)
        || !is_retired_in_commit(staged, parent_target, version_id)
    {
        return Err(mismatch(
            LineagePostureKind::Split,
            parent_target,
            "Split posture requires its parent to exist at basis and be retired in this commit",
        ));
    }
    if !claimed_basis_targets.insert(parent_target) {
        return Err(mismatch(
            LineagePostureKind::Split,
            parent_target,
            "Split parent is already claimed by another posture in this commit",
        ));
    }
    let parent_lineage = require_live_holder_and_lineage(
        LineagePostureKind::Split,
        staged,
        basis,
        parent_target,
        parent_name,
    )?;
    retired_names_in_commit.insert(parent_name);

    let mut child_targets = Vec::with_capacity(children.len());
    for child in children {
        if child.target.subject() != parent_target.subject() {
            return Err(mismatch(
                LineagePostureKind::Split,
                child.target,
                "Split child subject kind must match its parent subject kind",
            ));
        }
        if !created_targets.contains(&child.target) {
            return Err(mismatch(
                LineagePostureKind::Split,
                child.target,
                "Split child must be newly created and live in this commit",
            ));
        }
        if !claimed_created_targets.insert(child.target) {
            return Err(mismatch(
                LineagePostureKind::Split,
                child.target,
                "Split child is already claimed by another posture in this commit",
            ));
        }
        validate_birth_holder_and_subject(
            LineagePostureKind::Split,
            child.target,
            &child.birth,
            scheme_registry,
        )?;
        if let Some(scheme) = scheme_registry.get(&child.birth.scheme_id)
            && !scheme.split_capable()
        {
            return Err(mismatch(
                LineagePostureKind::Split,
                child.target,
                "Split child scheme is not declared split-capable",
            ));
        }
        let mut normalized_birth = child.birth.clone();
        if normalized_birth.parent_identities.is_empty() {
            normalized_birth.parent_identities = vec![parent_name];
        } else if normalized_birth.parent_identities != vec![parent_name] {
            return Err(mismatch(
                LineagePostureKind::Split,
                child.target,
                "Split child birth parent_identities must match the split parent",
            ));
        }
        if normalized_birth.canonical_discriminator.is_empty() {
            return Err(LineageNameDenial::MissingSplitDiscriminator {
                scheme_id: normalized_birth.scheme_id,
                role_path: normalized_birth.role_path,
            });
        }
        births_to_mint.insert(child.target, normalized_birth);
        child_targets.push(child.target);
    }
    Ok(ValidatedPosture::Split {
        subject: parent_target.subject(),
        parent_lineage,
        children: child_targets,
    })
}
