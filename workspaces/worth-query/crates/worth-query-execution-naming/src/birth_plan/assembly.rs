//! Deterministic execution-time birth-plan and posture assembly into
//! Relational's `CommitNamingPlan`.

use std::collections::{BTreeMap, BTreeSet};

use worth_query_declaration::facade::application_naming::{
    ApplicationNameDiscriminator, ApplicationNameSubjectKind, ErasedWorthQueryName, WorthQueryName,
};
use worth_relational::facade::lineage::{
    CommitNamingPlan, LineagePostureClaim, PendingBirthTarget, PendingCommitBirth,
    PersistentNameDiscriminatorReference, PersistentNameIdentity, PersistentNameReferenceBasis,
};
use worth_relational::facade::transactions::{CreatedEntityRef, CreatedRelationRef};

use crate::denial::ExecutionNamingDenial;
use crate::lowering::LoweredExecutionNameSchemeCatalog;

/// Execution-time specification for one pending named entity birth.
#[derive(Clone, Debug)]
pub struct ExecutionPendingEntityBirth {
    pub created: CreatedEntityRef,
    pub scheme_id: String,
    pub occurrence_key: String,
    pub role_path: String,
    pub parent_names: Vec<ErasedWorthQueryName>,
    pub canonical_discriminator: Vec<u8>,
    pub embedded_name_digests: Vec<[u8; 32]>,
    pub same_occurrence_roles: BTreeMap<[u8; 32], String>,
}

impl ExecutionPendingEntityBirth {
    pub fn new(
        created: CreatedEntityRef,
        scheme_id: impl Into<String>,
        occurrence_key: impl Into<String>,
        role_path: impl Into<String>,
    ) -> Self {
        Self {
            created,
            scheme_id: scheme_id.into(),
            occurrence_key: occurrence_key.into(),
            role_path: role_path.into(),
            parent_names: Vec::new(),
            canonical_discriminator: Vec::new(),
            embedded_name_digests: Vec::new(),
            same_occurrence_roles: BTreeMap::new(),
        }
    }

    pub fn with_parent<K>(mut self, parent: WorthQueryName<K>) -> Self {
        self.parent_names.push(parent.erased());
        self
    }

    pub fn with_discriminator<D: ApplicationNameDiscriminator>(mut self, discriminator: &D) -> Self {
        let (bytes, refs) = discriminator.canonical_bytes_with_references();
        self.canonical_discriminator = bytes;
        self.embedded_name_digests = refs;
        self
    }

    /// Declares that an embedded name digest was minted under `referenced_role`
    /// earlier in the exact same occurrence.
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

/// Execution-time specification for one pending named relation birth.
#[derive(Clone, Debug)]
pub struct ExecutionPendingRelationBirth {
    pub created: CreatedRelationRef,
    pub scheme_id: String,
    pub occurrence_key: String,
    pub role_path: String,
    pub parent_names: Vec<ErasedWorthQueryName>,
    pub canonical_discriminator: Vec<u8>,
    pub embedded_name_digests: Vec<[u8; 32]>,
    pub same_occurrence_roles: BTreeMap<[u8; 32], String>,
}

impl ExecutionPendingRelationBirth {
    pub fn new(
        created: CreatedRelationRef,
        scheme_id: impl Into<String>,
        occurrence_key: impl Into<String>,
        role_path: impl Into<String>,
    ) -> Self {
        Self {
            created,
            scheme_id: scheme_id.into(),
            occurrence_key: occurrence_key.into(),
            role_path: role_path.into(),
            parent_names: Vec::new(),
            canonical_discriminator: Vec::new(),
            embedded_name_digests: Vec::new(),
            same_occurrence_roles: BTreeMap::new(),
        }
    }

    pub fn with_parent<K>(mut self, parent: WorthQueryName<K>) -> Self {
        self.parent_names.push(parent.erased());
        self
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

/// Assembled, validated execution-time naming plan ready to attach to one
/// Relational transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionNameBirthPlan {
    commit_plan: CommitNamingPlan,
}

impl ExecutionNameBirthPlan {
    pub fn relational_plan(&self) -> &CommitNamingPlan {
        &self.commit_plan
    }

    pub fn into_relational_plan(self) -> CommitNamingPlan {
        self.commit_plan
    }

    pub fn pending_birth_count(&self) -> usize {
        self.commit_plan.pending_births.len()
    }

