//! Strict fail-closed validation of Phase 2 entity and relation lineage
//! postures against actual performed commit effects in `WorkingState`.

use std::collections::{BTreeMap, BTreeSet};

use crate::identity::data::{LineageId, VersionId};
use crate::lineage::data::{
    InstalledPersistentNameSchemeRegistry, LineageBasisView, LineageNameDenial,
    LineagePostureClaim, LineagePostureKind, LineageRecordTarget, LineageSubject,
    PersistentNameBirthRecord, PersistentNameBirthRequest, PersistentNameIdentity,
};
use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::overlay::WorkingState;
use crate::transactions::data::primitives::RecordRef;

use super::posture_continuity::{classify_minted_births, validate_explicit_restores};
use super::posture_merge_validation::validate_merge_claim;
use super::posture_split_validation::validate_split_claim;
use super::staged_effects::{
    basis_lineage_id, existed_at_basis, is_created_in_commit, is_live_after_commit,
    is_retired_in_commit, target_kind_id,
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
    Regenerate {
        subject: LineageSubject,
        source_lineage: LineageId,
        target: LineageRecordTarget,
        name: PersistentNameIdentity,
    },
    Restore {
        target: LineageRecordTarget,
        source_lineage: LineageId,
        name: PersistentNameIdentity,
    },
    Retire {
        subject: LineageSubject,
        source_lineage: LineageId,
        name: PersistentNameIdentity,
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
    pub(super) restored_created_targets: BTreeSet<LineageRecordTarget>,
    pub(super) claimed_basis_targets: BTreeSet<LineageRecordTarget>,
}

/// Validates every declared entity and relation posture against the actual
/// staged creations and retirements in `staged`, then mints all required
/// persistent names via [`super::name_minting::mint_target_births`].
///
/// Pure and fail-closed: performs zero allocator reservations and zero
/// mutations to `staged`.
#[allow(clippy::too_many_arguments)]
pub(super) fn validate_commit_postures(
    staged: &WorkingState,
    version_id: VersionId,
    changed_records: &[RecordRef],
    scheme_registry: &InstalledPersistentNameSchemeRegistry,
    schema_registry: Option<&RelationalSchemaRegistry>,
    basis: &dyn LineageBasisView,
    postures: &[LineagePostureClaim],
) -> Result<ValidatedPostureBatch, LineageNameDenial> {
    let check_missing_births =
        schema_registry.is_some_and(RelationalSchemaRegistry::has_kind_name_schemes);
    if postures.is_empty() && !check_missing_births {
        return Ok(ValidatedPostureBatch {
            postures: Vec::new(),
            minted_births: BTreeMap::new(),
            claimed_created_targets: BTreeSet::new(),
            restored_created_targets: BTreeSet::new(),
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
    let mut create_claimed_targets = BTreeSet::new();
    let mut claimed_basis_targets = BTreeSet::new();
    let mut retired_names_in_commit = BTreeSet::new();
    let mut explicit_retires = BTreeMap::new();
    let mut explicit_restores = Vec::new();
    let mut kept_names_in_commit = BTreeSet::new();
    let mut births_to_mint = BTreeMap::new();
    let mut validated = Vec::with_capacity(postures.len());

    for claim in postures {
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
                    scheme_registry,
                )?;
                create_claimed_targets.insert(*target);
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
                    basis,
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
                    basis,
                    *target,
                    *name,
                )?;
                retired_names_in_commit.insert(*name);
                explicit_retires.insert(*name, (*target, source_lineage));
                validated.push(ValidatedPosture::Retire {
                    subject: target.subject(),
                    source_lineage,
                    name: *name,
                });
            }
            LineagePostureClaim::Restore { target, name } => {
                if !created_targets.contains(target) {
                    return Err(mismatch(
                        LineagePostureKind::Restore,
                        *target,
                        "Restore posture targets a record not created and live in this commit",
                    ));
                }
                if !claimed_created_targets.insert(*target) {
                    return Err(mismatch(
                        LineagePostureKind::Restore,
                        *target,
                        "record is already claimed by another posture in this commit",
                    ));
                }
                explicit_restores.push((*target, *name));
            }
            LineagePostureClaim::Split {
                parent_target,
                parent_name,
                children,
            } => {
                let validated_split = validate_split_claim(
                    staged,
                    version_id,
                    scheme_registry,
                    basis,
                    &created_targets,
                    *parent_target,
                    *parent_name,
                    children,
                    &mut claimed_created_targets,
                    &mut claimed_basis_targets,
                    &mut retired_names_in_commit,
                    &mut births_to_mint,
                )?;
                validated.push(validated_split);
            }
            LineagePostureClaim::Merge {
                parents,
                target,
                keeper,
            } => {
                let validated_merge = validate_merge_claim(
                    staged,
                    version_id,
                    scheme_registry,
                    basis,
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

    // Fail closed if any schemed created record lacks a birth/posture or claims a different kind's scheme.
    if let Some(schema) = schema_registry {
        for &target in &created_targets {
            let Some(kind_id) = target_kind_id(staged, target) else {
                continue;
            };
            let Some(scheme_id) = schema.scheme_id_for_kind(target.subject_kind(), kind_id) else {
                continue;
            };
            if !claimed_created_targets.contains(&target) {
                return Err(LineageNameDenial::MissingBirth {
                    target,
                    scheme_id: scheme_id.to_owned(),
                });
            }
            if let Some(birth) = births_to_mint.get(&target)
                && birth.scheme_id != scheme_id
            {
                return Err(mismatch(
                    LineagePostureKind::Create,
                    target,
                    &format!(
                        "birth scheme `{}` does not match kind's installed scheme `{scheme_id}`",
                        birth.scheme_id
                    ),
                ));
            }
        }
    }

    let (explicit_restore_postures, restored_names_in_commit, restored_created_targets) =
        validate_explicit_restores(basis, &retired_names_in_commit, &explicit_restores)?;
    let mut minted_births = super::name_minting::mint_target_births(
        scheme_registry,
        basis,
        &retired_names_in_commit,
        &created_targets,
        &births_to_mint,
    )?;
    let classified = classify_minted_births(
        basis,
        validated,
        &explicit_retires,
        &retired_names_in_commit,
        &create_claimed_targets,
        explicit_restore_postures,
        restored_names_in_commit,
        restored_created_targets,
        &mut minted_births,
    )?;

    Ok(ValidatedPostureBatch {
        postures: classified.postures,
        minted_births,
        claimed_created_targets,
        restored_created_targets: classified.restored_created_targets,
        claimed_basis_targets,
    })
}

pub(super) fn validate_birth_holder_and_subject(
    posture: LineagePostureKind,
    target: LineageRecordTarget,
    birth: &PersistentNameBirthRequest,
    scheme_registry: &InstalledPersistentNameSchemeRegistry,
) -> Result<(), LineageNameDenial> {
    if birth.holder != target.to_holder() {
        return Err(mismatch(
            posture,
            target,
            "birth request holder does not match posture target record",
        ));
    }
    if let Some(scheme) = scheme_registry.get(&birth.scheme_id)
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
    basis: &dyn LineageBasisView,
    target: LineageRecordTarget,
    name: PersistentNameIdentity,
) -> Result<LineageId, LineageNameDenial> {
    if basis.live_holder(&name) != Some(&target.to_holder()) {
        return Err(mismatch(
            posture,
            target,
            "persistent name does not match a live basis holder for this target",
        ));
    }
    basis_lineage_id(staged, basis, target).ok_or_else(|| {
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
