use serde::{Deserialize, Serialize};

use crate::identity::data::KindId;
use crate::schema::data::{
    AcyclicityContractDeclaration, AllowedCycleClass, ConnectivityMinimumContractDeclaration,
    ConnectivityMinimumEnforcement, DirectedTraversalKind, PartitionIsolationContractDeclaration,
    PartitionIsolationMode,
};

use super::{
    CardinalityContractDeclaration, EndpointDeletionIntegrityDeclaration,
    EndpointDeletionIntegrityMode, EndpointKindContractDeclaration, MinimumCardinalityEnforcement,
    PairMinimumSemantics, SymmetryContractDeclaration, SymmetryMode, UniquenessContractDeclaration,
    UniquenessScope,
};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct RelationIntegrityPlanRevision(pub u128);

pub(crate) fn derive_relation_integrity_plan_revision(
    endpoint_kind_contracts: &[EndpointKindContractDeclaration],
    cardinality_contracts: &[CardinalityContractDeclaration],
    uniqueness_contracts: &[UniquenessContractDeclaration],
    symmetry_contracts: &[SymmetryContractDeclaration],
    endpoint_deletion_integrity_contracts: &[EndpointDeletionIntegrityDeclaration],
    acyclicity_contracts: &[AcyclicityContractDeclaration],
    partition_isolation_contracts: &[PartitionIsolationContractDeclaration],
    connectivity_minimum_contracts: &[ConnectivityMinimumContractDeclaration],
    aggregate_identity: Option<&super::AggregateIdentityContractDeclaration>,
) -> RelationIntegrityPlanRevision {
    let mut digest = RelationIntegrityRevisionDigest::new();
    digest.mix_endpoint_kind_contracts(endpoint_kind_contracts);
    digest.mix_cardinality_contracts(cardinality_contracts);
    digest.mix_uniqueness_contracts(uniqueness_contracts);
    digest.mix_symmetry_contracts(symmetry_contracts);
    digest.mix_endpoint_deletion_integrity_contracts(endpoint_deletion_integrity_contracts);
    digest.mix_acyclicity_contracts(acyclicity_contracts);
    digest.mix_partition_isolation_contracts(partition_isolation_contracts);
    digest.mix_connectivity_minimum_contracts(connectivity_minimum_contracts);
    if let Some(aggregate) = aggregate_identity {
        digest.mix_bytes(b"aggregate_identity_v1");
        digest.mix_bytes(&aggregate.canonical_bytes());
    }
    RelationIntegrityPlanRevision(digest.finish())
}

struct RelationIntegrityRevisionDigest {
    hash: u128,
}

impl RelationIntegrityRevisionDigest {
    fn new() -> Self {
        Self {
            hash: 0x6c62272e07bb014262b821756295c58d,
        }
    }
    const FNV_PRIME: u128 = 0x0000000001000000000000000000013B;

    fn finish(self) -> u128 {
        self.hash
    }

    fn mix_endpoint_kind_contracts(&mut self, contracts: &[EndpointKindContractDeclaration]) {
        for declaration in contracts {
            self.mix_bytes(b"endpoint_kind");
            self.mix_string(declaration.contract_id.as_str());
            self.mix_kind_ids(&declaration.allowed_source_kinds);
            self.mix_kind_ids(&declaration.allowed_target_kinds);
            self.mix_bytes(&[u8::from(declaration.self_edges_allowed)]);
            self.mix_bytes(match declaration.cross_context_policy {
                crate::config::data::CrossContextPolicy::AllowExplicit => b"allow_explicit",
                crate::config::data::CrossContextPolicy::SchemaControlled => b"schema_controlled",
                crate::config::data::CrossContextPolicy::Forbid => b"forbid",
            });
        }
    }

    fn mix_cardinality_contracts(&mut self, contracts: &[CardinalityContractDeclaration]) {
        for declaration in contracts {
            self.mix_bytes(b"cardinality");
            self.mix_string(declaration.contract_id.as_str());
            self.mix_optional_u64(declaration.source_max);
            self.mix_optional_u64(declaration.target_max);
            self.mix_optional_u64(declaration.pair_max);
            self.mix_optional_u64(declaration.source_min);
            self.mix_optional_u64(declaration.target_min);
            self.mix_optional_u64(declaration.pair_min);
            self.mix_bytes(match declaration.pair_min_semantics {
                PairMinimumSemantics::ObservedDirectedPairs => b"observed_directed_pairs",
            });
            self.mix_bytes(match declaration.minimum_enforcement {
                MinimumCardinalityEnforcement::CommitBoundary => b"minimum_commit_boundary",
                MinimumCardinalityEnforcement::CertificationBoundary => {
                    b"minimum_certification_boundary"
                }
            });
        }
    }