    pub fn posture_count(&self) -> usize {
        self.commit_plan.postures.len()
    }
}

/// Builder assembling and validating execution-time births and postures
/// against a [`LoweredExecutionNameSchemeCatalog`] and known admitted basis
/// identities.
pub struct ExecutionNameBirthPlanBuilder<'a> {
    catalog: &'a LoweredExecutionNameSchemeCatalog,
    admitted_identities: BTreeMap<[u8; 32], PersistentNameIdentity>,
    pending_births: Vec<(u32, PendingCommitBirth)>,
    postures: Vec<LineagePostureClaim>,
}

impl<'a> ExecutionNameBirthPlanBuilder<'a> {
    /// Creates a new birth-plan builder backed by `catalog` and the known
    /// Relational `PersistentNameIdentity` values visible at the commit basis.
    pub fn new(
        catalog: &'a LoweredExecutionNameSchemeCatalog,
        basis_identities: impl IntoIterator<Item = PersistentNameIdentity>,
    ) -> Self {
        let admitted_identities = basis_identities
            .into_iter()
            .map(|id| (*id.as_bytes(), id))
            .collect();
        Self {
            catalog,
            admitted_identities,
            pending_births: Vec::new(),
            postures: Vec::new(),
        }
    }

    /// Registers a pending entity birth after validating scheme membership,
    /// authored occurrence requirement, split discriminator presence, and
    /// acyclic same-occurrence role references.
    pub fn add_entity_birth(
        &mut self,
        birth: ExecutionPendingEntityBirth,
    ) -> Result<&mut Self, ExecutionNamingDenial> {
        let (rank, pending) = self.validate_pending_birth(
            ApplicationNameSubjectKind::Entity,
            PendingBirthTarget::CreatedEntity(birth.created),
            birth.scheme_id,
            birth.occurrence_key,
            birth.role_path,
            &birth.parent_names,
            birth.canonical_discriminator,
            &birth.embedded_name_digests,
            &birth.same_occurrence_roles,
        )?;
        self.pending_births.push((rank, pending));
        Ok(self)
    }

    /// Registers a pending relation birth after validating scheme membership,
    /// authored occurrence requirement, split discriminator presence, and
    /// acyclic same-occurrence role references.
    pub fn add_relation_birth(
        &mut self,
        birth: ExecutionPendingRelationBirth,
    ) -> Result<&mut Self, ExecutionNamingDenial> {
        let (rank, pending) = self.validate_pending_birth(
            ApplicationNameSubjectKind::Relation,
            PendingBirthTarget::CreatedRelation(birth.created),
            birth.scheme_id,
            birth.occurrence_key,
            birth.role_path,
            &birth.parent_names,
            birth.canonical_discriminator,
            &birth.embedded_name_digests,
            &birth.same_occurrence_roles,
        )?;
        self.pending_births.push((rank, pending));
        Ok(self)
    }

    pub(super) fn push_posture(&mut self, claim: LineagePostureClaim) {
        self.postures.push(claim);
    }

