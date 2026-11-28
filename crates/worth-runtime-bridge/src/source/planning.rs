use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::snapshot::PlannedTruthViewPacket;

use super::{AdmittedSourceContract, ValidatedSourceDeclaration};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedSourceReadPacketSet {
    contract: AdmittedSourceContract,
    validated_declaration: ValidatedSourceDeclaration,
    packets: Arc<[PlannedTruthViewPacket]>,
    packet_member_count: usize,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl PlannedSourceReadPacketSet {
    pub(crate) fn new(
        contract: AdmittedSourceContract,
        validated_declaration: ValidatedSourceDeclaration,
        packets: Vec<PlannedTruthViewPacket>,
    ) -> Self {
        assert!(
            !packets.is_empty(),
            "planned source packet set must contain at least one packet"
        );
        let packet_member_count = packets
            .iter()
            .map(|packet| packet.read_packet().reads().len())
            .sum::<usize>();
        let canonical_basis = Arc::<str>::from(format!(
            "planned-source-read-packet-set|contract={}|validated={}|packets={}",
            contract.digest(),
            validated_declaration.digest(),
            packets
                .iter()
                .map(|packet| packet.digest())
                .collect::<Vec<_>>()
                .join(","),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            contract,
            validated_declaration,
            packets: Arc::from(packets),
            packet_member_count,
            canonical_basis,
            digest: Arc::from(format!("planned-source-read-packet-set:sha256:{digest:x}")),
        }
    }

    pub fn contract(&self) -> &AdmittedSourceContract {
        &self.contract
    }

    pub fn validated_declaration(&self) -> &ValidatedSourceDeclaration {
        &self.validated_declaration
    }

    pub fn packets(&self) -> &[PlannedTruthViewPacket] {
        &self.packets
    }

    pub fn packet_count(&self) -> usize {
        self.packets.len()
    }

    pub fn packet_member_count(&self) -> usize {
        self.packet_member_count
    }

    pub fn first(&self) -> &PlannedTruthViewPacket {
        &self.packets[0]
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::PlannedSourceReadPacketSet;

    use crate::policy::BridgeDiagnosticsTier;
    use crate::snapshot::{
        BridgeDeliveryIntent, BridgeReplayMode, BridgeTruthViewAuthorityBasis,
        BridgeTruthViewSelector, HistoricalEvaluationDeclaration, PlannedTruthViewPacket,
        ResolvedTruthViewPolicy, SnapshotReadPacket, TruthViewReplayContinuity,
        TruthViewRetentionAdmission, TruthViewSourceCapability,
    };
    use crate::source::{
        AdmittedSourceContract, BridgeSourceCapability, BridgeSourceCapabilitySet,
        SourceDeclaration, SourceDeclarationIdentity, ValidatedSourceDeclaration,
    };

    fn admitted_contract() -> AdmittedSourceContract {
        let declaration = SourceDeclaration::new(
            SourceDeclarationIdentity::admit_bridge_owned("source:analysis-history"),
            BridgeTruthViewSelector::historical_commit(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            ),
            BridgeSourceCapabilitySet::new(vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::HistoricalRead,
            ]),
        );
        let registry = crate::source::AdmittedSourceRegistry::freeze(vec![declaration.clone()])
            .expect("source registry should freeze");
        registry
            .contract_for_declaration(&declaration)
            .expect("contract should exist")
            .clone()
    }

    fn planned_packet(contract: &AdmittedSourceContract) -> PlannedTruthViewPacket {
        let declaration = HistoricalEvaluationDeclaration::new(
            contract.declaration().selector().clone(),
            BridgeReplayMode::Disabled,
            BridgeDiagnosticsTier::Standard,
            BridgeDeliveryIntent::PrepareSignalEvaluation,
        );
        PlannedTruthViewPacket::new(
            declaration.clone(),
            ResolvedTruthViewPolicy::admitted(
                &declaration,
                TruthViewRetentionAdmission::HistoricalLookupRequired,
                TruthViewSourceCapability::HistoricalLookupAndSnapshotRead,
                TruthViewReplayContinuity::ReplayPermitted,
            ),
            BridgeTruthViewAuthorityBasis::from_resolved_snapshot(
                declaration.selector(),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
            SnapshotReadPacket::new(vec![]),
        )
    }

    #[test]
    fn planned_source_packet_set_is_canonical_for_same_inputs() {
        let contract = admitted_contract();
        let validated = ValidatedSourceDeclaration::from_contract(&contract);
        let left = PlannedSourceReadPacketSet::new(
            contract.clone(),
            validated.clone(),
            vec![planned_packet(&contract)],
        );
        let right = PlannedSourceReadPacketSet::new(
            contract,
            validated,
            vec![planned_packet(&admitted_contract())],
        );

        assert_eq!(left, right);
        assert_eq!(left.packet_count(), 1);
        assert_eq!(left.packet_member_count(), 0);
    }
}
