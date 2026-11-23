//! Explicit lifecycle posture (`Retire`, `Split`, `Merge`) assembly onto
//! [`ExecutionNameBirthPlanBuilder`].

use std::collections::BTreeMap;

use worth_query_declaration::facade::application_naming::{
    ApplicationNameDiscriminator, ApplicationNameSubjectKind, WorthQueryName,
};
use worth_relational::facade::identity::{EntityId, RelationId};
use worth_relational::facade::lineage::{
    LineagePostureClaim, LineageRecordTarget, PendingBirthTarget, PendingMergeClaim,
    PendingMergeKeeper, PendingMergeTarget, PendingSplitChild, PendingSplitClaim,
    PersistentNameIdentity,
};
use worth_relational::facade::transactions::{CreatedEntityRef, CreatedRelationRef};

use crate::birth_plan::assembly::ExecutionNameBirthPlanBuilder;
use crate::denial::ExecutionNamingDenial;

/// One newly created successor child inside an explicit `Split` posture.
#[derive(Clone, Debug)]
pub struct ExecutionSplitChildSpec {
    pub target: PendingBirthTarget,
    pub scheme_id: String,
    pub occurrence_key: String,
    pub role_path: String,
    pub canonical_discriminator: Vec<u8>,
    pub embedded_name_digests: Vec<[u8; 32]>,
    pub same_occurrence_roles: BTreeMap<[u8; 32], String>,
}

impl ExecutionSplitChildSpec {
    pub fn entity(
        created: CreatedEntityRef,
        scheme_id: impl Into<String>,
        occurrence_key: impl Into<String>,
        role_path: impl Into<String>,
    ) -> Self {
        Self::new(
            PendingBirthTarget::CreatedEntity(created),
            scheme_id,
            occurrence_key,
            role_path,
        )
    }

    pub fn relation(
        created: CreatedRelationRef,
        scheme_id: impl Into<String>,
        occurrence_key: impl Into<String>,
        role_path: impl Into<String>,
    ) -> Self {
        Self::new(
            PendingBirthTarget::CreatedRelation(created),
            scheme_id,
            occurrence_key,
            role_path,
        )
    }

    fn new(
        target: PendingBirthTarget,
        scheme_id: impl Into<String>,
        occurrence_key: impl Into<String>,
        role_path: impl Into<String>,
    ) -> Self {
        Self {
            target,
            scheme_id: scheme_id.into(),
            occurrence_key: occurrence_key.into(),
            role_path: role_path.into(),
            canonical_discriminator: Vec::new(),
            embedded_name_digests: Vec::new(),
            same_occurrence_roles: BTreeMap::new(),
        }
    }

    pub fn with_discriminator<D: ApplicationNameDiscriminator>(mut self, discriminator: &D) -> Self {
        let (bytes, refs) = discriminator.canonical_bytes_with_references();
        self.canonical_discriminator = bytes;
        self.embedded_name_digests = refs;
        self
    }

    pub fn with_same_occurrence_reference<K>(
        mut self,
        name: WorthQueryName<K>,
        referenced_role: impl Into<String>,
    ) -> Self {
        self.same_occurrence_roles
            .insert(*name.as_bytes(), referenced_role.into());
        self
    }
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

impl ExecutionNamedMergeKeeper {
    pub fn mint_new<D: ApplicationNameDiscriminator>(
        scheme_id: impl Into<String>,
        occurrence_key: impl Into<String>,
        role_path: impl Into<String>,
        discriminator: &D,
    ) -> Self {
        let (bytes, refs) = discriminator.canonical_bytes_with_references();
        Self::MintNew {
            scheme_id: scheme_id.into(),
            occurrence_key: occurrence_key.into(),
            role_path: role_path.into(),
            canonical_discriminator: bytes,
            embedded_name_digests: refs,
        }
    }
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

    /// Registers an explicit `Split` posture whose newly created children will
    /// be bound by Relational to their allocated record identities.
    pub fn add_split_posture(
        &mut self,
        parent_target: LineageRecordTarget,
        parent_name: PersistentNameIdentity,
        children: Vec<ExecutionSplitChildSpec>,
    ) -> Result<&mut Self, ExecutionNamingDenial> {
        let mut lowered_children = Vec::with_capacity(children.len());
        for child in children {
            let expected_kind = match child.target {
                PendingBirthTarget::CreatedEntity(_) => ApplicationNameSubjectKind::Entity,
                PendingBirthTarget::CreatedRelation(_) => ApplicationNameSubjectKind::Relation,
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
            lowered_children.push(PendingSplitChild {
                target: child.target,
                scheme_id: child.scheme_id,
                occurrence_key: child.occurrence_key,
                role_path: child.role_path,
                referenced_names,
                canonical_discriminator: child.canonical_discriminator,
            });
        }
        self.push_pending_split(PendingSplitClaim {
            parent_target,
            parent_name,
            children: lowered_children,
        });
        Ok(self)
    }

    /// Registers an explicit `Merge` posture over an existing in-place keeper
    /// or newly created merge target.
    pub fn add_merge_posture(
        &mut self,
        parents: Vec<(LineageRecordTarget, PersistentNameIdentity)>,
        target: PendingMergeTarget,
        keeper: ExecutionNamedMergeKeeper,
    ) -> Result<&mut Self, ExecutionNamingDenial> {
        let expected_kind = match &target {
            PendingMergeTarget::Existing(LineageRecordTarget::Entity(_))
            | PendingMergeTarget::Created(PendingBirthTarget::CreatedEntity(_)) => {
                ApplicationNameSubjectKind::Entity
            }
            PendingMergeTarget::Existing(LineageRecordTarget::Relation(_))
            | PendingMergeTarget::Created(PendingBirthTarget::CreatedRelation(_)) => {
                ApplicationNameSubjectKind::Relation
            }
        };
        let keeper = match keeper {
            ExecutionNamedMergeKeeper::KeepParent(id) => PendingMergeKeeper::KeepParent(id),
            ExecutionNamedMergeKeeper::MintNew {
                scheme_id,
                occurrence_key,
                role_path,
                canonical_discriminator,
                embedded_name_digests,
            } => {
                let layout = self.require_scheme(expected_kind, &scheme_id, &role_path)?;
                if layout.require_authored_occurrence() && occurrence_key.trim().is_empty() {
                    return Err(ExecutionNamingDenial::MissingAuthoredOccurrenceKey { scheme_id });
                }
                let role_rank = layout.role_rank(&role_path).expect("validated role");
                let referenced_names = self.resolve_references(
                    layout,
                    &role_path,
                    role_rank,
                    &embedded_name_digests,
                    &BTreeMap::new(),
                )?;
                PendingMergeKeeper::MintNew {
                    scheme_id,
                    occurrence_key,
                    role_path,
                    referenced_names,
                    canonical_discriminator,
                }
            }
        };
        self.push_pending_merge(PendingMergeClaim {
            parents,
            target,
            keeper,
        });
        Ok(self)
    }
}