    fn mix_uniqueness_contracts(&mut self, contracts: &[UniquenessContractDeclaration]) {
        for declaration in contracts {
            self.mix_bytes(b"uniqueness");
            self.mix_string(declaration.contract_id.as_str());
            self.mix_bytes(match declaration.scope {
                UniquenessScope::DirectedSemanticEdge => b"directed",
                UniquenessScope::NormalizedSymmetricEdge => b"normalized",
            });
        }
    }

    fn mix_symmetry_contracts(&mut self, contracts: &[SymmetryContractDeclaration]) {
        for declaration in contracts {
            self.mix_bytes(b"symmetry");
            self.mix_string(declaration.contract_id.as_str());
            self.mix_bytes(match declaration.mode {
                SymmetryMode::CanonicalUndirected => b"canonical_undirected",
                SymmetryMode::PairedInverseRequired => b"paired_inverse_required",
                SymmetryMode::InverseProhibited => b"inverse_prohibited",
                SymmetryMode::PairedTwinRequired => b"paired_twin_required",
            });
        }
    }

    fn mix_endpoint_deletion_integrity_contracts(
        &mut self,
        contracts: &[EndpointDeletionIntegrityDeclaration],
    ) {
        for declaration in contracts {
            self.mix_bytes(b"endpoint_delete");
            self.mix_string(declaration.contract_id.as_str());
            self.mix_bytes(match declaration.mode {
                EndpointDeletionIntegrityMode::RejectDeleteWithLiveRelations => {
                    b"reject_delete_with_live_relations"
                }
                EndpointDeletionIntegrityMode::RequireRelationDeletionInSameCommit => {
                    b"require_relation_deletion_in_same_commit"
                }
                EndpointDeletionIntegrityMode::RequireRelationRetirement => {
                    b"require_relation_retirement"
                }
            });
        }
    }

    fn mix_acyclicity_contracts(&mut self, contracts: &[AcyclicityContractDeclaration]) {
        for declaration in contracts {
            self.mix_bytes(b"acyclicity");
            self.mix_string(declaration.contract_id.as_str());
            self.mix_bytes(match declaration.traversal_direction {
                DirectedTraversalKind::SourceToTarget => b"source_to_target",
            });
            self.mix_bytes(match declaration.allowed_cycle_class {
                AllowedCycleClass::NoCycles => b"no_cycles",
            });
        }
    }

    fn mix_partition_isolation_contracts(
        &mut self,
        contracts: &[PartitionIsolationContractDeclaration],
    ) {
        for declaration in contracts {
            self.mix_bytes(b"partition_isolation");
            self.mix_string(declaration.contract_id.as_str());
            self.mix_bytes(match declaration.isolation_mode {
                PartitionIsolationMode::SamePartitionEndpoints => b"same_partition_endpoints",
            });
        }
    }

    fn mix_connectivity_minimum_contracts(
        &mut self,
        contracts: &[ConnectivityMinimumContractDeclaration],
    ) {
        for declaration in contracts {
            self.mix_bytes(b"connectivity_minimum");
            self.mix_string(declaration.contract_id.as_str());
            self.mix_kind_ids(&declaration.source_kind_ids);
            self.mix_kind_ids(&declaration.target_kind_ids);
            self.mix_bytes(&u64::from(declaration.minimum_reachable_targets).to_le_bytes());
            self.mix_bytes(match declaration.enforcement_boundary {
                ConnectivityMinimumEnforcement::SnapshotPublication => b"snapshot_publication",
            });
        }
    }

    fn mix_optional_u64(&mut self, value: Option<u64>) {
        match value {
            None => self.mix_bytes(&[0]),
            Some(value) => {
                self.mix_bytes(&[1]);
                self.mix_bytes(&value.to_le_bytes());
            }
        }
    }

    fn mix_kind_ids(&mut self, kinds: &[KindId]) {
        self.mix_bytes(&(kinds.len() as u64).to_le_bytes());
        for kind in kinds {
            self.mix_bytes(&kind.0.to_le_bytes());
        }
    }

    fn mix_string(&mut self, value: &str) {
        self.mix_bytes(value.as_bytes());
    }

    fn mix_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.hash ^= *byte as u128;
            self.hash = self.hash.wrapping_mul(Self::FNV_PRIME);
        }
        self.hash ^= 0xff_u128;
        self.hash = self.hash.wrapping_mul(Self::FNV_PRIME);
    }
}
