use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::identity::data::KindId;

use super::{AspectContractPlanRevision, RelationIntegrityPlanRevision, SchemaId, SchemaVersionId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaAuthoritySnapshot {
    pub primary_schema_id: Option<SchemaId>,
    pub primary_schema_version_id: Option<SchemaVersionId>,
    pub entity_kinds: Vec<SchemaAuthorityKindSnapshot>,
    pub relation_kinds: Vec<SchemaAuthorityRelationSnapshot>,
    /// Complete executable recipes. Empty legacy authorities keep their text
    /// representation and V2 digest unchanged.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derived_relations: Vec<crate::derived_relation::DerivedRelationDefinitionParts>,
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
        self.entity_kinds
            .iter()
            .fold(
                entity_storage
                    .saturating_add(relation_storage)
                    .saturating_add(derived_storage),
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
mod tests {
    use super::*;
    use crate::schema::data::{
        CardinalityContractDeclaration, ConnectivityMinimumContractDeclaration,
        ConnectivityMinimumEnforcement, EndpointKindContractDeclaration,
        MinimumCardinalityEnforcement, PairMinimumSemantics, RelationIntegrityDeclarations,
    };

    #[test]
    fn digest_distinguishes_variable_length_registry_boundaries() {
        let mut boundary_name = vec![b'a'];
        boundary_name.extend_from_slice(b"s");
        boundary_name.extend_from_slice(&0_u32.to_le_bytes());
        boundary_name.extend_from_slice(&0_u128.to_le_bytes());
        boundary_name.extend_from_slice(b"entity_kind");
        boundary_name.extend_from_slice(&2_u32.to_le_bytes());
        boundary_name.extend_from_slice(b"b");
        let boundary_name = String::from_utf8(boundary_name).expect("test bytes are UTF-8");
        let one_kind = SchemaAuthoritySnapshot {
            primary_schema_id: Some(SchemaId("s".to_owned())),
            primary_schema_version_id: Some(SchemaVersionId(0)),
            entity_kinds: vec![kind(1, &boundary_name)],
            relation_kinds: Vec::new(),
            derived_relations: Vec::new(),
        };
        let two_kinds = SchemaAuthoritySnapshot {
            primary_schema_id: Some(SchemaId("s".to_owned())),
            primary_schema_version_id: Some(SchemaVersionId(0)),
            entity_kinds: vec![kind(1, "a"), kind(2, "b")],
            relation_kinds: Vec::new(),
            derived_relations: Vec::new(),
        };

        assert_ne!(
            schema_authority_snapshot_digest_bytes(&one_kind),
            schema_authority_snapshot_digest_bytes(&two_kinds),
            "length-prefixed canonical encoding must distinguish registry boundaries"
        );
    }

    #[test]
    fn digest_binds_relation_admission_and_delete_policies() {
        let mut cross_context = relation(
            CrossContextPolicy::AllowExplicit,
            CascadeDeletePolicy::RetainDanglingForAudit,
        );
        let mut cascade = cross_context.clone();
        cascade.cascade_delete_policy = CascadeDeletePolicy::CascadeDeleteRelations;
        cross_context.cross_context_policy = CrossContextPolicy::Forbid;

        let base = snapshot_with_relation(relation(
            CrossContextPolicy::AllowExplicit,
            CascadeDeletePolicy::RetainDanglingForAudit,
        ));
        assert_ne!(
            schema_authority_snapshot_digest_bytes(&base),
            schema_authority_snapshot_digest_bytes(&snapshot_with_relation(cross_context)),
            "cross-context policy is schema authority"
        );
        assert_ne!(
            schema_authority_snapshot_digest_bytes(&base),
            schema_authority_snapshot_digest_bytes(&snapshot_with_relation(cascade)),
            "cascade-delete policy is schema authority"
        );
    }

    #[test]
    fn digest_binds_absent_and_explicit_max_u64_cardinality() {
        let cardinality = |source_max| CardinalityContractDeclaration {
            contract_id: "cardinality".into(),
            source_max,
            target_max: None,
            pair_max: None,
            source_min: None,
            target_min: None,
            pair_min: None,
            pair_min_semantics: PairMinimumSemantics::ObservedDirectedPairs,
            minimum_enforcement: MinimumCardinalityEnforcement::CertificationBoundary,
        };
        let absent_revision = RelationIntegrityDeclarations::new(
            Vec::new(),
            vec![cardinality(None)],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
        .plan_revision;
        let explicit_max_revision = RelationIntegrityDeclarations::new(
            Vec::new(),
            vec![cardinality(Some(u64::MAX))],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
        .plan_revision;

        let mut absent = relation(
            CrossContextPolicy::AllowExplicit,
            CascadeDeletePolicy::RetainDanglingForAudit,
        );
        absent.relation_integrity_plan_revision = absent_revision;
        let mut explicit_max = absent.clone();
        explicit_max.relation_integrity_plan_revision = explicit_max_revision;

        assert_ne!(
            schema_authority_snapshot_digest_bytes(&snapshot_with_relation(absent)),
            schema_authority_snapshot_digest_bytes(&snapshot_with_relation(explicit_max)),
            "None and Some(u64::MAX) are distinct schema cardinality authority"
        );
    }

    #[test]
    fn digest_binds_endpoint_source_and_target_kind_boundaries() {
        let endpoint = |source, target| EndpointKindContractDeclaration {
            contract_id: "endpoint".into(),
            allowed_source_kinds: source,
            allowed_target_kinds: target,
            self_edges_allowed: false,
            cross_context_policy: CrossContextPolicy::AllowExplicit,
        };
        let source_one_target_two_three = RelationIntegrityDeclarations::new(
            vec![endpoint(vec![KindId(1)], vec![KindId(2), KindId(3)])],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
        .plan_revision;
        let source_one_two_target_three = RelationIntegrityDeclarations::new(
            vec![endpoint(vec![KindId(1), KindId(2)], vec![KindId(3)])],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
        .plan_revision;

        let mut left = relation(
            CrossContextPolicy::AllowExplicit,
            CascadeDeletePolicy::RetainDanglingForAudit,
        );
        left.relation_integrity_plan_revision = source_one_target_two_three;
        let mut right = left.clone();
        right.relation_integrity_plan_revision = source_one_two_target_three;

        assert_ne!(
            schema_authority_snapshot_digest_bytes(&snapshot_with_relation(left)),
            schema_authority_snapshot_digest_bytes(&snapshot_with_relation(right)),
            "source and target kind collections are separate schema authority"
        );
    }

    #[test]
    fn digest_binds_connectivity_source_and_target_kind_boundaries() {
        let connectivity = |source, target| ConnectivityMinimumContractDeclaration {
            contract_id: "connectivity".into(),
            source_kind_ids: source,
            target_kind_ids: target,
            minimum_reachable_targets: 1,
            enforcement_boundary: ConnectivityMinimumEnforcement::SnapshotPublication,
        };
        let source_one_target_two_three = RelationIntegrityDeclarations::new(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
        .with_connectivity_minimum_contracts(vec![connectivity(
            vec![KindId(1)],
            vec![KindId(2), KindId(3)],
        )])
        .plan_revision;
        let source_one_two_target_three = RelationIntegrityDeclarations::new(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
        .with_connectivity_minimum_contracts(vec![connectivity(
            vec![KindId(1), KindId(2)],
            vec![KindId(3)],
        )])
        .plan_revision;

        let mut left = relation(
            CrossContextPolicy::AllowExplicit,
            CascadeDeletePolicy::RetainDanglingForAudit,
        );
        left.relation_integrity_plan_revision = source_one_target_two_three;
        let mut right = left.clone();
        right.relation_integrity_plan_revision = source_one_two_target_three;

        assert_ne!(
            schema_authority_snapshot_digest_bytes(&snapshot_with_relation(left)),
            schema_authority_snapshot_digest_bytes(&snapshot_with_relation(right)),
            "connectivity source and target kind collections are separate schema authority"
        );
    }

    fn kind(kind_id: u32, kind_name: &str) -> SchemaAuthorityKindSnapshot {
        SchemaAuthorityKindSnapshot {
            kind_id: KindId(kind_id),
            kind_name: kind_name.to_owned(),
            schema_id: SchemaId("s".to_owned()),
            schema_version_id: SchemaVersionId(0),
            aspect_plan_revision: AspectContractPlanRevision(0),
        }
    }

    fn relation(
        cross_context_policy: CrossContextPolicy,
        cascade_delete_policy: CascadeDeletePolicy,
    ) -> SchemaAuthorityRelationSnapshot {
        SchemaAuthorityRelationSnapshot {
            kind_id: KindId(2),
            kind_name: "relation".to_owned(),
            schema_id: SchemaId("s".to_owned()),
            schema_version_id: SchemaVersionId(0),
            aspect_plan_revision: AspectContractPlanRevision(0),
            relation_integrity_plan_revision: RelationIntegrityPlanRevision(0),
            cross_context_policy,
            cascade_delete_policy,
        }
    }

    fn snapshot_with_relation(
        relation: SchemaAuthorityRelationSnapshot,
    ) -> SchemaAuthoritySnapshot {
        SchemaAuthoritySnapshot {
            primary_schema_id: Some(SchemaId("s".to_owned())),
            primary_schema_version_id: Some(SchemaVersionId(0)),
            entity_kinds: Vec::new(),
            relation_kinds: vec![relation],
            derived_relations: Vec::new(),
        }
    }
}
