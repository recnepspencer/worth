//! Reconstructs and re-verifies a commit's [`CommitNamingPlan`] from its
//! durable canonical [`CanonicalCommitEnvelope`] during journal tail replay so
//! replayed commits execute through the full production naming/posture
//! finalization path and reproduce exact canonical `LineageEventRecord`s.

use contracts::lineage::data::name_resolution::PersistentNameKey;
use contracts::lineage::data::naming::{
    PersistentNameBirthParts, PersistentNameBirthRecord, PersistentNameBirthRequest,
    PersistentNameDiscriminatorReference, PersistentNameHolder, PersistentNameIdentity,
    PersistentNameIdentityOwnerPort as _, PersistentNameReferenceBasis,
};
use std::collections::{BTreeMap, BTreeSet};
use worth_relational_contracts::internal_port as contracts;

use crate::durability::authority::lineage_allocation::{
    entity_allocation_bindings, CommitAllocationView,
};
use contracts::durability::data::recovery_errors::{DurabilityError, RecoveryFailureClass};
use contracts::identity::data::primitives::LineageId;
use contracts::identity::data::record_ids::{EntityId, RelationId};
use contracts::lineage::data::birth_wire::PersistedBirthClaim;
use contracts::lineage::data::discriminator_scan::extract_embedded_name_refs;
use contracts::lineage::data::events::{LineageEventKind, LineageSubject};
use contracts::lineage::data::postures::{
    CommitNamingPlan, LineageBasisView, LineageMergeKeeper, LineagePostureClaim,
    LineageRecordTarget, LineageSplitChild,
};
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use worth_relational_lineage::internal_port::lineage::authority::name_readmission::verify_persisted_birth_digest;
use worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::LineageSubsystem;

