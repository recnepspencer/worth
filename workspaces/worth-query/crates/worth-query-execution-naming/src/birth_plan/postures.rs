//! Explicit lifecycle posture (`Retire`, `Split`, `Merge`) assembly onto
//! [`ExecutionNameBirthPlanBuilder`].

use std::collections::BTreeMap;

use worth_query_declaration::facade::application_naming::ApplicationNameSubjectKind;
use worth_relational::facade::identity::{EntityId, RelationId};
use worth_relational::facade::lineage::{
    LineageMergeKeeper, LineagePostureClaim, LineageRecordTarget, LineageSplitChild,
    PersistentNameBirthRequest, PersistentNameIdentity,
};

use crate::birth_plan::assembly::ExecutionNameBirthPlanBuilder;
use crate::denial::ExecutionNamingDenial;

/// One created child inside an explicit `Split` posture.
#[derive(Clone, Debug)]
pub struct ExecutionSplitChildSpec {
    pub target: LineageRecordTarget,
    pub scheme_id: String,
    pub occurrence_key: String,
    pub role_path: String,
    pub canonical_discriminator: Vec<u8>,
    pub embedded_name_digests: Vec<[u8; 32]>,
    pub same_occurrence_roles: BTreeMap<[u8; 32], String>,
}

/// Keeper choice for an explicit `Merge` posture.
#[derive(Clone, Debug)]
pub enum ExecutionNamedMergeKeeper {
    KeepParent(PersistentNameIdentity),
    MintNew {
        scheme_id: String,
        occurrence_key: String,
        role_path: String,
        canonical_discriminator: Vec<u8>,
        embedded_name_digests: Vec<[u8; 32]>,
    },
}

impl<'a> ExecutionNameBirthPlanBuilder<'a> {
    /// Registers an explicit `Retire` posture for a committed entity record.
    pub fn add_retire_entity(
        &mut self,
        entity_id: EntityId,
        identity: PersistentNameIdentity,
    ) -> &mut Self {
        self.push_posture(LineagePostureClaim::Retire {
            target: LineageRecordTarget::Entity(entity_id),
            name: identity,
        });
        self
    }

    /// Registers an explicit `Retire` posture for a committed relation record.
    pub fn add_retire_relation(
        &mut self,
        relation_id: RelationId,
        identity: PersistentNameIdentity,
    ) -> &mut Self {
        self.push_posture(LineagePostureClaim::Retire {
            target: LineageRecordTarget::Relation(relation_id),
            name: identity,
        });
        self
    }

    /// Registers an explicit `Split` posture over concrete created records.
    pub fn add_split_posture(
        &mut self,
        parent_target: LineageRecordTarget,
        parent_name: PersistentNameIdentity,
        children: Vec<ExecutionSplitChildSpec>,
    ) -> Result<&mut Self, ExecutionNamingDenial> {
        let mut lowered_children = Vec::with_capacity(children.len());
        for child in children {
            let expected_kind = match child.target {
                LineageRecordTarget::Entity(_) => ApplicationNameSubjectKind::Entity,
                LineageRecordTarget::Relation(_) => ApplicationNameSubjectKind::Relation,
            };
            let layout = self.require_scheme(expected_kind, &child.scheme_id, &child.role_path)?;
            if layout.require_authored_occurrence() && child.occurrence_key.trim().is_empty() {
                return Err(ExecutionNamingDenial::MissingAuthoredOccurrenceKey {
                    scheme_id: child.scheme_id,
                });
            }
            if child.canonical_discriminator.is_empty() {
                return Err(ExecutionNamingDenial::MissingSplitDiscriminator {
                    scheme_id: child.scheme_id,
                    role_path: child.role_path,
                });
            }
            let role_rank = layout.role_rank(&child.role_path).expect("validated role");
            let referenced_names = self.resolve_references(
                layout,
                &child.role_path,
                role_rank,
                &child.embedded_name_digests,
                &child.same_occurrence_roles,
            )?;
            lowered_children.push(LineageSplitChild {
                target: child.target,
                birth: PersistentNameBirthRequest {
                    scheme_id: child.scheme_id,
                    occurrence_key: child.occurrence_key,
                    role_path: child.role_path,
                    parent_identities: vec![parent_name],
                    referenced_names,
                    canonical_discriminator: child.canonical_discriminator,
                    holder: child.target.to_holder(),
                },
            });
        }
        self.push_posture(LineagePostureClaim::Split {
            parent_target,
            parent_name,
            children: lowered_children,
        });
        Ok(self)
    }

    /// Registers an explicit `Merge` posture over concrete records.
    pub fn add_merge_posture(
        &mut self,
        parents: Vec<(LineageRecordTarget, PersistentNameIdentity)>,
        target: LineageRecordTarget,
        keeper: ExecutionNamedMergeKeeper,
    ) -> Result<&mut Self, ExecutionNamingDenial> {
        let keeper = match keeper {
            ExecutionNamedMergeKeeper::KeepParent(id) => LineageMergeKeeper::KeepParent(id),
            ExecutionNamedMergeKeeper::MintNew {
                scheme_id,
                occurrence_key,
                role_path,
                canonical_discriminator,
                embedded_name_digests,
            } => {
                let expected_kind = match target {
                    LineageRecordTarget::Entity(_) => ApplicationNameSubjectKind::Entity,
                    LineageRecordTarget::Relation(_) => ApplicationNameSubjectKind::Relation,
                };
                let layout = self.require_scheme(expected_kind, &scheme_id, &role_path)?;
                let role_rank = layout.role_rank(&role_path).expect("validated role");
                let referenced_names = self.resolve_references(
                    layout,
                    &role_path,
                    role_rank,
                    &embedded_name_digests,
                    &BTreeMap::new(),
                )?;
                let mut parent_ids: Vec<_> = parents.iter().map(|(_, id)| *id).collect();
                parent_ids.sort_unstable();
                LineageMergeKeeper::MintNew {
                    birth: PersistentNameBirthRequest {
                        scheme_id,
                        occurrence_key,
                        role_path,
                        parent_identities: parent_ids,
                        referenced_names,
                        canonical_discriminator,
                        holder: target.to_holder(),
                    },
                }
            }
        };
        self.push_posture(LineagePostureClaim::Merge {
            parents,
            target,
            keeper,
        });
        Ok(self)
    }
}
