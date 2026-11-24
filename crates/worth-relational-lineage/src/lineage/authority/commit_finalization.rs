use crate::contracts::lineage::data::naming::{PersistentNameBirthRequest, PersistentNameHolder};
use crate::contracts::lineage::data::naming_denial::LineageNameDenial;
use crate::contracts::lineage::data::postures::{
    CommitNamingPlan, LineageMergeKeeper, LineagePostureClaim, LineageRecordTarget,
    LineageSplitChild, PendingBirthTarget, PendingMergeKeeper, PendingMergeTarget,
};
use crate::contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;

use crate::contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings;

fn resolve_pending_target(
    pending: &PendingBirthTarget,
    scheme_id: &str,
    occurrence_key: &str,
    role_path: &str,
    ordinal: usize,
    created_entities: &CommitCreatedEntityBindings,
    created_relations: &CommitCreatedRelationBindings,
) -> Result<LineageRecordTarget, LineageNameDenial> {
    match pending {
        PendingBirthTarget::CreatedEntity(created_ref) => created_entities
            .resolve(created_ref)
            .map(LineageRecordTarget::Entity)
            .ok_or_else(|| LineageNameDenial::BirthWithoutCreate {
                scheme_id: scheme_id.to_owned(),
                occurrence_key: occurrence_key.to_owned(),
                role_path: role_path.to_owned(),
                holder: PersistentNameHolder::PendingBirth {
                    occurrence_key: occurrence_key.to_owned(),
                    role_path: role_path.to_owned(),
                    ordinal,
                },
            }),
        PendingBirthTarget::CreatedRelation(created_ref) => created_relations
            .resolve(created_ref)
            .map(LineageRecordTarget::Relation)
            .ok_or_else(|| LineageNameDenial::BirthWithoutCreate {
                scheme_id: scheme_id.to_owned(),
                occurrence_key: occurrence_key.to_owned(),
                role_path: role_path.to_owned(),
                holder: PersistentNameHolder::PendingBirth {
                    occurrence_key: occurrence_key.to_owned(),
                    role_path: role_path.to_owned(),
                    ordinal,
                },
            }),
    }
}

pub fn resolve_commit_naming_plan(
    plan: &CommitNamingPlan,
    created_entities: &CommitCreatedEntityBindings,
    created_relations: &CommitCreatedRelationBindings,
) -> Result<Vec<LineagePostureClaim>, LineageNameDenial> {
    let mut postures = Vec::with_capacity(
        plan.pending_births.len()
            + plan.pending_splits.len()
            + plan.pending_merges.len()
            + plan.postures.len(),
    );
    for (ordinal, pending) in plan.pending_births.iter().enumerate() {
        let target = resolve_pending_target(
            &pending.target,
            &pending.scheme_id,
            &pending.occurrence_key,
            &pending.role_path,
            ordinal,
            created_entities,
            created_relations,
        )?;
        postures.push(LineagePostureClaim::Create {
            target,
            birth: PersistentNameBirthRequest {
                scheme_id: pending.scheme_id.clone(),
                occurrence_key: pending.occurrence_key.clone(),
                role_path: pending.role_path.clone(),
                parent_identities: pending.parent_identities.clone(),
                referenced_names: pending.referenced_names.clone(),
                canonical_discriminator: pending.canonical_discriminator.clone(),
                holder: target.to_holder(),
            },
        });
    }
    for split in &plan.pending_splits {
        let mut children = Vec::with_capacity(split.children.len());
        for (ordinal, child) in split.children.iter().enumerate() {
            let target = resolve_pending_target(
                &child.target,
                &child.scheme_id,
                &child.occurrence_key,
                &child.role_path,
                ordinal,
                created_entities,
                created_relations,
            )?;
            children.push(LineageSplitChild {
                target,
                birth: PersistentNameBirthRequest {
                    scheme_id: child.scheme_id.clone(),
                    occurrence_key: child.occurrence_key.clone(),
                    role_path: child.role_path.clone(),
                    parent_identities: vec![split.parent_name],
                    referenced_names: child.referenced_names.clone(),
                    canonical_discriminator: child.canonical_discriminator.clone(),
                    holder: target.to_holder(),
                },
            });
        }
        postures.push(LineagePostureClaim::Split {
            parent_target: split.parent_target,
            parent_name: split.parent_name,
            children,
        });
    }
    for (ordinal, merge) in plan.pending_merges.iter().enumerate() {
        let target = match &merge.target {
            PendingMergeTarget::Existing(existing) => *existing,
            PendingMergeTarget::Created(pending) => {
                let (sid, occ, role) = match &merge.keeper {
                    PendingMergeKeeper::KeepParent(_) => ("", "", ""),
                    PendingMergeKeeper::MintNew {
                        scheme_id,
                        occurrence_key,
                        role_path,
                        ..
                    } => (
                        scheme_id.as_str(),
                        occurrence_key.as_str(),
                        role_path.as_str(),
                    ),
                };
                resolve_pending_target(
                    pending,
                    sid,
                    occ,
                    role,
                    ordinal,
                    created_entities,
                    created_relations,
                )?
            }
        };
        let keeper = match &merge.keeper {
            PendingMergeKeeper::KeepParent(id) => LineageMergeKeeper::KeepParent(*id),
            PendingMergeKeeper::MintNew {
                scheme_id,
                occurrence_key,
                role_path,
                referenced_names,
                canonical_discriminator,
            } => {
                let mut parent_ids: Vec<_> = merge.parents.iter().map(|(_, id)| *id).collect();
                parent_ids.sort_unstable();
                LineageMergeKeeper::MintNew {
                    birth: PersistentNameBirthRequest {
                        scheme_id: scheme_id.clone(),
                        occurrence_key: occurrence_key.clone(),
                        role_path: role_path.clone(),
                        parent_identities: parent_ids,
                        referenced_names: referenced_names.clone(),
                        canonical_discriminator: canonical_discriminator.clone(),
                        holder: target.to_holder(),
                    },
                }
            }
        };
        postures.push(LineagePostureClaim::Merge {
            parents: merge.parents.clone(),
            target,
            keeper,
        });
    }
    postures.extend(plan.postures.iter().cloned());
    Ok(postures)
}
