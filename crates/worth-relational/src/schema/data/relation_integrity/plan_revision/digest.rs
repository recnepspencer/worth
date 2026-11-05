//! The one relation revision byte mixer, with original-request checkpoints.
use super::*;

pub(super) struct RelationIntegrityRevisionDigest<'request, 'authority> {
    control: crate::schema::SchemaCompilationControl<'request, 'authority>,
    hash: u128,
}

impl<'request, 'authority> RelationIntegrityRevisionDigest<'request, 'authority> {
    pub(super) fn new(
        control: crate::schema::SchemaCompilationControl<'request, 'authority>,
    ) -> Self {
        Self {
            control,
            hash: 0x6c62272e07bb014262b821756295c58d,
        }
    }
    const FNV_PRIME: u128 = 0x0000000001000000000000000000013B;

    pub(super) fn finish(self) -> u128 {
        self.hash
    }

    pub(super) fn mix_endpoint_kind_contracts(
        &mut self,
        contracts: &[EndpointKindContractDeclaration],
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        for declaration in contracts {
            self.control.checkpoint()?;
            self.mix_bytes(b"endpoint_kind")?;
            self.mix_string(declaration.contract_id.as_str())?;
            self.mix_kind_ids(&declaration.allowed_source_kinds)?;
            self.mix_kind_ids(&declaration.allowed_target_kinds)?;
            self.mix_bytes(&[u8::from(declaration.self_edges_allowed)])?;
            self.mix_bytes(match declaration.cross_context_policy {
                crate::config::data::CrossContextPolicy::AllowExplicit => b"allow_explicit",
                crate::config::data::CrossContextPolicy::SchemaControlled => b"schema_controlled",
                crate::config::data::CrossContextPolicy::Forbid => b"forbid",
            })?;
        }
        Ok(())
    }

    pub(super) fn mix_cardinality_contracts(
        &mut self,
        contracts: &[CardinalityContractDeclaration],
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        for declaration in contracts {
            self.control.checkpoint()?;
            self.mix_bytes(b"cardinality")?;
            self.mix_string(declaration.contract_id.as_str())?;
            self.mix_optional_u64(declaration.source_max)?;
            self.mix_optional_u64(declaration.target_max)?;
            self.mix_optional_u64(declaration.pair_max)?;
            self.mix_optional_u64(declaration.source_min)?;
            self.mix_optional_u64(declaration.target_min)?;
            self.mix_optional_u64(declaration.pair_min)?;
            self.mix_bytes(match declaration.pair_min_semantics {
                PairMinimumSemantics::ObservedDirectedPairs => b"observed_directed_pairs",
            })?;
            self.mix_bytes(match declaration.minimum_enforcement {
                MinimumCardinalityEnforcement::CommitBoundary => b"minimum_commit_boundary",
                MinimumCardinalityEnforcement::CertificationBoundary => {
                    b"minimum_certification_boundary"
                }
            })?;
        }
        Ok(())
    }

    pub(super) fn mix_uniqueness_contracts(
        &mut self,
        contracts: &[UniquenessContractDeclaration],
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        for declaration in contracts {
            self.control.checkpoint()?;
            self.mix_bytes(b"uniqueness")?;
            self.mix_string(declaration.contract_id.as_str())?;
            self.mix_bytes(match declaration.scope {
                UniquenessScope::DirectedSemanticEdge => b"directed",
                UniquenessScope::NormalizedSymmetricEdge => b"normalized",
            })?;
        }
        Ok(())
    }

    pub(super) fn mix_symmetry_contracts(
        &mut self,
        contracts: &[SymmetryContractDeclaration],
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        for declaration in contracts {
            self.control.checkpoint()?;
            self.mix_bytes(b"symmetry")?;
            self.mix_string(declaration.contract_id.as_str())?;
            self.mix_bytes(match declaration.mode {
                SymmetryMode::CanonicalUndirected => b"canonical_undirected",
                SymmetryMode::PairedInverseRequired => b"paired_inverse_required",
                SymmetryMode::InverseProhibited => b"inverse_prohibited",
                SymmetryMode::PairedTwinRequired => b"paired_twin_required",
            })?;
        }
        Ok(())
    }

