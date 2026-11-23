//! Phase 5 re-minting and cross-commit restore classification:
//!
//! 1. **In-commit regeneration (`Preserve` / `Replace` continuity):** a live
//!    holder retired by `LineagePostureClaim::Retire` whose exact identity is
//!    re-minted by `LineagePostureClaim::Create` in the same commit with the
//!    same subject kind upgrades into canonical `Replace` continuity onto the
//!    fresh generation without emitting a false `Retire`.
//! 2. **Cross-commit restore (`LineageEventKind::Restore`):** an explicit
//!    `LineagePostureClaim::Restore` or a birth whose minted identity matches a
//!    previously retired name in `retired_lineages` (not live at basis and not
//!    retired in this commit) emits `LineageEventKind::Restore` linking
//!    `[prior_retired_lineage_id] -> [new_lineage_id]`.
//! 3. **Live conflict guard (`DuplicateBirth`):** attempting to mint or restore
//!    an identity whose holder is still live and not retired in this commit, or
//!    claiming the same restored identity twice in one commit, refuses typed
//!    with zero effects before allocator reservation.

use std::collections::{BTreeMap, BTreeSet};

use crate::identity::data::LineageId;
use crate::lineage::data::{
    LineageBasisView, LineageNameDenial, LineagePostureKind, LineageRecordTarget,
    PersistentNameBirthRecord, PersistentNameHolder, PersistentNameIdentity,
};

use super::posture_validation::{mismatch, ValidatedPosture};

pub(super) struct ContinuityClassification {
    pub(super) postures: Vec<ValidatedPosture>,
    pub(super) restored_created_targets: BTreeSet<LineageRecordTarget>,
}

/// Validates explicit `LineagePostureClaim::Restore` claims against live basis
/// holders, same-commit retirements, and previously retired basis lineages.
pub(super) fn validate_explicit_restores(
    basis: &dyn LineageBasisView,
    retired_names_in_commit: &BTreeSet<PersistentNameIdentity>,
    explicit_restores: &[(LineageRecordTarget, PersistentNameIdentity)],
) -> Result<
    (
        Vec<ValidatedPosture>,
        BTreeMap<PersistentNameIdentity, PersistentNameHolder>,
        BTreeSet<LineageRecordTarget>,
    ),
    LineageNameDenial,
> {
    let mut restored_postures = Vec::with_capacity(explicit_restores.len());
    let mut restored_names = BTreeMap::<PersistentNameIdentity, PersistentNameHolder>::new();
    let mut restored_targets = BTreeSet::new();

    for &(target, name) in explicit_restores {
        if let Some(first) = restored_names.get(&name) {
            return Err(LineageNameDenial::DuplicateBirth {
                identity: name,
                first: first.clone(),
                second: target.to_holder(),
            });
        }
        if let Some(live_holder) = basis.live_holder(&name) {
            if !retired_names_in_commit.contains(&name) {
                return Err(LineageNameDenial::DuplicateBirth {
                    identity: name,
                    first: live_holder.clone(),
                    second: target.to_holder(),
                });
            }
            return Err(mismatch(
                LineagePostureKind::Restore,
                target,
                "Restore posture targets a persistent name that was live at commit basis rather than previously retired",
            ));
        }
        let Some((retired_subject, source_lineage)) = basis.retired_lineage(&name) else {
            return Err(mismatch(
                LineagePostureKind::Restore,
                target,
                "Restore posture targets a persistent name not found in retired basis lineages",
            ));
        };
        if retired_subject != target.subject() {
            return Err(mismatch(
                LineagePostureKind::Restore,
                target,
                "Restore target subject kind does not match the retired lineage subject kind",
            ));
        }
        restored_names.insert(name, target.to_holder());
        restored_targets.insert(target);
        restored_postures.push(ValidatedPosture::Restore {
            target,
            source_lineage,
            name,
        });
    }

    Ok((restored_postures, restored_names, restored_targets))
}

