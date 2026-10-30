use super::{commit_strategy_digest, commit_strategy_hex_digest, StrategyDigestBytes};
use crate::commit_strategies::data::{
    CommitStrategyDescriptor, CommitStrategyDescriptorDigest, CommitStrategyFamilyName,
    CommitStrategyId, CommitStrategySemanticName, CommitStrategyVersion, PersistentArtifactName,
    StrategyInputSchemaName, StrategyInputSchemaVersion, StrategyIntentName,
    StrategyOutputSchemaName, StrategyPacketContract, StrategyReadContract, StrategyReadCostClass,
    StrategyReadLocalityClass, StrategyReadScopeClass, StrategyTraversalBasis,
};

pub(crate) fn commit_strategy_descriptor_digest(
    id: CommitStrategyId,
    semantic_name: &CommitStrategySemanticName,
    family_name: &CommitStrategyFamilyName,
    version: CommitStrategyVersion,
    intent_name: &StrategyIntentName,
    input_schema_name: &StrategyInputSchemaName,
    input_schema_version: StrategyInputSchemaVersion,
    output_schema_name: &StrategyOutputSchemaName,
    read_contract: &StrategyReadContract,
    artifact_name: &PersistentArtifactName,
) -> CommitStrategyDescriptorDigest {
    CommitStrategyDescriptorDigest(commit_strategy_digest(
        "commit-strategy-descriptor-v1",
        |bytes| {
            bytes.u32(id.0);
            bytes.string(semantic_name.as_str());
            bytes.string(family_name.as_str());
            bytes.u16(version.major);
            bytes.u16(version.minor);
            bytes.string(intent_name.as_str());
            bytes.string(input_schema_name.as_str());
            bytes.u16(input_schema_version.0);
            bytes.string(output_schema_name.as_str());
            write_native_strategy_input_wire_tag(bytes);
            write_read_contract(bytes, read_contract);
            bytes.string(artifact_name.as_str());
        },
    ))
}

pub(crate) fn commit_strategy_registry_digest(descriptors: &[CommitStrategyDescriptor]) -> String {
    let mut canonical_descriptors = descriptors.to_vec();
    canonical_descriptors.sort_by(|left, right| {
        left.semantic_name()
            .cmp(right.semantic_name())
            .then_with(|| left.version().cmp(&right.version()))
            .then_with(|| left.id().cmp(&right.id()))
    });
    commit_strategy_hex_digest("commit-strategy-registry-v1", |bytes| {
        bytes.usize(canonical_descriptors.len());
        for descriptor in canonical_descriptors {
            bytes.u32(descriptor.id().0);
            bytes.string(descriptor.semantic_name().as_str());
            bytes.string(descriptor.family_name().as_str());
            bytes.u16(descriptor.version().major);
            bytes.u16(descriptor.version().minor);
            bytes.digest_bytes(&descriptor.digest().0);
        }
    })
}

fn write_native_strategy_input_wire_tag(bytes: &mut StrategyDigestBytes) {
    bytes.tag(1);
}

fn write_read_contract(bytes: &mut StrategyDigestBytes, contract: &StrategyReadContract) {
    write_scope_class(bytes, contract.scope_class);
    write_locality_class(bytes, contract.locality_class);
    write_traversal_basis(bytes, contract.traversal_basis);
    write_packet_contract(bytes, contract.packet_contract);
    write_cost_class(bytes, contract.cost_class);
}

fn write_scope_class(bytes: &mut StrategyDigestBytes, value: StrategyReadScopeClass) {
    bytes.tag(match value {
        StrategyReadScopeClass::ExplicitTargetsOnly => 1,
        StrategyReadScopeClass::KindBoundedScan => 2,
        StrategyReadScopeClass::PartitionBoundedScan => 3,
        StrategyReadScopeClass::BoundedNeighborhood => 4,
    });
}

fn write_locality_class(bytes: &mut StrategyDigestBytes, value: StrategyReadLocalityClass) {
    bytes.tag(match value {
        StrategyReadLocalityClass::SinglePartition => 1,
        StrategyReadLocalityClass::PartitionBounded => 2,
        StrategyReadLocalityClass::CrossPartitionBounded => 3,
    });
}

fn write_traversal_basis(bytes: &mut StrategyDigestBytes, value: StrategyTraversalBasis) {
    match value {
        StrategyTraversalBasis::NoTraversal => bytes.tag(1),
        StrategyTraversalBasis::AdjacencyBounded { max_depth } => {
            bytes.tag(2);
            bytes.u16(max_depth);
        }
    }
}

fn write_packet_contract(bytes: &mut StrategyDigestBytes, value: StrategyPacketContract) {
    bytes.tag(match value {
        StrategyPacketContract::ProjectionOnly => 1,
        StrategyPacketContract::PlannedPacketOnly => 2,
    });
}

fn write_cost_class(bytes: &mut StrategyDigestBytes, value: StrategyReadCostClass) {
    bytes.tag(match value {
        StrategyReadCostClass::ORequestedSurface => 1,
        StrategyReadCostClass::OPartitionBoundedSurface => 2,
    });
}