pub fn reconstruct_replay_naming_plan(
    lineage: &LineageSubsystem,
    envelope: &CanonicalCommitEnvelope,
) -> Result<Option<CommitNamingPlan>, DurabilityError> {
    let events = envelope.lineage_events();
    let group_transitions = super::group_recovery::replay_group_transitions(envelope)?;
    let has_naming_or_postures = !group_transitions.is_empty()
        || events.iter().any(|event| {
            event.name_birth().is_some()
                || event.kept_name().is_some()
                || event.subject() == LineageSubject::Relation
                || matches!(
                    event.kind(),
                    LineageEventKind::Split | LineageEventKind::Merge | LineageEventKind::Restore
                )
        });
    if !has_naming_or_postures {
        return Ok(None);
    }

    let branch_id = &envelope.branch_context;
    let entity_by_lineage: BTreeMap<LineageId, EntityId> =
        entity_allocation_bindings(&CommitAllocationView::of_envelope(envelope))
            .map_err(|refusal| {
                DurabilityError::new(
                    RecoveryFailureClass::ReplayFailure,
                    refusal.describe(envelope.commit.commit_id.0),
                )
            })?
            .into_iter()
            .collect();
    let relation_by_lineage: BTreeMap<LineageId, RelationId> = envelope
        .lineage_relation_lineages()
        .iter()
        .map(|&(relation_id, lineage_id)| (lineage_id, relation_id))
        .collect();

    let resolve_created_target = |subject: LineageSubject, lineage_id: LineageId| {
        match subject {
            LineageSubject::Entity => entity_by_lineage
                .get(&lineage_id)
                .copied()
                .map(LineageRecordTarget::Entity),
            LineageSubject::Relation => relation_by_lineage
                .get(&lineage_id)
                .copied()
                .map(LineageRecordTarget::Relation),
        }
        .ok_or_else(|| {
            DurabilityError::new(
                RecoveryFailureClass::ReplayFailure,
                format!(
                    "replay could not bind created {:?} lineage {}",
                    subject, lineage_id.0
                ),
            )
        })
    };

    let resolve_basis_target = |lineage_id: LineageId| -> Result<
        (LineageRecordTarget, PersistentNameIdentity),
        DurabilityError,
    > {
        let Some((holder, name)) = lineage
            .holder_and_name_for_lineage(branch_id, lineage_id)
            .map_err(naming_unavailable_error)?
        else {
            return Err(DurabilityError::new(
                RecoveryFailureClass::ReplayFailure,
                format!(
                    "replay could not resolve live basis holder/name for lineage {}",
                    lineage_id.0
                ),
            ));
        };
        let target = match holder {
            PersistentNameHolder::Entity(id) => LineageRecordTarget::Entity(id),
            PersistentNameHolder::Relation(id) => LineageRecordTarget::Relation(id),
            other => {
                return Err(DurabilityError::new(
                    RecoveryFailureClass::ReplayFailure,
                    format!("unexpected non-record holder {other:?} during replay"),
                ));
            }
        };
        Ok((target, name))
    };

    // Index all verified birth records in this envelope by their target lineage
    // and collect same-occurrence role provenance for discriminator references.
    let mut create_births =
        BTreeMap::<LineageId, (LineageRecordTarget, PersistentNameBirthRecord)>::new();
    let mut same_occurrence_roles = BTreeMap::<[u8; 32], (String, String)>::new();
    for event in events {
        if event.kind() == LineageEventKind::Create
            && let Some(birth) = event.name_birth()
            && let Some(&target_lineage) = event.targets().first()
        {
            reverify_birth(birth)?;
            let target = resolve_created_target(event.subject(), target_lineage)?;
            same_occurrence_roles.insert(
                *birth.identity().as_bytes(),
                (
                    birth.parts().occurrence_key().to_owned(),
                    birth.parts().role_path().to_owned(),
                ),
            );
            create_births.insert(target_lineage, (target, birth.clone()));
        }
    }

    // In-commit regenerations (`Replace` keeping its name) are reconstructed as
    // `Retire` + `Create` using their recorded births. Because every basis name
    // (live or retired) is admitted as `PersistentNameReferenceBasis::Basis`,
    // references to a regenerated name replay directly against the basis.
    let regenerated = regenerated_births(lineage, envelope)?;
    let basis_binding = lineage.naming_basis_binding();
    let basis = basis_binding.read_guard(branch_id.clone());
    let build_birth_request = |target: LineageRecordTarget,
                               parts: &PersistentNameBirthParts|
     -> Result<PersistentNameBirthRequest, DurabilityError> {
        let embedded =
            extract_embedded_name_refs(parts.canonical_discriminator()).map_err(|()| {
                DurabilityError::new(
                    RecoveryFailureClass::ReplayFailure,
                    "replay found malformed discriminator bytes in durable birth".to_owned(),
                )
            })?;
        let mut seen = BTreeSet::new();
        let mut referenced_names = Vec::new();
        for digest in embedded {
            if !seen.insert(digest) {
                continue;
            }
            let identity = PersistentNameIdentity::from_minted_digest(digest);
            let ref_basis =
                if basis.contains_live_holder(&identity) || basis.contains_retired(&identity) {
                    PersistentNameReferenceBasis::Basis
                } else if let Some((occ, role)) = same_occurrence_roles.get(&digest)
                    && occ == parts.occurrence_key()
                {
                    PersistentNameReferenceBasis::SameOccurrence { role: role.clone() }
                } else {
                    PersistentNameReferenceBasis::Basis
                };
            referenced_names.push(PersistentNameDiscriminatorReference {
                identity,
                basis: ref_basis,
            });
        }
        Ok(PersistentNameBirthRequest {
            scheme_id: parts.scheme_id().to_owned(),
            occurrence_key: parts.occurrence_key().to_owned(),
            role_path: parts.role_path().to_owned(),
            parent_identities: parts.parent_identities().to_vec(),
            referenced_names,
            canonical_discriminator: parts.canonical_discriminator().to_vec(),
            holder: target.to_holder(),
        })
    };

    let mut consumed_create_lineages = BTreeSet::<LineageId>::new();
    let mut postures = Vec::new();

    // 1. Reconstruct Split, Merge, Replace-with-kept-name, Retire, and Restore.
    for event in events {
        match event.kind() {
            LineageEventKind::Split => {
                let Some(&parent_lineage) = event.sources().first() else {
                    continue;
                };
                let (parent_target, parent_name) = resolve_basis_target(parent_lineage)?;
                let mut children = Vec::with_capacity(event.targets().len());
                for &child_lineage in event.targets() {
                    let Some((child_target, birth)) = create_births.get(&child_lineage) else {
                        return Err(DurabilityError::new(
                            RecoveryFailureClass::ReplayFailure,
                            format!(
                                "split replay missing child Create birth for lineage {}",
                                child_lineage.0
                            ),
                        ));
                    };
                    consumed_create_lineages.insert(child_lineage);
                    children.push(LineageSplitChild {
                        target: *child_target,
                        birth: build_birth_request(*child_target, birth.parts())?,
                    });
                }
                postures.push(LineagePostureClaim::Split {
                    parent_target,
                    parent_name,
                    children,
                });
            }
            LineageEventKind::Merge => {
                let mut parents = Vec::with_capacity(event.sources().len());
                for &source_lineage in event.sources() {
                    parents.push(resolve_basis_target(source_lineage)?);
                }
                let Some(&target_lineage) = event.targets().first() else {
                    continue;
                };
                if let Some((target, birth)) = create_births.get(&target_lineage) {
                    consumed_create_lineages.insert(target_lineage);
                    postures.push(LineagePostureClaim::Merge {
                        parents,
                        target: *target,
                        keeper: LineageMergeKeeper::MintNew {
                            birth: build_birth_request(*target, birth.parts())?,
                        },
                    });
                } else if let Some(kept) = event.kept_name() {
                    let target = if event.sources().contains(&target_lineage) {
                        resolve_basis_target(target_lineage)?.0
                    } else {
                        consumed_create_lineages.insert(target_lineage);
                        resolve_created_target(event.subject(), target_lineage)?
                    };
                    postures.push(LineagePostureClaim::Merge {
                        parents,
                        target,
                        keeper: LineageMergeKeeper::KeepParent(kept),
                    });
                }
            }
            LineageEventKind::Retire => {
                if let Some(&source_lineage) = event.sources().first() {
                    match resolve_basis_target(source_lineage) {
                        Ok((target, name)) => {
                            postures.push(LineagePostureClaim::Retire { target, name });
                        }
                        Err(err) if err.class == RecoveryFailureClass::ColdStorageUnavailable => {
                            return Err(err);
                        }
                        Err(_) => {}
                    }
                }
            }
            LineageEventKind::Restore => {
                if let Some(&target_lineage) = event.targets().first()
                    && let Some(kept) = event.kept_name()
                {
                    let target = resolve_created_target(event.subject(), target_lineage)?;
                    postures.push(LineagePostureClaim::Restore { target, name: kept });
                }
            }
            LineageEventKind::Create | LineageEventKind::Replace => {}
        }
    }
    // In-commit regeneration: the kept name re-minted from its recorded birth.
    for (subject, source, created, _, parts) in &regenerated {
        let (target, name) = resolve_basis_target(*source)?;
        postures.push(LineagePostureClaim::Retire { target, name });
        let target = resolve_created_target(*subject, *created)?;
        let birth = build_birth_request(target, parts)?;
        postures.push(LineagePostureClaim::Create { target, birth });
    }

    // 2. Emit remaining standalone Create births.
    for event in events {
        if event.kind() == LineageEventKind::Create
            && let Some(&target_lineage) = event.targets().first()
            && !consumed_create_lineages.contains(&target_lineage)
            && let Some((target, birth)) = create_births.get(&target_lineage)
        {
            postures.push(LineagePostureClaim::Create {
                target: *target,
                birth: build_birth_request(*target, birth.parts())?,
            });
        }
    }

    let groups = super::group_recovery::replay_group_claims(envelope, group_transitions, &basis);
    basis.settle().map_err(naming_unavailable_error)?;
    let groups = groups?;
    if postures.is_empty() && groups.is_empty() {
        return Ok(None);
    }
    Ok(Some(CommitNamingPlan {
        postures,
        groups,
        ..Default::default()
    }))
}

