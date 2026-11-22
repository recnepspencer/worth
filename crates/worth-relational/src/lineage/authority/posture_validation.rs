//! Strict fail-closed validation of Phase 2 entity and relation lineage
//! postures against actual performed commit effects in `WorkingState`.

use std::collections::{BTreeMap, BTreeSet};

use crate::identity::data::{LineageId, VersionId};
use crate::lineage::data::{
    LineageCommitPostureInput, LineageNameDenial, LineagePostureClaim, LineagePostureKind,
    LineageRecordTarget, LineageSubject, PersistentNameBirthRecord, PersistentNameBirthRequest,
    PersistentNameIdentity,
};
use crate::runtime::WorkingState;
use crate::transactions::data::RecordRef;

use super::posture_merge_validation::validate_merge_claim;
use super::staged_effects::{
    basis_lineage_id, existed_at_basis, is_created_in_commit, is_live_after_commit,
    is_retired_in_commit,
};

pub(super) enum ValidatedMergeTarget {
    InPlaceKeeper {
        lineage_id: LineageId,
        kept_name: PersistentNameIdentity,
    },
    RecreatedKeeper {
        target: LineageRecordTarget,
        kept_name: PersistentNameIdentity,
    },
    MintedNew {
        target: LineageRecordTarget,
    },
}

pub(super) enum ValidatedPosture {
    Create {
        target: LineageRecordTarget,
    },
    Preserve,
    Retire {
        subject: LineageSubject,
        source_lineage: LineageId,
    },
    Split {
        subject: LineageSubject,
        parent_lineage: LineageId,
        children: Vec<LineageRecordTarget>,
    },
    Merge {
        subject: LineageSubject,
        source_lineages: Vec<LineageId>,
        target: ValidatedMergeTarget,
    },
}

pub(super) struct ValidatedPostureBatch {
    pub(super) postures: Vec<ValidatedPosture>,
    pub(super) minted_births: BTreeMap<LineageRecordTarget, PersistentNameBirthRecord>,
    pub(super) claimed_created_targets: BTreeSet<LineageRecordTarget>,
    pub(super) claimed_basis_targets: BTreeSet<LineageRecordTarget>,
}

