use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::identity::data::KindId;

use super::{AspectContractPlanRevision, RelationIntegrityPlanRevision, SchemaId, SchemaVersionId};

mod encoding;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SchemaAuthoritySnapshot {
    pub primary_schema_id: Option<SchemaId>,
    pub primary_schema_version_id: Option<SchemaVersionId>,
    pub entity_kinds: Vec<SchemaAuthorityKindSnapshot>,
    pub relation_kinds: Vec<SchemaAuthorityRelationSnapshot>,
    /// Complete executable recipes. Empty legacy authorities keep their text
    /// representation and V2 digest unchanged.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derived_relations: Vec<crate::derived_relation::DerivedRelationDefinitionParts>,
    /// The optional finite member-domain authority. Physical-only encodings
    /// and digests acquire no new field or suffix.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub projected_member_domains: Vec<crate::projected_member_domain::ProjectedMemberDomainParts>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaAuthorityKindSnapshot {
    pub kind_id: KindId,
    pub kind_name: String,
    pub schema_id: SchemaId,
    pub schema_version_id: SchemaVersionId,
    pub aspect_plan_revision: AspectContractPlanRevision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaAuthorityRelationSnapshot {
    pub kind_id: KindId,
    pub kind_name: String,
    pub schema_id: SchemaId,
    pub schema_version_id: SchemaVersionId,
    pub aspect_plan_revision: AspectContractPlanRevision,
    pub relation_integrity_plan_revision: RelationIntegrityPlanRevision,
    pub cross_context_policy: CrossContextPolicy,
    pub cascade_delete_policy: CascadeDeletePolicy,
}

impl SchemaAuthoritySnapshot {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        let entity_storage = (self.entity_kinds.capacity() as u64)
            .saturating_mul(std::mem::size_of::<SchemaAuthorityKindSnapshot>() as u64);
        let relation_storage = (self.relation_kinds.capacity() as u64)
            .saturating_mul(std::mem::size_of::<SchemaAuthorityRelationSnapshot>() as u64);
        let derived_storage = (self.derived_relations.capacity() as u64)
            .saturating_mul(std::mem::size_of::<
                crate::derived_relation::DerivedRelationDefinitionParts,
            >() as u64)
            .saturating_add(
                self.derived_relations
                    .iter()
                    .map(|definition| definition.owned_capacity_bytes() as u64)
                    .sum(),
            );
        let domain_storage = (self.projected_member_domains.capacity() as u64)
            .saturating_mul(
                size_of::<crate::projected_member_domain::ProjectedMemberDomainParts>() as u64,
            )
            .saturating_add(
                self.projected_member_domains
                    .iter()
                    .map(|domain| domain.owned_bytes() as u64)
                    .sum(),
            );
        self.entity_kinds
            .iter()
            .fold(
                entity_storage
                    .saturating_add(relation_storage)
                    .saturating_add(derived_storage)
                    .saturating_add(domain_storage),
                |bytes, kind| bytes.saturating_add(kind.kind_name.capacity() as u64),
            )
            .saturating_add(
                self.relation_kinds
                    .iter()
                    .map(|kind| kind.kind_name.capacity() as u64)
                    .sum(),
            )
    }
}

pub fn schema_authority_snapshot_digest_bytes(snapshot: &SchemaAuthoritySnapshot) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"WORTH_SCHEMA_AUTHORITY_SNAPSHOT_V2");
    if let Some(schema_id) = &snapshot.primary_schema_id {
        hasher.update([1]);
        update_bytes(&mut hasher, schema_id.0.as_bytes());
    } else {
        hasher.update([0]);
    }
    if let Some(schema_version_id) = snapshot.primary_schema_version_id {
        hasher.update([1]);
        hasher.update(schema_version_id.0.to_le_bytes());
    } else {
        hasher.update([0]);
    }
    update_count(&mut hasher, snapshot.entity_kinds.len());
    for entity_kind in &snapshot.entity_kinds {
        hasher.update(b"entity_kind\0");
        hasher.update(entity_kind.kind_id.0.to_le_bytes());
        update_bytes(&mut hasher, entity_kind.kind_name.as_bytes());
        update_bytes(&mut hasher, entity_kind.schema_id.0.as_bytes());
        hasher.update(entity_kind.schema_version_id.0.to_le_bytes());
        hasher.update(entity_kind.aspect_plan_revision.0.to_le_bytes());
    }
    update_count(&mut hasher, snapshot.relation_kinds.len());
    for relation_kind in &snapshot.relation_kinds {
        hasher.update(b"relation_kind\0");
        hasher.update(relation_kind.kind_id.0.to_le_bytes());
        update_bytes(&mut hasher, relation_kind.kind_name.as_bytes());
        update_bytes(&mut hasher, relation_kind.schema_id.0.as_bytes());
        hasher.update(relation_kind.schema_version_id.0.to_le_bytes());
        hasher.update(relation_kind.aspect_plan_revision.0.to_le_bytes());
        hasher.update(
            relation_kind
                .relation_integrity_plan_revision
                .0
                .to_le_bytes(),
        );
        hasher.update([cross_context_policy_tag(relation_kind.cross_context_policy)]);
        hasher.update([cascade_delete_policy_tag(
            relation_kind.cascade_delete_policy,
        )]);
    }
    crate::derived_relation::canonical::update_digest(&mut hasher, &snapshot.derived_relations);
    crate::projected_member_domain::canonical::update_digest(
        &mut hasher,
        &snapshot.projected_member_domains,
    );
    let digest = hasher.finalize();
    let mut out = [0_u8; 32];
    out.copy_from_slice(&digest);
    out
}

fn update_count(hasher: &mut Sha256, count: usize) {
    hasher.update((count as u64).to_le_bytes());
}

fn update_bytes(hasher: &mut Sha256, bytes: &[u8]) {
    update_count(hasher, bytes.len());
    hasher.update(bytes);
}

fn cross_context_policy_tag(policy: CrossContextPolicy) -> u8 {
    match policy {
        CrossContextPolicy::AllowExplicit => 0,
        CrossContextPolicy::SchemaControlled => 1,
        CrossContextPolicy::Forbid => 2,
    }
}

fn cascade_delete_policy_tag(policy: CascadeDeletePolicy) -> u8 {
    match policy {
        CascadeDeletePolicy::RetainDanglingForAudit => 0,
        CascadeDeletePolicy::CascadeDeleteRelations => 1,
    }
}

#[cfg(test)]
mod tests;
