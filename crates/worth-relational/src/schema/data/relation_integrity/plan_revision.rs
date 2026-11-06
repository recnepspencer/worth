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
    derive_relation_integrity_plan_revision_for_compilation(
        crate::schema::SchemaCompilationControl::Configuration,
        endpoint_kind_contracts,
        cardinality_contracts,
        uniqueness_contracts,
        symmetry_contracts,
        endpoint_deletion_integrity_contracts,
        acyclicity_contracts,
        partition_isolation_contracts,
        connectivity_minimum_contracts,
        aggregate_identity,
    )
    .expect("configuration revision hashing has no request refusal")
}

pub(crate) fn derive_relation_integrity_plan_revision_for_compilation(
    control: crate::schema::SchemaCompilationControl<'_, '_>,
    endpoint_kind_contracts: &[EndpointKindContractDeclaration],
    cardinality_contracts: &[CardinalityContractDeclaration],
    uniqueness_contracts: &[UniquenessContractDeclaration],
    symmetry_contracts: &[SymmetryContractDeclaration],
    endpoint_deletion_integrity_contracts: &[EndpointDeletionIntegrityDeclaration],
    acyclicity_contracts: &[AcyclicityContractDeclaration],
    partition_isolation_contracts: &[PartitionIsolationContractDeclaration],
    connectivity_minimum_contracts: &[ConnectivityMinimumContractDeclaration],
    aggregate_identity: Option<&super::AggregateIdentityContractDeclaration>,
) -> Result<RelationIntegrityPlanRevision, crate::execution::RelationalExecutionDenialCause> {
    let mut digest = RelationIntegrityRevisionDigest::new(control);
    digest.mix_endpoint_kind_contracts(endpoint_kind_contracts)?;
    digest.mix_cardinality_contracts(cardinality_contracts)?;
    digest.mix_uniqueness_contracts(uniqueness_contracts)?;
    digest.mix_symmetry_contracts(symmetry_contracts)?;
    digest.mix_endpoint_deletion_integrity_contracts(endpoint_deletion_integrity_contracts)?;
    digest.mix_acyclicity_contracts(acyclicity_contracts)?;
    digest.mix_partition_isolation_contracts(partition_isolation_contracts)?;
    digest.mix_connectivity_minimum_contracts(connectivity_minimum_contracts)?;
    if let Some(aggregate) = aggregate_identity {
        digest.mix_bytes(b"aggregate_identity_v1")?;
        aggregate.try_write_canonical(&mut |part| digest.mix_segment(part))?;
        digest.finish_bytes();
    }
    control.checkpoint()?;
    Ok(RelationIntegrityPlanRevision(digest.finish()))
}

mod digest;
use digest::RelationIntegrityRevisionDigest;

#[cfg(test)]
mod tests;