    pub(super) fn mix_endpoint_deletion_integrity_contracts(
        &mut self,
        contracts: &[EndpointDeletionIntegrityDeclaration],
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        for declaration in contracts {
            self.control.checkpoint()?;
            self.mix_bytes(b"endpoint_delete")?;
            self.mix_string(declaration.contract_id.as_str())?;
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
            })?;
        }
        Ok(())
    }

    pub(super) fn mix_acyclicity_contracts(
        &mut self,
        contracts: &[AcyclicityContractDeclaration],
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        for declaration in contracts {
            self.control.checkpoint()?;
            self.mix_bytes(b"acyclicity")?;
            self.mix_string(declaration.contract_id.as_str())?;
            self.mix_bytes(match declaration.traversal_direction {
                DirectedTraversalKind::SourceToTarget => b"source_to_target",
            })?;
            self.mix_bytes(match declaration.allowed_cycle_class {
                AllowedCycleClass::NoCycles => b"no_cycles",
            })?;
        }
        Ok(())
    }

    pub(super) fn mix_partition_isolation_contracts(
        &mut self,
        contracts: &[PartitionIsolationContractDeclaration],
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        for declaration in contracts {
            self.control.checkpoint()?;
            self.mix_bytes(b"partition_isolation")?;
            self.mix_string(declaration.contract_id.as_str())?;
            self.mix_bytes(match declaration.isolation_mode {
                PartitionIsolationMode::SamePartitionEndpoints => b"same_partition_endpoints",
            })?;
        }
        Ok(())
    }

    pub(super) fn mix_connectivity_minimum_contracts(
        &mut self,
        contracts: &[ConnectivityMinimumContractDeclaration],
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        for declaration in contracts {
            self.control.checkpoint()?;
            self.mix_bytes(b"connectivity_minimum")?;
            self.mix_string(declaration.contract_id.as_str())?;
            self.mix_kind_ids(&declaration.source_kind_ids)?;
            self.mix_kind_ids(&declaration.target_kind_ids)?;
            self.mix_bytes(&u64::from(declaration.minimum_reachable_targets).to_le_bytes())?;
            self.mix_bytes(match declaration.enforcement_boundary {
                ConnectivityMinimumEnforcement::SnapshotPublication => b"snapshot_publication",
            })?;
        }
        Ok(())
    }

    pub(super) fn mix_optional_u64(
        &mut self,
        value: Option<u64>,
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        match value {
            None => self.mix_bytes(&[0])?,
            Some(value) => {
                self.mix_bytes(&[1])?;
                self.mix_bytes(&value.to_le_bytes())?;
            }
        }
        Ok(())
    }

    pub(super) fn mix_kind_ids(
        &mut self,
        kinds: &[KindId],
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        self.mix_bytes(&(kinds.len() as u64).to_le_bytes())?;
        for kind in kinds {
            self.control.checkpoint()?;
            self.mix_bytes(&kind.0.to_le_bytes())?;
        }
        Ok(())
    }

    pub(super) fn mix_string(
        &mut self,
        value: &str,
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        self.mix_bytes(value.as_bytes())?;
        Ok(())
    }

    pub(super) fn mix_bytes(
        &mut self,
        bytes: &[u8],
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        self.mix_segment(bytes)?;
        self.finish_bytes();
        Ok(())
    }

    pub(super) fn mix_segment(
        &mut self,
        bytes: &[u8],
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        for chunk in bytes.chunks(64 * 1024) {
            self.control.checkpoint()?;
            for byte in chunk {
                self.hash ^= *byte as u128;
                self.hash = self.hash.wrapping_mul(Self::FNV_PRIME);
            }
        }
        Ok(())
    }

    pub(super) fn finish_bytes(&mut self) {
        self.hash ^= 0xff_u128;
        self.hash = self.hash.wrapping_mul(Self::FNV_PRIME);
    }
}
