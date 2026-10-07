use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::snapshot::MaterializedTruthViewObservation;

use super::PlannedSourceReadPacketSet;

pub struct MaterializedTruthViewPacketSet {
    planned_packet_set: PlannedSourceReadPacketSet,
    observations: Vec<MaterializedTruthViewObservation>,
    materialization_count: usize,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl MaterializedTruthViewPacketSet {
    pub(crate) fn new(
        planned_packet_set: PlannedSourceReadPacketSet,
        observations: Vec<MaterializedTruthViewObservation>,
    ) -> Self {
        assert!(
            !observations.is_empty(),
            "materialized truth-view packet set must contain at least one observation"
        );
        assert_eq!(
            planned_packet_set.packet_count(),
            observations.len(),
            "materialized truth-view packet set must preserve planned packet count"
        );
        let observation_basis = observations
            .iter()
            .map(|observation| {
                format!(
                    "{}|{}|{:?}|{}",
                    observation.planned().digest(),
                    observation.snapshot_identity().as_str(),
                    observation.materialization_path(),
                    observation.snapshot_token().snapshot_identity().as_str(),
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let canonical_basis = Arc::<str>::from(format!(
            "materialized-truth-view-packet-set|planned={}|observations={}",
            planned_packet_set.digest(),
            observation_basis,
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            planned_packet_set,
            materialization_count: observations.len(),
            observations,
            canonical_basis,
            digest: Arc::from(format!(
                "materialized-truth-view-packet-set:sha256:{digest:x}"
            )),
        }
    }

    pub fn planned_packet_set(&self) -> &PlannedSourceReadPacketSet {
        &self.planned_packet_set
    }

    pub fn observations(&self) -> &[MaterializedTruthViewObservation] {
        &self.observations
    }

    pub fn materialization_count(&self) -> usize {
        self.materialization_count
    }

    pub fn first(&self) -> &MaterializedTruthViewObservation {
        &self.observations[0]
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
    use worth_foundational::facade::AspectValue;

    use super::MaterializedTruthViewPacketSet;
    use crate::diagnostics::BridgeHistoricalMaterializationPath;

    use crate::policy::BridgeDiagnosticsTier;
    use crate::snapshot::{
        AdmittedSnapshotContext, BridgeDeliveryIntent, BridgeReplayMode, BridgeSnapshotContext,
        BridgeSnapshotToken, BridgeTruthViewAuthorityBasis, BridgeTruthViewSelector,
        HistoricalEvaluationDeclaration, PlannedTruthViewPacket, ResolvedTruthViewPolicy,
        SnapshotReadPacket, SnapshotReadPacketResult, TruthSnapshotIdentity, TruthSnapshotReader,
        TruthViewReplayContinuity, TruthViewRetentionAdmission, TruthViewSourceCapability,
    };
    use crate::source::{
        AdmittedSourceContract, BridgeSourceCapability, BridgeSourceCapabilitySet,
        PlannedSourceReadPacketSet, SourceDeclaration, SourceDeclarationIdentity,
        ValidatedSourceDeclaration,
    };

    #[derive(Debug)]
    struct FixtureReader;

    impl TruthSnapshotReader for FixtureReader {
        fn snapshot_identity(&self) -> TruthSnapshotIdentity {
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a")
        }

        fn read_packet(
            &self,
            request: &SnapshotReadPacket,
            _execution: worth_execution::ExecutionRequest<'_, '_>,
        ) -> Result<SnapshotReadPacketResult, crate::snapshot::BridgeSnapshotReadError> {
            Ok(SnapshotReadPacketResult::new(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                request
                    .reads()
                    .iter()
                    .map(|read| {
                        crate::snapshot::SnapshotReadRecord::for_request(
                            read,
                            AspectValue::String("value".into()),
                        )
                    })
                    .collect(),
            ))
        }
    }

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

    fn planned_source_packet_set(contract: &AdmittedSourceContract) -> PlannedSourceReadPacketSet {
        let declaration = HistoricalEvaluationDeclaration::new(
            contract.declaration().selector().clone(),
            BridgeReplayMode::Disabled,
            BridgeDiagnosticsTier::Standard,
            BridgeDeliveryIntent::PrepareSignalEvaluation,
        );
        let packet = PlannedTruthViewPacket::new(
            declaration.clone(),
            ResolvedTruthViewPolicy::admitted(
                &declaration,
                TruthViewRetentionAdmission::HistoricalLookupRequired,
                TruthViewSourceCapability::HistoricalLookupAndSnapshotRead,
                TruthViewReplayContinuity::ReplayPermitted,
            ),
            BridgeTruthViewAuthorityBasis::from_resolved_envelope(
                declaration.selector(),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
            SnapshotReadPacket::new(vec![]),
        );
        PlannedSourceReadPacketSet::new(
            contract.clone(),
            ValidatedSourceDeclaration::from_contract(contract),
            vec![packet],
        )
    }

    #[test]
    fn materialized_source_packet_set_is_canonical_for_same_inputs() {
        let contract = admitted_contract();
        let planned = planned_source_packet_set(&contract);
        let snapshot =
            BridgeSnapshotContext::bind(Box::new(FixtureReader) as Box<dyn TruthSnapshotReader>);
        let admitted = AdmittedSnapshotContext::admit_for(
            snapshot,
            &crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        )
        .expect("snapshot should admit");
        let observation = crate::snapshot::MaterializedTruthViewObservation::new(
            planned.first().clone(),
            BridgeSnapshotToken::issued(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                "test-materialized-source-packet-set",
            ),
            BridgeHistoricalMaterializationPath::CommitEnvelopeSnapshot,
            admitted,
            crate::policy::BridgeExecutionPolicyBaseline::development(),
        );

        let left = MaterializedTruthViewPacketSet::new(planned.clone(), vec![observation]);
        let snapshot =
            BridgeSnapshotContext::bind(Box::new(FixtureReader) as Box<dyn TruthSnapshotReader>);
        let admitted = AdmittedSnapshotContext::admit_for(
            snapshot,
            &crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        )
        .expect("snapshot should admit");
        let observation = crate::snapshot::MaterializedTruthViewObservation::new(
            planned.first().clone(),
            BridgeSnapshotToken::issued(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                "test-materialized-source-packet-set",
            ),
            BridgeHistoricalMaterializationPath::CommitEnvelopeSnapshot,
            admitted,
            crate::policy::BridgeExecutionPolicyBaseline::development(),
        );
        let right = MaterializedTruthViewPacketSet::new(planned, vec![observation]);

        assert_eq!(left.digest(), right.digest());
        assert_eq!(left.materialization_count(), 1);
    }
}