/// Validates every declared entity and relation posture against the actual
/// staged creations and retirements in `staged`, then mints all required
/// persistent names via [`super::name_minting::mint_target_births`].
///
/// Pure and fail-closed: performs zero allocator reservations and zero
/// mutations to `staged`.
pub(super) fn validate_commit_postures(
    staged: &WorkingState,
    version_id: VersionId,
    changed_records: &[RecordRef],
    input: &LineageCommitPostureInput,
) -> Result<ValidatedPostureBatch, LineageNameDenial> {
    if input.postures.is_empty() {
        return Ok(ValidatedPostureBatch {
            postures: Vec::new(),
            minted_births: BTreeMap::new(),
            claimed_created_targets: BTreeSet::new(),
            claimed_basis_targets: BTreeSet::new(),
        });
    }
    let changed_set: BTreeSet<LineageRecordTarget> = changed_records
        .iter()
        .map(|record| match record {
            RecordRef::Entity(id) => LineageRecordTarget::Entity(*id),
            RecordRef::Relation(id) => LineageRecordTarget::Relation(*id),
        })
        .collect();
    let created_targets: BTreeSet<LineageRecordTarget> = changed_set
        .iter()
        .copied()
        .filter(|target| {
            is_created_in_commit(staged, *target, version_id)
                && is_live_after_commit(staged, *target)
        })
        .collect();

    let mut claimed_created_targets = BTreeSet::new();
    let mut claimed_basis_targets = BTreeSet::new();
    let mut retired_names_in_commit = BTreeSet::new();
    let mut kept_names_in_commit = BTreeSet::new();
    let mut births_to_mint = BTreeMap::new();
    let mut validated = Vec::with_capacity(input.postures.len());

    for claim in &input.postures {
        match claim {
            LineagePostureClaim::Create { target, birth } => {
                if !created_targets.contains(target) {
                    return Err(mismatch(
                        LineagePostureKind::Create,
                        *target,
                        "Create posture targets a record not created and live in this commit",
                    ));
                }
                if !claimed_created_targets.insert(*target) {
                    return Err(mismatch(
                        LineagePostureKind::Create,
                        *target,
                        "record is already claimed by another posture in this commit",
                    ));
                }
                validate_birth_holder_and_subject(
                    LineagePostureKind::Create,
                    *target,
                    birth,
                    input,
                )?;
                births_to_mint.insert(*target, birth.clone());
                validated.push(ValidatedPosture::Create { target: *target });
            }
            LineagePostureClaim::Preserve { target, name } => {
                if !existed_at_basis(staged, *target, version_id)
                    || created_targets.contains(target)
                    || !is_live_after_commit(staged, *target)
                {
                    return Err(mismatch(
                        LineagePostureKind::Preserve,
                        *target,
                        "Preserve posture requires an existing live basis record not freshly created or retired",
                    ));
                }
                if !claimed_basis_targets.insert(*target) {
                    return Err(mismatch(
                        LineagePostureKind::Preserve,
                        *target,
                        "basis record is already claimed by another posture in this commit",
                    ));
                }
                require_live_holder_and_lineage(
                    LineagePostureKind::Preserve,
                    staged,
                    input,
                    *target,
                    *name,
                )?;
                validated.push(ValidatedPosture::Preserve);
            }
            LineagePostureClaim::Retire { target, name } => {
                if !existed_at_basis(staged, *target, version_id)
                    || !is_retired_in_commit(staged, *target, version_id)
                {
                    return Err(mismatch(
                        LineagePostureKind::Retire,
                        *target,
                        "Retire posture requires an existing basis record retired in this commit",
                    ));
                }
                if !claimed_basis_targets.insert(*target) {
                    return Err(mismatch(
                        LineagePostureKind::Retire,
                        *target,
                        "basis record is already claimed by another posture in this commit",
                    ));
                }
                let source_lineage = require_live_holder_and_lineage(
                    LineagePostureKind::Retire,
                    staged,
                    input,
                    *target,
                    *name,
                )?;
                retired_names_in_commit.insert(*name);
                validated.push(ValidatedPosture::Retire {
                    subject: target.subject(),
                    source_lineage,
                });
            }
            LineagePostureClaim::Split {
                parent_target,
                parent_name,
                children,
            } => {
                if children.is_empty() {
                    return Err(mismatch(
                        LineagePostureKind::Split,
                        *parent_target,
                        "Split posture requires at least one created successor",
                    ));
                }
                if !existed_at_basis(staged, *parent_target, version_id)
                    || !is_retired_in_commit(staged, *parent_target, version_id)
                {
                    return Err(mismatch(
                        LineagePostureKind::Split,
                        *parent_target,
                        "Split posture requires its parent to exist at basis and be retired in this commit",
                    ));
                }
                if !claimed_basis_targets.insert(*parent_target) {
                    return Err(mismatch(
                        LineagePostureKind::Split,
                        *parent_target,
                        "Split parent is already claimed by another posture in this commit",
                    ));
                }
                let parent_lineage = require_live_holder_and_lineage(
                    LineagePostureKind::Split,
                    staged,
                    input,
                    *parent_target,
                    *parent_name,
                )?;
                retired_names_in_commit.insert(*parent_name);

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
                        input,
                    )?;
                    if let Some(scheme) = input.scheme_registry.get(&child.birth.scheme_id)
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
                        normalized_birth.parent_identities = vec![*parent_name];
                    } else if normalized_birth.parent_identities != vec![*parent_name] {
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
                validated.push(ValidatedPosture::Split {
                    subject: parent_target.subject(),
                    parent_lineage,
                    children: child_targets,
                });
            }
            LineagePostureClaim::Merge {
                parents,
                target,
                keeper,
            } => {
                let validated_merge = validate_merge_claim(
                    staged,
                    version_id,
                    input,
                    &created_targets,
                    parents,
                    *target,
                    keeper,
                    &mut claimed_created_targets,
                    &mut claimed_basis_targets,
                    &mut retired_names_in_commit,
                    &mut kept_names_in_commit,
                    &mut births_to_mint,
                )?;
                validated.push(validated_merge);
            }
        }
    }

    let minted_births = super::name_minting::mint_target_births(
        &input.scheme_registry,
        &input.live_holders,
        &retired_names_in_commit,
        &created_targets,
        &births_to_mint,
    )?;

    Ok(ValidatedPostureBatch {
        postures: validated,
        minted_births,
        claimed_created_targets,
        claimed_basis_targets,
    })
}

pub(super) fn validate_birth_holder_and_subject(
    posture: LineagePostureKind,
    target: LineageRecordTarget,
    birth: &PersistentNameBirthRequest,
    input: &LineageCommitPostureInput,
) -> Result<(), LineageNameDenial> {
    if birth.holder != target.to_holder() {
        return Err(mismatch(
            posture,
            target,
            "birth request holder does not match posture target record",
        ));
    }
    if let Some(scheme) = input.scheme_registry.get(&birth.scheme_id)
        && scheme.subject_kind() != target.subject_kind()
    {
        return Err(mismatch(
            posture,
            target,
            "birth request scheme subject kind does not match target subject kind",
        ));
    }
    Ok(())
}

pub(super) fn require_live_holder_and_lineage(
    posture: LineagePostureKind,
    staged: &WorkingState,
    input: &LineageCommitPostureInput,
    target: LineageRecordTarget,
    name: PersistentNameIdentity,
) -> Result<LineageId, LineageNameDenial> {
    if input.live_holders.get(&name) != Some(&target.to_holder()) {
        return Err(mismatch(
            posture,
            target,
            "persistent name does not match a live basis holder for this target",
        ));
    }
    basis_lineage_id(staged, &input.live_lineages, target).ok_or_else(|| {
        mismatch(
            posture,
            target,
            "basis record has no live lineage identity at commit start",
        )
    })
}

pub(super) fn mismatch(
    posture: LineagePostureKind,
    target: LineageRecordTarget,
    detail: &str,
) -> LineageNameDenial {
    LineageNameDenial::LineagePostureMismatch {
        posture,
        subject: target.subject(),
        holder: target.to_holder(),
        detail: detail.to_owned(),
    }
}