fn naming_unavailable_error(
    denial: worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::NamingBasisUnavailable,
) -> DurabilityError {
    use worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::NamingBasisUnavailable;
    match denial {
        NamingBasisUnavailable::Storage(cause) => DurabilityError::new(
            RecoveryFailureClass::ColdStorageUnavailable,
            format!("cold naming page unavailable during replay: {cause:?}"),
        ),
        NamingBasisUnavailable::Unrecoverable => DurabilityError::new(
            RecoveryFailureClass::ReplayFailure,
            "branch naming basis is unavailable during replay",
        ),
    }
}

fn reverify_birth(birth: &PersistentNameBirthRecord) -> Result<(), DurabilityError> {
    let claim = PersistedBirthClaim {
        stored_identity: *birth.identity().as_bytes(),
        subject_kind: birth.subject_kind(),
        scheme_id: birth.parts().scheme_id().to_owned(),
        occurrence_key: birth.parts().occurrence_key().to_owned(),
        role_path: birth.parts().role_path().to_owned(),
        parent_identities: birth
            .parts()
            .parent_identities()
            .iter()
            .map(|id| *id.as_bytes())
            .collect(),
        canonical_discriminator: birth.parts().canonical_discriminator().to_vec(),
        structural_identity: birth.parts().structural_identity(),
    };
    verify_persisted_birth_digest(claim).map_err(|refusal| {
        DurabilityError::new(
            RecoveryFailureClass::ReplayFailure,
            format!("persisted birth failed digest verification during replay: {refusal:?}"),
        )
    })?;
    Ok(())
}

type Regenerated = (
    LineageSubject,
    LineageId,
    LineageId,
    PersistentNameIdentity,
    PersistentNameBirthParts,
);

/// Each regeneration (a `Replace` keeping its name) with the birth parts the
/// branch's index ingested from persisted events, read before the basis guard.
fn regenerated_births(
    lineage: &LineageSubsystem,
    envelope: &CanonicalCommitEnvelope,
) -> Result<Vec<Regenerated>, DurabilityError> {
    let mut births = Vec::new();
    for event in envelope.lineage_events() {
        let (LineageEventKind::Replace, Some(name), [source], [created]) = (
            event.kind(),
            event.kept_name(),
            event.sources(),
            event.targets(),
        ) else {
            continue;
        };
        let key = match event.subject() {
            LineageSubject::Entity => PersistentNameKey::Entity(name),
            LineageSubject::Relation => PersistentNameKey::Relation(name),
        };
        let parts = lineage.read_name_index(&envelope.branch_context, |index, scope| {
            index.and_then(|index| index.recorded_birth(key, scope).cloned())
        });
        let Ok(Some(parts)) = parts else {
            let detail = format!("regenerated name {name:?} has no recorded birth");
            return Err(DurabilityError::new(
                RecoveryFailureClass::ReplayFailure,
                detail,
            ));
        };
        births.push((event.subject(), *source, *created, name, parts));
    }
    Ok(births)
}