    /// Canonicalizes pending births in deterministic meaning order
    /// `(scheme_id, occurrence_key, role_rank, role_path, parent_identities, canonical_discriminator)`
    /// independent of caller submission order or worker scheduling.
    pub fn build(mut self) -> ExecutionNameBirthPlan {
        self.pending_births.sort_by(|(rank_a, a), (rank_b, b)| {
            (
                &a.scheme_id,
                &a.occurrence_key,
                rank_a,
                &a.role_path,
                &a.parent_identities,
                &a.canonical_discriminator,
            )
                .cmp(&(
                    &b.scheme_id,
                    &b.occurrence_key,
                    rank_b,
                    &b.role_path,
                    &b.parent_identities,
                    &b.canonical_discriminator,
                ))
        });
        ExecutionNameBirthPlan {
            commit_plan: CommitNamingPlan {
                scheme_registry: self.catalog.relational_registry().clone(),
                pending_births: self
                    .pending_births
                    .into_iter()
                    .map(|(_, birth)| birth)
                    .collect(),
                postures: self.postures,
            },
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn validate_pending_birth(
        &self,
        expected_kind: ApplicationNameSubjectKind,
        target: PendingBirthTarget,
        scheme_id: String,
        occurrence_key: String,
        role_path: String,
        parent_names: &[ErasedWorthQueryName],
        canonical_discriminator: Vec<u8>,
        embedded_name_digests: &[[u8; 32]],
        same_occurrence_roles: &BTreeMap<[u8; 32], String>,
    ) -> Result<(u32, PendingCommitBirth), ExecutionNamingDenial> {
        let layout = self.require_scheme(expected_kind, &scheme_id, &role_path)?;
        if layout.require_authored_occurrence() && occurrence_key.trim().is_empty() {
            return Err(ExecutionNamingDenial::MissingAuthoredOccurrenceKey { scheme_id });
        }
        if !parent_names.is_empty() && layout.split_capable() && canonical_discriminator.is_empty()
        {
            return Err(ExecutionNamingDenial::MissingSplitDiscriminator {
                scheme_id,
                role_path,
            });
        }
        let role_rank = layout.role_rank(&role_path).expect("validated role");
        let mut parent_identities = Vec::with_capacity(parent_names.len());
        for parent in parent_names {
            let identity = self
                .admitted_identities
                .get(parent.as_bytes())
                .copied()
                .ok_or_else(|| ExecutionNamingDenial::UnresolvedReferencedName {
                    scheme_id: scheme_id.clone(),
                    role_path: role_path.clone(),
                    digest: *parent.as_bytes(),
                })?;
            parent_identities.push(identity);
        }
        parent_identities.sort_unstable();
        let referenced_names = self.resolve_references(
            layout,
            &role_path,
            role_rank,
            embedded_name_digests,
            same_occurrence_roles,
        )?;
        Ok((
            role_rank,
            PendingCommitBirth {
                target,
                scheme_id,
                occurrence_key,
                role_path,
                parent_identities,
                referenced_names,
                canonical_discriminator,
            },
        ))
    }

    pub(super) fn require_scheme(
        &self,
        expected_kind: ApplicationNameSubjectKind,
        scheme_id: &str,
        role_path: &str,
    ) -> Result<&worth_query_installation::facade::InstalledNameSchemeLayout, ExecutionNamingDenial>
    {
        let layout = self.catalog.scheme_by_id(scheme_id).ok_or_else(|| {
            ExecutionNamingDenial::UnknownScheme {
                scheme_id: scheme_id.to_owned(),
            }
        })?;
        if layout.subject_kind() != expected_kind {
            return Err(ExecutionNamingDenial::SubjectKindMismatch {
                scheme_id: scheme_id.to_owned(),
                expected: layout.subject_kind(),
                actual: expected_kind,
            });
        }
        if layout.role_rank(role_path).is_none() {
            return Err(ExecutionNamingDenial::UndeclaredRole {
                scheme_id: scheme_id.to_owned(),
                role_path: role_path.to_owned(),
            });
        }
        Ok(layout)
    }

    pub(super) fn resolve_references(
        &self,
        layout: &worth_query_installation::facade::InstalledNameSchemeLayout,
        role_path: &str,
        role_rank: u32,
        embedded_name_digests: &[[u8; 32]],
        same_occurrence_roles: &BTreeMap<[u8; 32], String>,
    ) -> Result<Vec<PersistentNameDiscriminatorReference>, ExecutionNamingDenial> {
        let mut seen = BTreeSet::new();
        let mut resolved = Vec::new();
        for &digest in embedded_name_digests {
            if !seen.insert(digest) {
                continue;
            }
            let identity = self.admitted_identities.get(&digest).copied().ok_or_else(|| {
                ExecutionNamingDenial::UnresolvedReferencedName {
                    scheme_id: layout.scheme_id().to_owned(),
                    role_path: role_path.to_owned(),
                    digest,
                }
            })?;
            let basis = if let Some(referenced_role) = same_occurrence_roles.get(&digest) {
                let referenced_rank = layout.role_rank(referenced_role);
                if !referenced_rank.is_some_and(|rank| rank < role_rank) {
                    return Err(ExecutionNamingDenial::NonPrecedingSameOccurrenceRole {
                        scheme_id: layout.scheme_id().to_owned(),
                        role_path: role_path.to_owned(),
                        referenced_role: referenced_role.clone(),
                    });
                }
                PersistentNameReferenceBasis::SameOccurrence {
                    role: referenced_role.clone(),
                }
            } else {
                PersistentNameReferenceBasis::Basis
            };
            resolved.push(PersistentNameDiscriminatorReference { identity, basis });
        }
        Ok(resolved)
    }
}