/// Classifies minted birth records into fresh `Create`, same-commit
/// regeneration (`Regenerate`), or cross-commit `Restore`, mutating
/// `minted_births` so only fresh births retain a `PersistentNameBirthRecord`.
#[allow(clippy::too_many_arguments)]
pub(super) fn classify_minted_births(
    basis: &dyn LineageBasisView,
    validated_postures: Vec<ValidatedPosture>,
    explicit_retires: &BTreeMap<PersistentNameIdentity, (LineageRecordTarget, LineageId)>,
    retired_names_in_commit: &BTreeSet<PersistentNameIdentity>,
    create_claimed_targets: &BTreeSet<LineageRecordTarget>,
    explicit_restore_postures: Vec<ValidatedPosture>,
    mut restored_names_in_commit: BTreeMap<PersistentNameIdentity, PersistentNameHolder>,
    mut restored_created_targets: BTreeSet<LineageRecordTarget>,
    minted_births: &mut BTreeMap<LineageRecordTarget, PersistentNameBirthRecord>,
) -> Result<ContinuityClassification, LineageNameDenial> {
    let mut regenerated_creates =
        BTreeMap::<LineageRecordTarget, (LineageId, PersistentNameIdentity)>::new();
    let mut consumed_retires = BTreeSet::<PersistentNameIdentity>::new();
    let mut birth_restores =
        BTreeMap::<LineageRecordTarget, (LineageId, PersistentNameIdentity)>::new();

    let minted_identities: Vec<(LineageRecordTarget, PersistentNameIdentity)> = minted_births
        .iter()
        .map(|(&target, record)| (target, record.identity()))
        .collect();

    for (target, id) in minted_identities {
        if let Some(first) = restored_names_in_commit.get(&id) {
            return Err(LineageNameDenial::DuplicateBirth {
                identity: id,
                first: first.clone(),
                second: target.to_holder(),
            });
        }

        if retired_names_in_commit.contains(&id) {
            let Some(&(old_target, old_lineage)) = explicit_retires.get(&id) else {
                return Err(mismatch(
                    LineagePostureKind::Create,
                    target,
                    "cannot re-mint a persistent name ended by Split or Merge in the same commit",
                ));
            };
            if !create_claimed_targets.contains(&target) {
                return Err(mismatch(
                    LineagePostureKind::Create,
                    target,
                    "in-commit regeneration requires a Create posture paired with Retire",
                ));
            }
            if old_target.subject() != target.subject() {
                return Err(mismatch(
                    LineagePostureKind::Create,
                    target,
                    "in-commit regeneration subject kind does not match the retired holder",
                ));
            }
            minted_births.remove(&target);
            consumed_retires.insert(id);
            regenerated_creates.insert(target, (old_lineage, id));
            continue;
        }

        if !basis.contains_live_holder(&id)
            && let Some((retired_subject, source_lineage)) = basis.retired_lineage(&id)
        {
            if retired_subject != target.subject() {
                return Err(mismatch(
                    LineagePostureKind::Restore,
                    target,
                    "birth-classified Restore target subject kind does not match the retired lineage subject kind",
                ));
            }
            minted_births.remove(&target);
            restored_names_in_commit.insert(id, target.to_holder());
            restored_created_targets.insert(target);
            birth_restores.insert(target, (source_lineage, id));
        }
    }

    let mut final_postures = Vec::with_capacity(
        validated_postures.len() + explicit_restore_postures.len() + birth_restores.len(),
    );
    let mut handled_birth_restores = BTreeSet::new();

    for posture in validated_postures {
        match posture {
            ValidatedPosture::Create { target } => {
                if let Some((source_lineage, name)) = regenerated_creates.remove(&target) {
                    final_postures.push(ValidatedPosture::Regenerate {
                        subject: target.subject(),
                        source_lineage,
                        target,
                        name,
                    });
                } else if let Some(&(source_lineage, name)) = birth_restores.get(&target) {
                    handled_birth_restores.insert(target);
                    final_postures.push(ValidatedPosture::Restore {
                        target,
                        source_lineage,
                        name,
                    });
                } else {
                    final_postures.push(ValidatedPosture::Create { target });
                }
            }
            ValidatedPosture::Retire { name, .. } if consumed_retires.contains(&name) => {}
            other => final_postures.push(other),
        }
    }

    // Any remaining birth-classified restores come from Split children or
    // Merge(MintNew) targets redoing previously retired identities.
    for (target, (source_lineage, name)) in birth_restores {
        if !handled_birth_restores.contains(&target) {
            final_postures.push(ValidatedPosture::Restore {
                target,
                source_lineage,
                name,
            });
        }
    }
    final_postures.extend(explicit_restore_postures);

    Ok(ContinuityClassification {
        postures: final_postures,
        restored_created_targets,
    })
}
